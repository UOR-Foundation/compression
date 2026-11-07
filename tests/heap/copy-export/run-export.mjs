// Diagnostic consumer orchestration, adapted from tests/heap/run-export.mjs.
// Uses an existing compiler; never builds/downloads one or modifies the source.
import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import {spawnSync} from 'node:child_process';
const [compiler,fixtureArgument,lane]=process.argv.slice(2);
if(!compiler||!path.isAbsolute(compiler)||!['original','new'].includes(lane))
 throw Error('usage: node tools/run-export.mjs /absolute/existing/lexlean FIXTURE original|new');
const fixture=path.resolve(fixtureArgument);
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
const manifestPath=path.join(fixture,'fixture-manifest.json');
const manifest=JSON.parse(await fs.readFile(manifestPath,'utf8'));
if(manifest.schema!=='uorc/runtime-image-copy-production-fixture/1'||manifest.sources.length!==4||
   manifest.production_roots.length!==5||manifest.source_commit!=='bd8c62063bd568c7f513042475e33a36bdccdeda')
 throw Error('unexpected exact Copy fixture');
for(const f of manifest.files) {
 if(path.isAbsolute(f.path)||f.path.split('/').includes('..'))throw Error('unsafe fixture path');
 if(hash(await fs.readFile(path.join(fixture,f.path)))!==f.sha256)throw Error('fixture hash mismatch: '+f.path);
}
async function sourceFiles(root) {
 const out=[];
 for(const f of await fs.readdir(root,{withFileTypes:true})) {
  if(f.isSymbolicLink())throw Error('unexpected source symlink');
  const full=path.join(root,f.name);
  if(f.isDirectory())out.push(...await sourceFiles(full));
  else out.push(path.relative(fixture,full).split(path.sep).join('/'));
 }
 return out.sort();
}
if(JSON.stringify(await sourceFiles(path.join(fixture,'src')))!==JSON.stringify(manifest.sources.map(s=>s.path).sort()))
 throw Error('missing or extra source file');
await fs.mkdir(path.join(fixture,'consumer-results'),{recursive:true});
const output=path.join(fixture,'consumer-results',lane);
await fs.mkdir(output); // Refuse any existing lane, preserving all previous outcomes.
const work=path.join(output,'project');
await fs.mkdir(work);
for(const relative of [...manifest.sources.map(s=>s.path),'lexlean.toml','lakefile.toml','lake-manifest.json','lean-toolchain']) {
 const destination=path.join(work,relative);
 await fs.mkdir(path.dirname(destination),{recursive:true});
 await fs.copyFile(path.join(fixture,relative),destination);
}
const commands=[];
let phase='compiler-identity';
let compilerSha;
try {
 compilerSha=hash(await fs.readFile(compiler));
 await fs.writeFile(path.join(output,'compiler-sha256.txt'),compilerSha+'\n');
} catch(error) {
 await fs.writeFile(path.join(output,'receipt.json'),JSON.stringify({
  schema:'uorc/runtime-image-copy-production-consumer-result/1',lane,
  source_commit:manifest.source_commit,failed_phase:'compiler-identity',
  error:String(error),accepted_export_and_replay:false
 },null,2)+'\n');
 throw error;
}
async function run(name,binary,args,json=true) {
 phase=name;
 const result=spawnSync(binary,args,{cwd:work,encoding:'utf8',maxBuffer:64*1024*1024});
 await fs.writeFile(path.join(output,name+'.stdout'),result.stdout??'');
 await fs.writeFile(path.join(output,name+'.stderr'),result.stderr??'');
 commands.push({name,command:[binary,...args],exit_code:result.status,signal:result.signal??null,error:result.error?.message??null});
 await fs.writeFile(path.join(output,'commands.json'),JSON.stringify(commands,null,2)+'\n');
 if(result.error||result.status!==0)throw Error(name+' failed; exact outcome retained');
 if(!json)return result.stdout;
 const parsed=JSON.parse(result.stdout);
 if(Object.hasOwn(parsed,'success')&&!parsed.success)throw Error(name+' reported failure');
 return parsed;
}
const base={schema:'uorc/runtime-image-copy-production-consumer-result/1',lane,
 source_commit:manifest.source_commit,source_count:4,production_roots:manifest.production_roots,
 compiler_executable_sha256:compilerSha,
 compiler_source_commit_binding:'Executor provenance required; lane label/version alone does not attest a Git commit.',
 pinned_sdk_release_acceptance_claimed:false,whole_product_acceptance_claimed:false};
try {
 const version=await run('compiler-version',compiler,['--version'],false);
 const config=path.join(work,'lexlean.toml');
 await run('lock',compiler,['--project',config,'lock','--diagnostic-format','json']);
 await fs.copyFile(path.join(work,'lexlean.lock'),path.join(output,'generated-lexlean.lock'));
 await run('format',compiler,['--project',config,'fmt','--check','--all','--diagnostic-format','json']);
 await run('check',compiler,['--project',config,'check','--all','--diagnostic-format','json']);
 const targets=await run('targets',compiler,['production','targets','--diagnostic-format','json']);
 const targetsPath=path.join(output,'targets.json'),requestPath=path.join(output,'request.json');
 await fs.writeFile(targetsPath,JSON.stringify(targets)+'\n');
 const requestText=await run('request',process.execPath,[path.join(fixture,'tools/request.mjs'),targetsPath,manifestPath],false);
 const request=JSON.parse(requestText);
 if(JSON.stringify(request.roots.map(r=>r.root))!==JSON.stringify(manifest.production_roots))throw Error('root inventory differs');
 await fs.writeFile(requestPath,requestText);
 const built=await run('production-build',compiler,['--project',config,'production','build','--request',requestPath,'--diagnostic-format','json']);
 if(built.artifacts?.length!==1)throw Error('expected one production bundle');
 const bundle=path.resolve(work,built.artifacts[0]);
 const replay=await run('artifact-replay',compiler,['production','verify-artifact',bundle,'--diagnostic-format','json']);
 const receipt={...base,compiler_version:version,bundle,export_result:built,replay_result:replay,
  generated_lock_sha256:hash(await fs.readFile(path.join(output,'generated-lexlean.lock'))),
  accepted_export_and_replay:true};
 await fs.writeFile(path.join(output,'receipt.json'),JSON.stringify(receipt,null,2)+'\n');
 console.log(JSON.stringify(receipt));
} catch(error) {
 const receipt={...base,failed_phase:phase,error:String(error),accepted_export_and_replay:false};
 await fs.writeFile(path.join(output,'receipt.json'),JSON.stringify(receipt,null,2)+'\n');
 console.error(JSON.stringify(receipt));
 process.exitCode=1;
}
