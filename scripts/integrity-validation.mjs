// SDK-only orchestration of source verification, planted faults and generated-model execution.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import crypto from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {loadCorpus,sourceRecords,leanDriver,requireFact} from './integrity-vectors.mjs';
const script=fileURLToPath(import.meta.url);
const shaFile='src/Uorc/Integrity/Sha256.lex.tex';
const sources=[shaFile,'src/Uorc/Integrity/BoundedChain.lex.tex','src/Uorc/Integrity/Contract.lex.tex'];
const sha=x=>crypto.createHash('sha256').update(x).digest('hex');
export function plant(source,mutation){
  const rows=[...source.matchAll(/\\semanticdata\{(.*)\}\n/g)];requireFact(rows.length===1,'one SHA semantic module required');
  const module=JSON.parse(rows[0][1]);const decls=module.declarations.filter(d=>d.kind==='definition'&&d.name===mutation.definition);
  requireFact(decls.length===1,'mutation requires one computational owner');let changed=0;
  const visit=node=>{
    if(Array.isArray(node)){for(const child of node)visit(child);return;}
    if(node&&typeof node==='object'){
      if(node.kind==='integer'&&node.value===mutation.before){node.value=mutation.after;changed++;}
      for(const child of Object.values(node))visit(child);
    }
  };
  visit(decls[0].body);requireFact(changed>0,'mutation changed no literal');
  return source.replace(rows[0][1],JSON.stringify(module));
}
export function expectedFailure(result,mutation,generated){
  requireFact(!result.error&&result.status!==null&&result.status!==0&&!result.signal,'mutation must complete with failure');
  const value=JSON.parse(result.stdout);requireFact(value.success===false,'mutation reported success');
  const lines=generated.split('\n');
  const start=lines.findIndex(line=>line.includes(`public theorem ${mutation.owner} `));
  requireFact(start>=0,'owning theorem absent from generated source');
  let end=lines.length;for(let i=start+1;i<lines.length;i++)if(/^(@\[expose\] )?public (def|theorem|inductive|structure) /.test(lines[i])){end=i;break;}
  requireFact(value.diagnostics.some(d=>d.code==='LLV7002'&&d.primary?.path===shaFile&&d.notes?.some(n=>{
    const m=n.message.match(/generated location: \$STAGING\/lean-src\/Uorc\/Uorc\/Integrity\/Sha256\.lean:(\d+):/);
    return m&&Number(m[1])-1>=start&&Number(m[1])-1<end;
  })),'mutation failure is not attributed to its owning observation');
}
function execute(command,args,cwd,output,label,extra={}){
  const result=spawnSync(command,args,{cwd,encoding:'utf8',timeout:300000,maxBuffer:32*1024*1024,...extra});
  fs.writeFileSync(path.join(output,label+'.stdout'),result.stdout??'');fs.writeFileSync(path.join(output,label+'.stderr'),result.stderr??'');return result;
}
function succeeded(result,label){requireFact(!result.error&&result.status===0&&!result.signal,`${label} failed: ${result.stderr??''} ${result.stdout??''}`);}
function verify(root,selected,output,label){
  const result=execute('lexlean',['--project',path.join(root,'lexlean.toml'),'--diagnostic-format','json','verify',...selected],root,output,label);
  return result;
}
export function auditRun(just,harness){
  requireFact(/^vv:.*\bintegrity\b/m.test(just),'required integrity gate omitted');
  requireFact(/^integrity:\n    node --test scripts\/integrity-vectors.test.mjs\n    node scripts\/integrity-validation.mjs\n/m.test(just),'integrity command omitted or changed');
  const body=harness.slice(harness.lastIndexOf('export function run(root){')).split('\nif(process.argv')[0];
  for(const required of ['for(const mutation of mutations){','expectedFailure(failed,mutation,generated);',"succeeded(restored,'restored source');",'receipts.length===mutations.length','leanDriver(cases,','129 complete CAVP cases; 2919 partitions; protocol and chain cases passed'])requireFact(body.includes(required),`omitted ${required}`);
  requireFact(!/\b(if|switch|while|return|continue|break)\b|mutations\.(filter|slice|splice)|cases\.(filter|slice|splice)/.test(body),'conditional or omitted integrity execution');
}
export function run(root){
  const just=fs.readFileSync(path.join(root,'Justfile'),'utf8');
  auditRun(just,fs.readFileSync(script,'utf8'));
  const cases=loadCorpus(root),mutations=sourceRecords(root).filter(r=>r.type==='Mutation');
  requireFact(mutations.length===2&&new Set(mutations.map(m=>m.id)).size===2,'mutation inventory incomplete');
  const source=fs.readFileSync(path.join(root,shaFile),'utf8');
  fs.mkdirSync(path.join(root,'.prism/uorc'),{recursive:true});const output=fs.mkdtempSync(path.join(root,'.prism/uorc/integrity.'));
  const temporary=fs.mkdtempSync(path.join(os.tmpdir(),'uorc-integrity-'));
  try{
    const current=verify(root,sources,output,'source');succeeded(current,'source verification');
    const receipts=[];
    for(const mutation of mutations){
      const project=path.join(temporary,mutation.id);
      fs.cpSync(root,project,{recursive:true,filter:p=>!path.relative(root,p).split(path.sep).some(p=>['.git','.lexlean','.prism','.lake','target','node_modules'].includes(p))});
      const mutant=plant(source,mutation);fs.writeFileSync(path.join(project,shaFile),mutant);
      const failed=verify(project,[shaFile],output,mutation.id);
      const builds=fs.readdirSync(path.join(project,'.lexlean/build'));requireFact(builds.length===1,'ambiguous mutant build');
      const generated=fs.readFileSync(path.join(project,'.lexlean/build',builds[0],'modules/Uorc/Uorc/Integrity/Sha256.lean'),'utf8');
      expectedFailure(failed,mutation,generated);
      fs.writeFileSync(path.join(project,shaFile),source);
      for(const dir of ['.lexlean','.lake'])fs.rmSync(path.join(project,dir),{recursive:true,force:true});
      const restored=verify(project,[shaFile],output,mutation.id+'-restored');succeeded(restored,'restored source');
      receipts.push({id:mutation.id,owner:mutation.owner,source_sha256:sha(source),mutant_sha256:sha(mutant),failed_stdout_sha256:sha(failed.stdout),restored_stdout_sha256:sha(restored.stdout)});
    }
    requireFact(receipts.length===mutations.length,'source mutations omitted');
    const built=execute('lexlean',['--project',path.join(root,'lexlean.toml'),'--diagnostic-format','json','build',...sources],root,output,'build');succeeded(built,'source build');
    const result=JSON.parse(built.stdout);requireFact(result.success&&result.artifacts.length===1,'source build artifact missing');
    const build=path.resolve(root,result.artifacts[0]);requireFact(build.startsWith(path.resolve(root)+path.sep),'build artifact escaped project');
    const modelRoot=path.join(temporary,'generated');
    for(const name of ['Sha256','BoundedChain']){
      const relative=`Uorc/Uorc/Integrity/${name}`;const target=path.join(modelRoot,relative+'.lean');fs.mkdirSync(path.dirname(target),{recursive:true});fs.copyFileSync(path.join(build,'modules',relative+'.lean'),target);
      const compiled=execute('lean',['-R',modelRoot,'-o',path.join(modelRoot,relative+'.olean'),target],root,output,'compile-'+name,{env:{...process.env,LEAN_PATH:modelRoot}});succeeded(compiled,'generated model compile');
    }
    const driver=path.join(temporary,'Oracle.lean');fs.writeFileSync(driver,leanDriver(cases,sourceRecords(root).filter(r=>r.type==='ChainFixture')));
    const checked=execute('lean',['-R',temporary,driver],root,output,'vectors',{env:{...process.env,LEAN_PATH:modelRoot}});succeeded(checked,'generated-model vectors');
    requireFact(checked.stdout.trim()==='129 complete CAVP cases; 2919 partitions; protocol and chain cases passed','vector output omitted expected coverage');
    const manifest=JSON.parse(fs.readFileSync(path.join(build,'manifest.json'),'utf8'));
    fs.writeFileSync(path.join(output,'receipt.json'),JSON.stringify({schema:'uorc/integrity-evidence/1',source_sha256:Object.fromEntries(sources.map(p=>[p,sha(fs.readFileSync(path.join(root,p)))])),build_manifest_sha256:sha(fs.readFileSync(path.join(build,'manifest.json'))),build_id:manifest.build_id??null,cases:cases.length,partitions:2919,vector_stdout_sha256:sha(checked.stdout),mutations:receipts,scope:'source verification and generated Lean execution; full just vv and certified production export remain separate acceptance requirements'})+'\n');
    console.log(checked.stdout.trim());console.log(`${receipts.length} source defects rejected; restored controls passed.`);
  }finally{fs.rmSync(temporary,{recursive:true,force:true});}
}
if(process.argv[1]&&path.resolve(process.argv[1])===script){try{requireFact(process.argv.length===2,'no arguments accepted');run(process.cwd());}catch(error){console.error(error.message);process.exitCode=1;}}
