// Diagnostic consumer only; uses an existing official compiler, never builds one.
import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import {spawnSync} from 'node:child_process';
const [compiler, fixtureArgument]=process.argv.slice(2);
if(!compiler || !path.isAbsolute(compiler)) throw Error('pass the absolute existing official compiler path');
const fixture=path.resolve(fixtureArgument??process.cwd());
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
const manifest=JSON.parse(await fs.readFile(path.join(fixture,'fixture-manifest.json'),'utf8'));
if(manifest.schema!=='uorc/e87-production-consumer-fixture/1' || manifest.sources.length!==14)
  throw Error('unexpected pinned fixture manifest');
for(const file of manifest.files) {
  if(path.isAbsolute(file.path) || file.path.split('/').includes('..')) throw Error('unsafe fixture path');
  if(hash(await fs.readFile(path.join(fixture,file.path)))!==file.sha256) throw Error('fixture hash mismatch: '+file.path);
}
async function sourceFiles(root) {
  const out=[];
  for(const f of await fs.readdir(root,{withFileTypes:true})) {
    if(f.isSymbolicLink()) throw Error('unexpected source symlink');
    const full=path.join(root,f.name);
    if(f.isDirectory())out.push(...await sourceFiles(full));
    else out.push(path.relative(fixture,full).split(path.sep).join('/'));
  }
  return out.sort();
}
if(JSON.stringify(await sourceFiles(path.join(fixture,'src')))!==JSON.stringify(manifest.sources.map(s=>s.path).sort()))
  throw Error('missing or extra source file');
const output=path.join(fixture,'consumer-results');
await fs.mkdir(output,{recursive:true});
if((await fs.readdir(output)).length) throw Error('consumer-results must be empty; preserve previous outcomes');
const commands=[];
async function run(name,binary,args,json=true) {
  const result=spawnSync(binary,args,{cwd:fixture,encoding:'utf8',maxBuffer:64*1024*1024});
  await fs.writeFile(path.join(output,name+'.stdout'),result.stdout??'');
  await fs.writeFile(path.join(output,name+'.stderr'),result.stderr??'');
  commands.push({name,command:[binary,...args],exit_code:result.status,error:result.error?.message??null});
  await fs.writeFile(path.join(output,'commands.json'),JSON.stringify(commands,null,2)+'\n');
  if(result.error || result.status!==0) throw Error(name+' failed; exact stdout/stderr/exit retained in consumer-results');
  if(!json)return result.stdout;
  const parsed=JSON.parse(result.stdout);
  if(Object.hasOwn(parsed,'success') && !parsed.success)throw Error(name+' reported failure');
  return parsed;
}
await fs.writeFile(path.join(output,'compiler-sha256.txt'),hash(await fs.readFile(compiler))+'\n');
await run('compiler-version',compiler,['--version'],false);
const config=path.join(fixture,'lexlean.toml');
await run('lock',compiler,['--project',config,'lock','--diagnostic-format','json']);
await run('format',compiler,['--project',config,'fmt','--check','--all','--diagnostic-format','json']);
await run('check',compiler,['--project',config,'check','--all','--diagnostic-format','json']);
const targets=await run('targets',compiler,['production','targets','--diagnostic-format','json']);
const targetsPath=path.join(output,'targets.json'),requestPath=path.join(output,'request.json');
await fs.writeFile(targetsPath,JSON.stringify(targets)+'\n');
const requestText=await run('request',process.execPath,[path.join(fixture,'tools/request.mjs'),targetsPath],false);
const request=JSON.parse(requestText);
if(JSON.stringify(request.roots.map(r=>r.root))!==JSON.stringify(manifest.production_roots))
  throw Error('request does not contain the exact sorted 21-root inventory');
await fs.writeFile(requestPath,requestText);
const built=await run('production-build',compiler,['--project',config,'production','build','--request',requestPath,'--diagnostic-format','json']);
if(built.artifacts?.length!==1)throw Error('expected one production bundle');
const bundle=path.resolve(fixture,built.artifacts[0]);
const replay=await run('artifact-replay',compiler,['production','verify-artifact',bundle,'--diagnostic-format','json']);
const receipt={schema:'uorc/e87-production-consumer-result/1',source_commit:manifest.source_commit,
  requested_compiler_commit:manifest.requested_compiler_commit,source_count:14,
  production_roots:manifest.production_roots,bundle,export_result:built,replay_result:replay,
  accepted_export_and_replay:true,pinned_sdk_release_acceptance_claimed:false};
await fs.writeFile(path.join(output,'receipt.json'),JSON.stringify(receipt,null,2)+'\n');
console.log(JSON.stringify(receipt));
