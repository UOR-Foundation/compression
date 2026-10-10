// Process/evidence orchestration only. All test behavior is authored in LexLean.
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync, mkdirSync, existsSync, copyFileSync, realpathSync, statSync, lstatSync, readdirSync } from 'node:fs';
import { dirname, join, resolve, relative, isAbsolute, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const encode = value => JSON.stringify(value, null, 2) + '\n';
export class CapabilityError extends Error {
  constructor(code, message) { super(message); this.code = code; }
}
function requireFact(condition, code, message) { if (!condition) throw new CapabilityError(code, message); }
export function loadContract(path) {
  const source = readFileSync(path, 'utf8');
  const matches = [...source.matchAll(/\\semanticdata\{(.*)\}\n/g)];
  requireFact(matches.length === 1, 'CAP0001', 'Capability contract has no unique semantic module');
  const module = JSON.parse(matches[0][1]);
  requireFact(module.spec === 'lexlean/semantic-module/1', 'CAP0001', 'Wrong capability contract schema');
  const rows = module.declarations.filter(d => d.kind === 'definition' && d.result?.member?.name === 'CapabilityCase').map(d => {
    requireFact(d.body?.kind === 'record', 'CAP0001', 'Capability row must be a closed modeled record');
    const row = Object.fromEntries(d.body.fields.map(f => {
      requireFact(f.value.kind === 'string', 'CAP0001', 'Capability metadata must be a modeled string');
      return [f.field, f.value.value];
    }));
    requireFact(Object.keys(row).sort().join() === ['application','fixture','id','intent','language','physical','project','root','schema','symbol'].sort().join(), 'CAP0001', 'Capability metadata field set changed');
    requireFact(/^[a-z][a-z0-9-]*$/.test(row.project)&&[row.fixture,row.application].every(file=>/^[A-Z][A-Za-z0-9]*\.lex\.tex$/.test(file)),'CAP0001','Capability project and fixture bindings must be confined identifiers');
    return row;
  });
  requireFact(rows.length > 0 && new Set(rows.map(r=>r.id)).size === rows.length, 'CAP0001', 'Capability register is empty or duplicated');
  return rows;
}
export function makeReport(contract, evidence) {
  const stage = id => evidence.observations.find(o=>o.id === id)?.status ?? 'not_run';
  const passed = id => stage(id) === 'passed';
  const capabilities = contract.map(row => {
    const project = row.project;
    const verified = passed(`${project}.app.evidence`) && passed(`${project}.app.verify`);
    const assertion = row.fixture === 'Core.lex.tex' || row.fixture === 'Store.lex.tex';
    const runtime = verified && (!assertion || passed(`${project}.falsify.${row.id}`)) ? 'passed' : verified ? 'negative_not_passed' : stage(`${project}.app.verify`);
    const languageFailure=evidence.observations.find(o=>[`${project}.init`,`${project}.lock`,`${project}.check`,`${project}.build`,`${project}.verify`].includes(o.id)&&o.status!=='passed');
    const result = { ...row, language: languageFailure?.status ?? stage(`${project}.verify`), language_failure_phase:languageFailure?.id??null, exporter: stage(`${project}.app.build`), runtime,
      physical_large_input: { status:'not_measured', boundary:row.physical },
      observation_ids:evidence.observations.filter(o=>o.id.startsWith(`${project}.`)).map(o=>o.id) };
    if (row.id === 'binary-transport') {
      result.runtime = verified ? 'restricted' : runtime;
      result.arbitrary_binary_round_trip = passed('core.binary-response-rejection') ? 'unsupported_by_text_profile' : 'binding_missing';
      result.observation_ids.push(...evidence.observations.filter(o=>o.id === 'core.binary-response-rejection').map(o=>o.id));
    }
    if (row.id === 'cli-filesystem') {
      result.language = 'not_applicable'; result.exporter = 'binding_missing'; result.runtime = 'binding_missing';
      result.observation_ids = [];
    }
    if (row.id === 'oracle') result.runtime = verified ? 'passed' : runtime;
    if (row.id === 'generated-conformance') result.runtime = verified && passed('core.reject') ? 'passed' : 'not_run';
    return result;
  });
  return { schema:'uorc/sdk-capability-report/1', accepted:capabilities.every(c=>c.runtime === 'passed'),
    evidence_kind:'finite SDK execution observations; no compression or large-input acceptance', ...evidence, capabilities };
}
// These receipt fields and process names are imported from the digest-bound
// SDK application verifier, not a second implementation of its runtime.
export function validateAcceptance(acceptance, manifest, acceptanceBytes, bundle={}) {
  const {build,verify,manifestBytes,lexleanBytes,modelBytes}=bundle;
  const sha=value=>typeof value==='string'&&/^[0-9a-f]{64}$/.test(value);
  requireFact(acceptance?.schema === 'prismpm/application-acceptance/1' && acceptance.status === 'verified', 'CAP0005', 'Missing complete SDK application acceptance');
  requireFact(acceptance.artifact_closure === 'verified' && acceptance.browser_projection === 'verified' && acceptance.core_wasm?.status === 'verified' && acceptance.hologram_oracle === 'verified' && acceptance.modeled_vectors === 3, 'CAP0005', 'SDK acceptance omitted a required runtime/oracle/vector result');
  requireFact(build?.schema==='prismpm/build-result/1'&&verify?.schema==='prismpm/verify-result/1'&&manifest?.schema==='prismpm/application-verification-manifest/1', 'CAP0005', 'SDK command result or verification manifest schema changed');
  requireFact(sha(build.build_id)&&build.build_id===verify.build_id&&build.build_id===acceptance.build_id&&build.build_id===manifest.build_id, 'CAP0005', 'SDK runtime receipt belongs to a different or malformed build');
  requireFact(sha(verify.attestation_id)&&verify.verified_root===`.prism/verified/${verify.attestation_id}`&&Buffer.isBuffer(manifestBytes)&&digest(manifestBytes)===verify.attestation_id, 'CAP0005', 'SDK command result does not bind the exact verification manifest');
  requireFact(manifest.acceptance_sha256===digest(acceptanceBytes)&&Buffer.isBuffer(lexleanBytes)&&manifest.lexlean_attestation_sha256===digest(lexleanBytes)&&Buffer.isBuffer(modelBytes)&&manifest.model_sha256===digest(modelBytes), 'CAP0005', 'SDK verification manifest does not bind the exact acceptance, kernel attestation and model bytes');
  let lexlean;try {lexlean=JSON.parse(lexleanBytes);} catch {throw new CapabilityError('CAP0005','SDK kernel attestation is not JSON');}
  requireFact(lexlean?.status==='verified'&&sha(lexlean.attestation_id)&&lexlean.attestation_id===acceptance.lexlean_attestation_id&&sha(build.source_id)&&build.source_id===acceptance.source_id&&build.source_id===lexlean.source_id&&sha(build.semantic_id)&&build.semantic_id===lexlean.semantic_id, 'CAP0005', 'SDK kernel attestation and runtime receipt belong to different sources');
  const required=['lean-version','lake-version','rustfmt-version','rustc-version','timeout-version','hologram-oracle-build','hologram-oracle','core-wasm-validate','core-wasm-inspect','application-package-test','application-package-no-std','application-consumer-lock','application-generated-rust-corpus'];
  requireFact(Array.isArray(manifest.processes)&&manifest.processes.length===required.length&&required.every(name=>manifest.processes.filter(p=>p?.tool===name).length===1), 'CAP0005', 'SDK verification manifest omits or duplicates a required application process');
  requireFact(manifest.processes.every(p=>p.exit_code===0&&sha(p.executable_sha256)&&Array.isArray(p.argv)&&p.argv.every(arg=>typeof arg==='string')&&typeof p.stdout==='string'&&typeof p.stderr==='string'), 'CAP0005', 'SDK verification manifest has failed or malformed application process evidence');
  let oracle;try {oracle=JSON.parse(manifest.processes.find(p=>p.tool==='hologram-oracle').stdout);} catch {throw new CapabilityError('CAP0005','SDK oracle process has no JSON result');}
  requireFact(oracle?.schema==='prismpm/hologram-oracle/1'&&oracle.footer_verified===true, 'CAP0005', 'SDK oracle process omitted its verified footer result');
}
export function confined(base, path) {
  requireFact(typeof path === 'string' && path.length > 0 && !isAbsolute(path), 'CAP0005', 'SDK result path is not relative');
  const target = resolve(base, path), rel = relative(base, target);
  requireFact(rel !== '..' && !rel.startsWith(`..${sep}`) && rel !== '', 'CAP0005', 'SDK result path escapes its probe project');
  let cursor = base;
  for (const part of rel.split(sep)) {
    cursor = join(cursor, part);
    requireFact(!lstatSync(cursor).isSymbolicLink(), 'CAP0005', 'SDK result contains a symbolic link');
  }
  const resolved = relative(realpathSync(base), realpathSync(target));
  requireFact(resolved !== '..' && !resolved.startsWith(`..${sep}`) && resolved !== '', 'CAP0005', 'SDK result resolves outside its probe project');
  return target;
}
export function negateRegisteredRoot(source, rootName) {
  const match = source.match(/\\semanticdata\{(.*)\}\n/);
  requireFact(match, 'CAP0001', 'Falsifier source has no semantic module');
  const module = JSON.parse(match[1]);
  const name = rootName.split('.').at(-1);
  const declarations = module.declarations.filter(d=>d.name===name && d.kind==='definition' && d.result?.kind==='bool' && d.parameters.length===0);
  requireFact(declarations.length===1, 'CAP0001', 'Falsifier must select exactly one registered zero-argument Boolean root');
  declarations[0].body = {kind:'not',value:declarations[0].body};
  return source.replace(match[1],JSON.stringify(module));
}
function readJson(path) { return JSON.parse(readFileSync(path, 'utf8')); }
function markdown(report) {
  return '<!-- uorc:non-authoritative -->\n\n# Observed SDK compatibility\n\n' +
    `Generated from the bound probe run. M0 capability acceptance: **${report.accepted ? 'passed' : 'blocked'}**.\n\n` +
    `SDK: ${report.sdk?.image ?? 'not available'}\n\n` +
    '| Capability | Language | Exporter | Runtime | Physical large-input boundary |\n| --- | --- | --- | --- | --- |\n' +
    report.capabilities.map(c=>`| ${c.id} | ${c.language} | ${c.exporter} | ${c.runtime} | ${c.physical_large_input.boundary} |`).join('\n') +
    '\n\nExact symbols, schemas, source hashes, command exits, diagnostic codes, logs and artifact bindings are in compatibility.json. A missing binding is not an executed rejection. A toolchain/dependency failure is not a language rejection. Text-profile byte requests do not establish arbitrary binary responses or generated CLI/filesystem adapters.\n';
}
export function run(outputDirectory) {
  const out = resolve(outputDirectory);
  requireFact(!existsSync(out), 'CAP0006', 'Report directory already exists; stale results cannot be reused');
  mkdirSync(out, {recursive:true});
  const contract = loadContract(join(root,'src/Uorc/CapabilityContract.lex.tex'));
  const evidence = {sdk:null, selected_upstream_sources:null, source_bindings:[], project_bindings:[], observations:[], bindings:[], artifacts:[], blockers:[]};
  for (const name of ['src/Uorc/CapabilityContract.lex.tex','src/Uorc/Registry.lex.tex','Justfile','lexlean.toml','lexlean.lock','xtask/src/capabilities.rs','scripts/capability-probes.mjs','scripts/capability-probes.test.mjs','.github/workflows/capability-probes.yml','docs/governance/m0-source-selection.json',...new Set(contract.flatMap(c=>[c.fixture,c.application]).map(file=>`probes/capabilities/${file}`)), 'probes/capabilities/BinaryResponse.lex.tex','probes/capabilities/RejectedAcceptance.lex.tex','probes/capabilities/StoreApplication.lex.tex']) {
    evidence.source_bindings.push({path:name,sha256:digest(readFileSync(join(root,name)))});
  }
  evidence.selected_upstream_sources = readJson(join(root,'docs/governance/m0-source-selection.json'));
  const bindProject = (id, project, phase) => {
    for (const name of ['lexlean.toml','lexlean.lock','lakefile.toml','lake-manifest.json','lean-toolchain','prismpm.toml']) {
      const path=join(project,name); if(!existsSync(path))continue;
      const bytes=readFileSync(path), file=`artifacts/${id}-${phase}-${name.replaceAll('/','-')}`;
      mkdirSync(join(out,'artifacts'),{recursive:true});writeFileSync(join(out,file),bytes);
      evidence.project_bindings.push({project:id,phase,input:name,path:file,sha256:digest(bytes)});
    }
  };
  const captureBuild = (project,id,path) => {
    const source=confined(project,path);
    const walk=(directory,parts=[])=>{
      for(const name of readdirSync(directory).sort()) {
        const sourcePath=join(directory,name), info=lstatSync(sourcePath), names=[...parts,name];
        requireFact(!info.isSymbolicLink(), 'CAP0005', 'Generated SDK evidence contains a symbolic link');
        if(info.isDirectory()){walk(sourcePath,names);continue;}
        requireFact(info.isFile(), 'CAP0005', 'Generated SDK evidence is not a regular file');
        const bytes=readFileSync(sourcePath), file=['artifacts',`${id}-lexlean-build`,...names].join('/');
        mkdirSync(dirname(join(out,file)),{recursive:true});writeFileSync(join(out,file),bytes);
        evidence.artifacts.push({path:file,sha256:digest(bytes)});
      }
    };walk(source);
  };
  const save = () => {const report=makeReport(contract,evidence);writeFileSync(join(out,'compatibility.json'),encode(report));writeFileSync(join(out,'compatibility.md'),markdown(report));return report;};
  let commands;
  function execute(id, command, args, cwd, {expectedExit=0, requiredDiagnostic, timeout=1200000}={}) {
    requireFact(!evidence.observations.some(o=>o.id===id), 'CAP0006', 'Duplicate observation identifier');
    const executable=commands.get(command);
    requireFact(executable, 'CAP0002', `SDK does not bind executable ${command}`);
    const result=spawnSync(executable,args,{cwd,encoding:'utf8',timeout,maxBuffer:32*1024*1024,env:{...process.env,CARGO_NET_OFFLINE:'true'}});
    const stdout=result.stdout??'',stderr=result.stderr??'';
    const codes=[...new Set((stdout+'\n'+stderr).match(/\b(?:LL[A-Z]|PP)[0-9]{4}\b/g)??[])];
    const diagnosticMatches=!requiredDiagnostic||codes.includes(requiredDiagnostic);
    const logBase=`logs/${id}`; mkdirSync(join(out,'logs'),{recursive:true});
    writeFileSync(join(out,`${logBase}.stdout`),stdout);writeFileSync(join(out,`${logBase}.stderr`),stderr);
    const success=result.status===expectedExit&&!result.error&&diagnosticMatches;
    const dependencyFailure=!!result.error||codes.some(c=>['LLV7001','PP1004','PP5001','PP5002','PP9001'].includes(c));
    const observation={id,command,executable,args:args.map(a=>a.startsWith(out)?`$RUN/${relative(out,a)}`:a),cwd:`$RUN/${relative(out,cwd)}`,status:success?'passed':dependencyFailure?'dependency_blocked':'failed',exit:result.status,signal:result.signal,diagnostic_codes:codes,expected_exit:expectedExit,...(requiredDiagnostic?{required_diagnostic:requiredDiagnostic}:{}),...(result.error?{process_error:result.error.code??String(result.error)}:{}),stdout:{path:`${logBase}.stdout`,sha256:digest(stdout)},stderr:{path:`${logBase}.stderr`,sha256:digest(stderr)}};
    evidence.observations.push(observation);save();return {success,stdout,stderr,observation};
  }
  try {
    const lock=readJson(join(root,'prismpm.lock'));
    const platform=lock.platforms.find(p=>p.platform===`linux/${process.arch==='x64'?'amd64':process.arch}`);
    requireFact(platform,'CAP0002','Current platform is absent from prismpm.lock');
    const inventory=readFileSync('/opt/prismpm/share/inventory.json');
    requireFact(inventory.toString()===platform.inventory_document && `sha256:${digest(inventory)}`===platform.inventory_digest,'CAP0002','Installed SDK inventory differs from the committed lock');
    const actual=JSON.parse(inventory);commands=new Map();
    for(const name of ['lexlean','prismpm','tar']) {
      const bindings=actual.commands.filter(c=>c.command===name);
      requireFact(bindings.length===1,'CAP0002',`No unique installed command binding for ${name}`);
      const binding=bindings[0];const real=realpathSync(binding.executable);
      requireFact(statSync(real).isFile()&&digest(readFileSync(real))===binding.sha256,'CAP0002',`Installed ${name} bytes differ from locked command digest`);
      commands.set(name,binding.executable);evidence.bindings.push({kind:'executable',...binding});
    }
    evidence.sdk={image:lock.sdk_image,platform:platform.platform,inventory_sha256:digest(inventory),lock_sha256:digest(readFileSync(join(root,'prismpm.lock')))};
    for(const file of ['schemas/model-document-v2.schema.json','SPEC.md','conformance-root/vendor/lexlean/schemas/semantic-module.schema.json','conformance-root/vendor/lexlean/schemas/semantic-module-v2.schema.json']) {
      const path=join('/opt/prismpm/share',file);if(existsSync(path))evidence.bindings.push({kind:'public-contract',path,sha256:digest(readFileSync(path))});
    }
    // Record absence honestly; do not invent or exercise an unpublished CLI/FS API.
    evidence.blockers.push({code:'CAP0004',capability:'cli-filesystem',status:'dependency_blocked',binding_status:'binding_missing',reason:'No generated product CLI/filesystem binding is selected by the modeled TextApplication profile; this probe does not claim that no other SDK API can exist.'});
    const projects=contract.filter(row=>['Core.lex.tex','Store.lex.tex'].includes(row.fixture));
    requireFact(new Set(projects.map(row=>row.project)).size===projects.length,'CAP0001','Primitive projects must be independently bound');
    for(const {project:id,language,fixture,application} of projects) {
      const project=join(out,id);
      if(!execute(`${id}.init`,'lexlean',['init',project,'--name',`uorc-probe-${id}`,'--module-prefix','UorcProbe','--language',language],out).success)continue;
      copyFileSync(join(root,'probes/capabilities',fixture),join(project,'src/Main.lex.tex'));
      const lex=['--project',join(project,'lexlean.toml'),'--diagnostic-format','json'];
      let ready=true;
      for(const action of ['lock','check','build','verify']) {
        const result=execute(`${id}.${action}`,'lexlean',[...lex,action],project);
        if(!result.success){ready=false;break;}
        if(action==='build'||action==='verify') {
          const build=JSON.parse(result.stdout);
          requireFact(Array.isArray(build.artifacts)&&build.artifacts.length===1,'CAP0005','LexLean did not identify exactly one build or verified root');
          captureBuild(project,action==='build'?id:`${id}-verified`,build.artifacts[0]);
        }
      }
      bindProject(id,project,'language');
      if(!ready)continue;
      // Public model imports come from the bound SDK archive, never copied from another checkout.
      const archive='/opt/prismpm/share/stdlib-sources.tar';
      const archiveBinding=platform.inventory.find(a=>a.id==='stdlib-sources');
      requireFact(archiveBinding&&`sha256:${digest(readFileSync(archive))}`===archiveBinding.digest,'CAP0002','SDK model archive differs from its locked digest');
      const exported=execute(`${id}.model-binding`,'tar',['-xOf',archive,'stdlib/Foundation/View/Text/V1/Model.lex.tex'],project);
      if(!exported.success)continue;
      requireFact(exported.stdout.includes('"name":"TextApplication"'),'CAP0005','SDK source does not define the selected public symbol');
      evidence.bindings.push({kind:'public-symbol',project:id,symbol:'Foundation.View.Text.V1.Model.TextApplication',archive_sha256:digest(readFileSync(archive)),member:'stdlib/Foundation/View/Text/V1/Model.lex.tex',sha256:digest(exported.stdout)});
      const modelPath=join(project,'src/Foundation/View/Text/V1/Model.lex.tex');mkdirSync(dirname(modelPath),{recursive:true});writeFileSync(modelPath,exported.stdout);
      if(!execute(`${id}.glossary-binding`,'tar',['-xf',archive,'-C',project,'language/prism.arch'],project).success)continue;
      const configPath=join(project,'lexlean.toml');let config=readFileSync(configPath,'utf8');
      requireFact(config.includes('entrypoints = ["src/Main.lex.tex"]')&&config.includes('[limits]'),'CAP0005','SDK initialized configuration shape changed');
      config=config.replace('entrypoints = ["src/Main.lex.tex"]','entrypoints = ["src/Application.lex.tex"]').replace('[limits]','[[lexicon_source]]\npackage = "prism.arch"\nkind = "path"\npath = "language/prism.arch"\n\n[limits]');writeFileSync(configPath,config);
      copyFileSync(join(root,'probes/capabilities',application),join(project,'src/Application.lex.tex'));
      writeFileSync(join(project,'prismpm.toml'),'spec = "prismpm/project/1"\nproject = "Uorc Capability Probe"\nlexlean_project = "lexlean.toml"\nbuild_root = ".prism"\n\n[limits]\nmax_holo_bytes = 16777216\nmax_entities = 100000\nmax_diagnostics = 256\n');
      if(!execute(`${id}.app.lock`,'lexlean',[...lex,'lock'],project).success)continue;
      bindProject(id,project,'application');
      const prism=['--project',project,'--json'];
      let applicationBuild;
      for(const action of ['check','build','verify']) {
        const result=execute(`${id}.app.${action}`,'prismpm',[...prism,action],project);
        if(!result.success){ready=false;break;}
        if(action==='build') {const response=JSON.parse(result.stdout);applicationBuild=response.result??response;}
        if(action==='verify') {
          const response=JSON.parse(result.stdout); const value=response.result??response;
          confined(project,value.verified_root);
          const acceptanceBytes=readFileSync(confined(project,`${value.verified_root}/application-acceptance.json`));
          const manifestBytes=readFileSync(confined(project,`${value.verified_root}/manifest.json`));
          const lexleanBytes=readFileSync(confined(project,`${value.verified_root}/lexlean-attestation.json`));
          const modelBytes=readFileSync(confined(project,applicationBuild?.model_path));
          validateAcceptance(JSON.parse(acceptanceBytes),JSON.parse(manifestBytes),acceptanceBytes,{build:applicationBuild,verify:value,manifestBytes,lexleanBytes,modelBytes});
          const modelFile=`artifacts/${id}-model.prism.json`;writeFileSync(join(out,modelFile),modelBytes);evidence.artifacts.push({path:modelFile,sha256:digest(modelBytes)});
          for(const name of ['application-acceptance.json','manifest.json','lexlean-attestation.json']) {
            const bytes=readFileSync(confined(project,`${value.verified_root}/${name}`));const file=`artifacts/${id}-${name}`;mkdirSync(join(out,'artifacts'),{recursive:true});writeFileSync(join(out,file),bytes);evidence.artifacts.push({path:file,sha256:digest(bytes)});
          }
          evidence.observations.push({id:`${id}.app.evidence`,status:'passed',schema:'prismpm/application-acceptance/1',acceptance_sha256:digest(acceptanceBytes)});save();
        }
      }
      if(ready) {
        const original=readFileSync(join(project,'src/Main.lex.tex'),'utf8');
        for(const row of contract.filter(c=>c.project===id&&c.fixture===fixture)) {
          const mutant=negateRegisteredRoot(original,row.root);
          writeFileSync(join(project,'src/Main.lex.tex'),mutant);
          const mutationPath=`artifacts/${id}-falsify-${row.id}.lex.tex`;
          writeFileSync(join(out,mutationPath),mutant);
          evidence.artifacts.push({path:mutationPath,sha256:digest(mutant),registered_root:row.root,purpose:'single-root dependency-edge falsification'});
          execute(`${id}.falsify.${row.id}`,'prismpm',[...prism,'verify'],project,{expectedExit:1,requiredDiagnostic:'PP5301'});
          writeFileSync(join(project,'src/Main.lex.tex'),original);
        }
      }
      if(id==='core') {
        // The same public TextApplication contract rejects invalid UTF-8 responses.
        copyFileSync(join(root,'probes/capabilities/BinaryResponse.lex.tex'),join(project,'src/Application.lex.tex'));
        execute('core.binary-response-rejection','prismpm',[...prism,'check'],project,{expectedExit:1,requiredDiagnostic:'PP2009'});
        if(ready) {
          copyFileSync(join(root,'probes/capabilities/RejectedAcceptance.lex.tex'),join(project,'src/Application.lex.tex'));
          execute('core.reject','prismpm',[...prism,'verify'],project,{expectedExit:1,requiredDiagnostic:'PP5301'});
        }
        copyFileSync(join(root,'probes/capabilities/Application.lex.tex'),join(project,'src/Application.lex.tex'));
      }
    }
  } catch(error) {
    evidence.blockers.push({code:error instanceof CapabilityError ? error.code : 'CAP0003',cause_code:error.code??null,status:'dependency_blocked',reason:error.message});
  }
  return save();
}
if (process.argv[1] && resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  try {
    requireFact(process.argv.length===3,'CAP0001','usage: node scripts/capability-probes.mjs ABSENT_REPORT_DIRECTORY');
    const report=run(process.argv[2]); console.log(`capability report: ${resolve(process.argv[2],'compatibility.json')}`);process.exitCode=report.accepted?0:1;
  } catch(error) { console.error(`${error.code??'CAP0003'}: ${error.message}`);process.exitCode=1; }
}
