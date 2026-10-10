// SDK orchestration; mutation definitions and owners are source-owned records.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {copySource,sha,canonical} from './offline-validation.mjs';
const script=fileURLToPath(import.meta.url);
const sourcePath='src/Uorc/Varint.lex.tex';
function requireFact(value,message){if(!value)throw new Error(`UC-VAR-01: ${message}`);}
export function auditGate(just,harness){
  requireFact(just.split('\n').filter(line=>/^vv:/.test(line)).length===1&&/^vv:.*\bvarint-proofs\b/m.test(just),'required varint gate omitted');
  requireFact(just.split('\n').filter(line=>/^varint-proofs:/.test(line)).length===1&&/^varint-proofs:\n    node scripts\/varint-validation\.mjs\n(?:\n|$)/m.test(just),'varint command altered or conditional');
  const body=harness.slice(harness.lastIndexOf('export function run(root)'));
  for(const required of ['for(const mutation of cases)', 'expectedFailure(result,mutation);', "verified.modules.includes('Uorc.Varint')", 'receipts.length===cases.length'])requireFact(body.includes(required),`omitted ${required}`);
  requireFact(!/\b(continue|break)\b|cases\.(filter|slice|splice)|process\.env/.test(body),'filtered or conditional mutation execution');
}
export function parse(source){const rows=[...source.matchAll(/\\coredata\{(.*)\}\n/g)];requireFact(rows.length===1,'expected one core module');return JSON.parse(rows[0][1]);}
export function plant(source,mutation){
  const core=parse(source);
  const rows=core.declarations.filter(d=>d.name===mutation.definition);
  requireFact(rows.length===1 && ['definition','abbrev'].includes(rows[0].kind),'mutation has no unique computational definition');
  const cache=new Map();let changes=0;
  const clone=i=>{
    if(cache.has(i))return cache.get(i);
    const node={...core.nodes[i]};
    for(const key of ({a:['f','x'],l:['t','v'],p:['t','v'],e:['t','v','body'],j:['x']}[node.k]??[]))node[key]=clone(node[key]);
    if(node.k==='n' && node.v===mutation.before){node.v=mutation.after;changes++;}
    const index=core.nodes.length;core.nodes.push(node);cache.set(i,index);return index;
  };
  rows[0].value=clone(rows[0].value);
  requireFact(changes>0,'mutation changed no intended literal');
  return source.replace(source.match(/\\coredata\{(.*)\}\n/)[1],canonical(core));
}
export function mutations(root){
  const source=fs.readFileSync(path.join(root,'src/Uorc/VarintContract.lex.tex'),'utf8');
  const module=JSON.parse(source.match(/\\semanticdata\{(.*)\}\n/)[1]);
  const rows=module.declarations.filter(d=>d.result?.member?.name==='CoreMutation').map(d=>Object.fromEntries(d.body.fields.map(f=>[f.field,f.value.value])));
  requireFact(rows.length>0&&new Set(rows.map(r=>r.id)).size===rows.length,'empty/duplicate mutation inventory');
  for(const row of rows){
    requireFact(Object.keys(row).sort().join()===['id','definition','before','after','owner'].sort().join(),'mutation fields differ');
    requireFact(/^[a-z][a-z0-9_]*$/.test(row.id)&&[row.before,row.after].every(v=>/^(0|[1-9][0-9]*)$/.test(v))&&[row.definition,row.owner].every(v=>/^Uorc\.Varint\.[A-Za-z][A-Za-z0-9_.]*$/.test(v)),'invalid mutation binding');
  }
  return rows;
}
export function expectedFailure(result,mutation){
  requireFact(!result.error&&result.status!==null&&result.status!==0&&!result.signal,'mutation did not complete with failure');
  const value=JSON.parse(result.stdout);
  requireFact(value.spec==='lexlean/command-result/1'&&value.success===false,'not an SDK failure');
  requireFact((value.diagnostics??[]).some(d=>d.code==='LLV7002'&&d.primary?.path===sourcePath&&d.message.includes(`native core declaration '${mutation.owner}':`) && /\(kernel\) (application|declaration) type mismatch/.test(d.message)),'failure is not attributed to the owning kernel declaration');
}
export function run(root){
  auditGate(fs.readFileSync(path.join(root,'Justfile'),'utf8'),fs.readFileSync(script,'utf8'));
  const cases=mutations(root),source=fs.readFileSync(path.join(root,sourcePath),'utf8');
  const temporary=fs.mkdtempSync(path.join(os.tmpdir(),'uorc-varint-mutations-'));
  fs.mkdirSync(path.join(root,'.prism/uorc'),{recursive:true});
  const output=fs.mkdtempSync(path.join(root,'.prism/uorc/varint.'));
  const receipts=[];
  const execute=(project,label)=>{
    const result=spawnSync('/usr/local/bin/lexlean',['--project',path.join(project,'lexlean.toml'),'--diagnostic-format','json','verify','--all'],{cwd:project,encoding:'utf8',timeout:300000,maxBuffer:32*1024*1024});
    fs.writeFileSync(path.join(output,`${label}.stdout`),result.stdout??'');fs.writeFileSync(path.join(output,`${label}.stderr`),result.stderr??'');return result;
  };
  try{
    for(const mutation of cases){
      const project=path.join(temporary,mutation.id);copySource(root,project);
      const mutant=plant(source,mutation);fs.writeFileSync(path.join(project,sourcePath),mutant);
      const result=execute(project,mutation.id);expectedFailure(result,mutation);
      fs.writeFileSync(path.join(project,sourcePath),source);
      for(const dir of ['.lexlean','.lake'])fs.rmSync(path.join(project,dir),{recursive:true,force:true});
      const restored=execute(project,`${mutation.id}-restored`);
      requireFact(!restored.error&&restored.status===0&&!restored.signal,'restored control failed');
      const verified=JSON.parse(restored.stdout);
      requireFact(verified.success===true&&verified.exit_code===0&&verified.modules.includes('Uorc.Varint'),'restored control omitted the codec');
      receipts.push({...mutation,source_sha256:sha(source),mutant_sha256:sha(mutant),exit_code:result.status,stdout_sha256:sha(result.stdout),restored_stdout_sha256:sha(restored.stdout),restored_exit_code:restored.status,restored_build_id:verified.build_id,restored_attestation_id:verified.attestation_id});
    }
    requireFact(receipts.length===cases.length,'source cases omitted');
    fs.writeFileSync(path.join(output,'mutations.json'),canonical({schema:'uorc/varint-mutations/1',cases:receipts})+'\n');
    console.log(`${receipts.length} varint defects rejected by their owning kernel declarations; every restored control passed.`);
  }finally{fs.rmSync(temporary,{recursive:true,force:true});}
}
if(process.argv[1]&&path.resolve(process.argv[1])===script){try{requireFact(process.argv.length===2,'no harness arguments accepted');run(process.cwd());}catch(error){console.error(error.message);process.exitCode=1;}}
