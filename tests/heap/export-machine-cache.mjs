// Read-only transfer of exact public machine sources through the official reader.
import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const [helper,prismRoot,sourceRoot,output]=process.argv.slice(2);
if(![helper,prismRoot,sourceRoot,output].every(p=>p&&path.isAbsolute(p)))
  throw Error('pass absolute reader, official PrismPM, materialized source root and output');
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
function run(binary,args,options={}) {
  const r=spawnSync(binary,args,{encoding:'utf8',maxBuffer:64*1024*1024,...options});
  if(r.error||r.status!==0)throw Error(`${path.basename(binary)} ${args[0]}: ${r.error??r.stderr}`);
  return r.stdout;
}
const git=(...args)=>run('git',['-C',prismRoot,...args]).trim();
if(git('status','--porcelain'))throw Error('official reader checkout must be clean');
const manifest=JSON.parse(await fs.readFile(path.join(sourceRoot,'source-manifest.json'),'utf8'));
if(!/^[0-9a-f]{40}$/.test(manifest.commit))throw Error('missing immutable source commit');
const modules=[];
for(const file of manifest.files) {
  if(path.isAbsolute(file.path)||file.path.split('/').includes('..'))throw Error('unsafe source path');
  const bytes=await fs.readFile(path.join(sourceRoot,file.path));
  if(bytes.length!==file.bytes||sha(bytes)!==file.sha256)throw Error('source SHA256 mismatch');
  const blob=crypto.createHash('sha1').update(`blob ${bytes.length}\0`).update(bytes).digest('hex');
  if(blob!==file.git_blob_sha1)throw Error('source does not match exact public Git blob');
  const source=bytes.toString('utf8');
  if(!Buffer.from(source,'utf8').equals(bytes))throw Error('source is not exact UTF8');
  const json=run(helper,['read'],{input:source});
  if(json!==run(helper,['read'],{input:source}))throw Error('repeated official read differs');
  const parsed=JSON.parse(json);
  if(parsed.spec!=='lexlean/semantic-module/2')throw Error('unexpected official schema');
  const canonical=run(helper,['write'],{input:JSON.stringify({source,value:parsed})});
  const canonicalRead=run(helper,['read'],{input:canonical});
  const module=source.match(/\\begin\{lexlean\}\{([^}]+)\}/)?.[1];
  if(!module)throw Error('module identity missing');
  modules.push({module,source_path:file.path,source_sha256:file.sha256,
    git_blob_sha1:file.git_blob_sha1,source_url:file.url,
    imports:[...source.matchAll(/\\importmodule\{([^}]+)\}/g)].map(m=>m[1]),
    parser_output_sha256:sha(json),repeat_read:'byte_exact',
    notation_round_trip:canonical===source?'byte_exact':'text_differs_original_preserved',
    canonical_source_sha256:sha(canonical),canonical_read_matches_original:canonicalRead===json,
    definitions:parsed.declarations.filter(d=>d.kind==='definition').map(d=>({name:d.name,
      executable_explicit:Object.hasOwn(d,'executable'),executable:d.executable??null,
      axioms_explicit:Object.hasOwn(d,'axioms'),axioms:d.axioms??null,
      ...(Object.hasOwn(d,'production')?{production:d.production}:{})})),parsed_module:parsed});
}
const helperSource='crates/conformance/src/bin/prismpm-semantic-source.rs';
const bundle={schema:'uorc/compiler-parsed-runtime-source/1',
  purpose:'read-only exact public machine IR transfer; source remains authoritative',
  source_repository:manifest.repository,source_commit:manifest.commit,
  official_reader:{repository:'https://github.com/UOR-Foundation/PrismPM',commit:git('rev-parse','HEAD'),
    helper_source:helperSource,helper_source_sha256:sha(await fs.readFile(path.join(prismRoot,helperSource))),
    helper_binary_sha256:sha(await fs.readFile(helper)),
    cargo_lock_sha256:sha(await fs.readFile(path.join(prismRoot,'Cargo.lock'))),
    vendored_lexlean_crate_sha256:sha(await fs.readFile(path.join(prismRoot,'vendor/lexlean/lexlean-0.4.0.crate'))),
    read_command:'prismpm-semantic-source read < SOURCE',
    api:'lexlean::sdk::read_module',round_trip_api:'lexlean::sdk::write_module'},
  export_script:{path:'tests/heap/export-machine-cache.mjs',sha256:sha(await fs.readFile(fileURLToPath(import.meta.url)))},
  modules,qualification:{semantic_cache_only:true,closure_linking_performed:false,
    production_export_accepted:false,pinned_sdk_accepted:false,full_release_accepted:false}};
const bytes=JSON.stringify(bundle)+'\n';
await fs.mkdir(path.dirname(output),{recursive:true});
await fs.writeFile(output,bytes);
console.log(JSON.stringify({path:output,source_commit:manifest.commit,modules:modules.length,
  bytes:Buffer.byteLength(bytes),sha256:sha(bytes)}));
