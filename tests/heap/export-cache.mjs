// Read-only diagnostic cache. All semantic parsing/writing belongs to PrismPM.
import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const repo=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
const [helper, prismRoot, output]=process.argv.slice(2);
if (![helper,prismRoot,output].every(x=>x && path.isAbsolute(x)))
  throw Error('pass absolute official helper, official PrismPM checkout, and output paths');
const sha=bytes=>crypto.createHash('sha256').update(bytes).digest('hex');
function run(binary,args,options={}) {
  const result=spawnSync(binary,args,{encoding:'utf8',maxBuffer:64*1024*1024,...options});
  if(result.error || result.status!==0) throw Error(`${path.basename(binary)} ${args[0]}: ${result.error??result.stderr}`);
  return result.stdout;
}
const git=(root,...args)=>run('git',['-C',root,...args]).trim();
const sourceCommit=git(repo,'rev-parse','HEAD');
const prismCommit=git(prismRoot,'rev-parse','HEAD');
if(git(prismRoot,'status','--porcelain')) throw Error('official PrismPM checkout must be clean');
const helperRelative='crates/conformance/src/bin/prismpm-semantic-source.rs';
const names=['RuntimeValue','Heap','HeapAudit','RuntimeTable','RuntimeSequence','RuntimeStore',
  'RuntimeByteBuilder','HeapTests','RuntimeTableTests','RuntimeIndexTests',
  'RuntimeTableCursorTests','RuntimeSequenceTests','RuntimeStoreTests','RuntimeByteBuilderTests',
  'RuntimeOwnershipProofs','RuntimeAuditProofs','RuntimeObservationProofs','RuntimeSequenceProofs',
  'RuntimeCost','RuntimeSequenceTickets','RuntimeSequenceTicketTests','HeapMovedReference','HeapMovedTests',
  'RuntimePeak','RuntimePeakProofs','RuntimePeakTests','RuntimeSequenceConcatReference',
  'RuntimeImage','RuntimeIndexProofs','RuntimeImageProofs','RuntimeImageTests',
  'RuntimeIndexFrameProofs','RuntimeTableCancelTests','RuntimeByteBuilderFactorReference'];
const modules=[];
for(const name of names) {
  const sourcePath=`src/Uorc/Core/${name}.lex.tex`;
  const source=await fs.readFile(path.join(repo,sourcePath),'utf8');
  const committed=run('git',['-C',repo,'show',`${sourceCommit}:${sourcePath}`]);
  if(source!==committed) throw Error(`${sourcePath}: commit does not match source bytes`);
  const json=run(helper,['read'],{input:source});
  if(json!==run(helper,['read'],{input:source})) throw Error(`${name}: repeat read differs`);
  const parsed=JSON.parse(json);
  if(parsed.spec!=='lexlean/semantic-module/2') throw Error(`${name}: unexpected official schema`);
  const restored=run(helper,['write'],{input:JSON.stringify({source,value:parsed})});
  if(restored!==source) throw Error(`${name}: official notation round trip differs`);
  const definitions=parsed.declarations.filter(x=>x.kind==='definition').map(d=>{
    const executableExplicit=Object.hasOwn(d,'executable');
    if((!executableExplicit && !name.endsWith('Proofs')) || !Object.hasOwn(d,'axioms'))
      throw Error(`${name}.${d.name}: missing explicit executable/axiom policy`);
    return {name:d.name,executable:executableExplicit?d.executable:null,
      executable_explicit:executableExplicit,axioms:d.axioms,
      ...(Object.hasOwn(d,'production')?{production:d.production}:{})};
  });
  if(source!==await fs.readFile(path.join(repo,sourcePath),'utf8')) throw Error(`${name}: source changed during read`);
  modules.push({module:`Uorc.Core.${name}`,source_path:sourcePath,source_sha256:sha(source),
    role:name.endsWith('Tests')?'diagnostic_scenarios':name.endsWith('Proofs')?'proof_only':
      name.endsWith('Reference')?'diagnostic_reference':name==='HeapAudit'?'independent_audit':'runtime',
    imports:[...source.matchAll(/\\importmodule\{([^}]+)\}/g)].map(x=>x[1]),
    parser_output_sha256:sha(json),repeat_read:'byte_exact',notation_round_trip:'byte_exact',
    definitions,parsed_module:parsed});
}
const bundle={schema:'uorc/compiler-parsed-runtime-source/1',
  purpose:'read-only diagnostic cache; authoritative source is the byte-bound .lex.tex, never this cache',
  source_commit:sourceCommit,
  official_reader:{repository:'https://github.com/UOR-Foundation/PrismPM',commit:prismCommit,
    helper_source:helperRelative,helper_source_sha256:sha(await fs.readFile(path.join(prismRoot,helperRelative))),
    helper_binary_sha256:sha(await fs.readFile(helper)),
    cargo_lock_sha256:sha(await fs.readFile(path.join(prismRoot,'Cargo.lock'))),
    vendored_lexlean_crate_sha256:sha(await fs.readFile(path.join(prismRoot,'vendor/lexlean/lexlean-0.4.0.crate'))),
    build_command:'cargo build --offline --locked -p repo-conformance --bin prismpm-semantic-source',
    read_command:'prismpm-semantic-source read < SOURCE',
    round_trip_command:'prismpm-semantic-source write < {"source":SOURCE_TEXT,"value":PARSED_MODULE}',
    api:'lexlean::sdk::read_module',round_trip_api:'lexlean::sdk::write_module'},
  export_script:{path:'tests/heap/export-cache.mjs',sha256:sha(await fs.readFile(fileURLToPath(import.meta.url)))},
  modules,
  qualification:{semantic_cache_only:true,production_export_accepted:false,
    pinned_sdk_accepted:false,full_release_accepted:false,
    retained_export_failure:'LLV7015 on the pre-fix compiler; candidate #393 consumer qualification is separate',
    candidate_export_status:'not attempted in this executor; wire executor owns the existing 3fe13b50 compiler',
    physical_memory_measurement:false}};
await fs.mkdir(path.dirname(output),{recursive:true});
const bytes=JSON.stringify(bundle)+'\n';
await fs.writeFile(output,bytes);
console.log(JSON.stringify({path:output,source_commit:sourceCommit,modules:modules.length,bytes:Buffer.byteLength(bytes),sha256:sha(bytes)}));
