import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { test } from 'node:test';
import { readFileSync, mkdtempSync, mkdirSync, writeFileSync, symlinkSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { loadContract, makeReport, validateAcceptance, CapabilityError, confined, negateRegisteredRoot, classifyResult } from './capability-probes.mjs';

const contract = loadContract(new URL('../src/Uorc/CapabilityContract.lex.tex', import.meta.url));
test('conformance_uc_sdk_01', () => {
  assert.equal(contract.length, 10);
  const report = makeReport(contract, { sdk: null, observations: [], bindings: [], artifacts: [] });
  assert.equal(report.accepted, false);
  assert(report.capabilities.every(row => row.language !== 'passed' && row.runtime !== 'passed'));
  assert(report.capabilities.every(row => row.physical_large_input.status === 'not_measured'));
});
test('conformance_uc_sdk_02', () => {
  const evidence = { sdk: 'test reporting only', observations: [{ id: 'core.check', status: 'passed' }], bindings: [], artifacts: [] };
  const report = makeReport(contract, evidence);
  assert.equal(report.accepted, false);
  assert.equal(report.capabilities.find(row => row.id === 'bytes').runtime, 'not_run');
  assert.equal(report.capabilities.find(row => row.id === 'binary-transport').runtime, 'not_run');
  assert.equal(report.capabilities.find(row => row.id === 'cli-filesystem').runtime, 'binding_missing');
});
test('zero exit without complete SDK acceptance is not runtime evidence', () => {
  assert.throws(() => validateAcceptance({}, {}, ''), CapabilityError);
  assert.throws(() => validateAcceptance({ status: 'verified', schema: 'prismpm/application-acceptance/1' }, {}, ''), CapabilityError);
});
test('report cannot promote text response verification to arbitrary binary transport', () => {
  const observations = ['core.check','core.verify','core.app.build','core.app.verify','core.app.evidence','core.reject'].map(id => ({id,status:'passed'}));
  const report = makeReport(contract, {sdk:'test reporting only',observations,bindings:[],artifacts:[]});
  const binary = report.capabilities.find(row => row.id === 'binary-transport');
  assert.equal(binary.runtime, 'restricted');
  assert.equal(report.accepted, false);
  assert.equal(binary.arbitrary_binary_round_trip, 'binding_missing');
});
test('all probe semantic payloads are canonical authored JSON', () => {
  for (const file of [...new Set(contract.flatMap(row=>[row.fixture,row.application]).map(file=>file.replace('.lex.tex',''))),'BinaryResponse','RejectedAcceptance']) {
    const text = readFileSync(new URL(`../probes/capabilities/${file}.lex.tex`, import.meta.url), 'utf8');
    const payload = text.match(/\\semanticdata\{(.*)\}\n/)[1];
    assert.equal(JSON.stringify(JSON.parse(payload)),payload);
    assert(JSON.parse(payload).declarations.length > 0);
  }
});

test('SDK artifact confinement rejects exact parent and symbolic-link escapes', () => {
  const temporary=mkdtempSync(join(tmpdir(),'uorc-confinement-'));
  try {
    const project=join(temporary,'project');mkdirSync(project);writeFileSync(join(project,'safe'),'ok');
    symlinkSync(temporary,join(project,'escape'));
    assert.throws(()=>confined(project,'..'),CapabilityError);
    assert.throws(()=>confined(project,'../project'),CapabilityError);
    assert.throws(()=>confined(project,'escape/project/safe'),CapabilityError);
    assert.equal(confined(project,'safe'),join(project,'safe'));
  } finally {rmSync(temporary,{recursive:true,force:true});}
});
test('each core assertion can be falsified independently without changing other roots', () => {
  const source=readFileSync(new URL('../probes/capabilities/Core.lex.tex',import.meta.url),'utf8');
  const parse=s=>JSON.parse(s.match(/\\semanticdata\{(.*)\}\n/)[1]);
  const original=parse(source);
  for(const row of contract.filter(c=>c.fixture==='Core.lex.tex')) {
    const mutant=parse(negateRegisteredRoot(source,row.root));
    const name=row.root.split('.').at(-1);
    const before=original.declarations.find(d=>d.name===name);
    const after=mutant.declarations.find(d=>d.name===name);
    assert.deepEqual(after.body,{kind:'not',value:before.body});
    for(const d of mutant.declarations.filter(d=>d.name!==name))assert.deepEqual(d,original.declarations.find(x=>x.name===d.name));
    assert.equal(mutant.declarations.length,original.declarations.length);
  }
  assert.throws(()=>negateRegisteredRoot(source,'UorcProbe.Main.absent'),CapabilityError);
});
test('successful baseline without its per-capability falsifier remains unaccepted', () => {
  const observations=['core.verify','core.app.build','core.app.verify','core.app.evidence'].map(id=>({id,status:'passed'}));
  const report=makeReport(contract,{sdk:'test reporting only',observations,bindings:[],artifacts:[]});
  assert.equal(report.capabilities.find(c=>c.id==='bytes').runtime,'negative_not_passed');
  assert.equal(report.accepted,false);
});

test('runtime requests reach their own assertions and retain all parameterized operations', () => {
  const parse=file=>JSON.parse(readFileSync(new URL(`../probes/capabilities/${file}`,import.meta.url),'utf8').match(/\\semanticdata\{(.*)\}\n/)[1]);
  const core=parse('Core.lex.tex'), exported=new Set();
  assert(!core.declarations.some(d=>d.kind==='theorem'));
  for(const row of contract.filter(row=>row.fixture==='Core.lex.tex')) {
    const app=parse(row.application),run=app.declarations.find(d=>d.name==='run');
    assert.deepEqual(run.body.condition.left,{arguments:[],function:{module:'Main',name:row.root.split('.').at(-1)},kind:'call'});
    const dynamic=run.body.condition.right;
    assert.deepEqual(dynamic.arguments,[{kind:'var',name:'request'}]);
    assert.equal(dynamic.function.module,'Main');
    const assertion=core.declarations.find(d=>d.name===dynamic.function.name);
    assert.deepEqual(assertion.parameters,[{name:'request',type:{kind:'bytes'}}]);
    assert.deepEqual(assertion.result,{kind:'bool'});
    let roots=app.declarations.find(d=>d.name==='application').body.fields.find(f=>f.field==='libraryRoots').value;const names=[];
    while(roots.kind==='cons'){names.push(roots.head.value);exported.add(roots.head.value);roots=roots.tail;}
    assert(names.includes(row.root));assert(names.includes(`UorcProbe.Main.${dynamic.function.name}`));
  }
  for(const name of ['appendBytes','indexBytes','sliceBytes','addU64','subtractU64','multiplyU64','countSteps','scan','recordVariantValue']) {
    assert(core.declarations.find(d=>d.name===name).parameters.length>0);
    assert(exported.has(`UorcProbe.Main.${name}`));
  }
  const bytes=core.declarations.find(d=>d.name==='runtimeBytes');
  assert(!JSON.stringify(bytes.body).includes('sliceBytes'));
  assert(!JSON.stringify(bytes.body).includes('indexBytes'));
});

test('each project profile binds only declarations in its own authored module', () => {
  const parse=file=>JSON.parse(readFileSync(new URL(`../probes/capabilities/${file}.lex.tex`,import.meta.url),'utf8').match(/\\semanticdata\{(.*)\}\n/)[1]);
  for(const row of contract.filter(row=>['Core.lex.tex','Store.lex.tex'].includes(row.fixture))) {
    const main=parse(row.fixture.replace('.lex.tex','')), app=parse(row.application.replace('.lex.tex',''));let roots=app.declarations.find(d=>d.name==='application').body.fields.find(f=>f.field==='libraryRoots').value;
    while(roots.kind==='cons') {
      const name=roots.head.value;
      assert(name==='UorcProbe.Application.run'||main.declarations.some(d=>`UorcProbe.Main.${d.name}`===name),name);
      roots=roots.tail;
    }
  }
});

test('profile vectors fit their separate input and response bounds and cover allocation boundary', () => {
  for(const name of [...new Set(contract.map(row=>row.application.replace('.lex.tex',''))),'BinaryResponse','RejectedAcceptance']) {
    const module=JSON.parse(readFileSync(new URL(`../probes/capabilities/${name}.lex.tex`,import.meta.url),'utf8').match(/\\semanticdata\{(.*)\}\n/)[1]);
    const fields=Object.fromEntries(module.declarations.find(d=>d.name==='application').body.fields.map(f=>[f.field,f.value]));
    const request=Number(fields.requestMaximum.value), guest=Number(fields.guestAllocationMaximum.value), response=Number(fields.responseMaximum.value);
    assert(request>0&&response>0&&guest>=request);
    let vectors=fields.acceptanceVectors,atBoundary=false;
    while(vectors.kind==='cons') {
      const vector=Object.fromEntries(vectors.head.fields.map(f=>[f.field,f.value.hex.length/2]));
      assert(vector.request<=request,`${name} request exceeds browser bound`);
      assert(vector.request<=guest,`${name} request exceeds guest input bound`);
      assert(vector.response<=response,`${name} response exceeds response bound`);
      atBoundary ||= vector.request===guest; vectors=vectors.tail;
    }
    assert(atBoundary,`${name} omits the required exact input-allocation boundary`);
  }
});

test('complete acceptance gate invokes real probes without weakening the source authority gate', () => {
  const just=readFileSync(new URL('../Justfile',import.meta.url),'utf8');
  const prerequisites=just.match(/^vv: (.*)$/m)[1].split(/\s+/);
  assert(prerequisites.includes('sdk-capabilities'));
  assert(prerequisites.includes('lexlean-artifacts'));
  assert(prerequisites.includes('reproducibility'));
  const recipe=just.split('\nsdk-capabilities:\n')[1];
  assert(recipe.includes('node scripts/capability-probes.mjs "$run/observations"'));
  assert(recipe.includes('set -euo pipefail'));
  assert(!recipe.includes('|| true'));
  const registry=readFileSync(new URL('../src/Uorc/Registry.lex.tex',import.meta.url),'utf8');
  assert(registry.includes('\\importmodule{Uorc.CapabilityContract}'));
  for(const id of ['UC-SDK-01','UC-SDK-02'])assert(registry.includes(`"value":"${id}"`));
});

test('each core capability has a distinct source-owned executable profile', () => {
  const cases=contract.filter(row=>row.fixture==='Core.lex.tex');
  assert.equal(new Set(cases.map(row=>row.project)).size,cases.length);
  for(const row of cases)assert(row.application.endsWith('.lex.tex'));
  const observations=['core.verify','core.app.build','core.app.verify','core.app.evidence','core.falsify.bytes'].map(id=>({id,status:'passed'}));
  const report=makeReport(contract,{sdk:'test reporting only',observations,bindings:[],artifacts:[]});
  assert.equal(report.capabilities.find(row=>row.id==='bytes').runtime,'passed');
  for(const id of ['checked-u64','records-variants','indexed-bytes','scan'])assert.notEqual(report.capabilities.find(row=>row.id===id).runtime,'passed');
});

// Synthetic receipts exercise this validator only; they never count as SDK evidence.
function syntheticReceipt() {
  const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
  const encode=value=>Buffer.from(JSON.stringify(value));
  const build={schema:'prismpm/build-result/1',build_id:'a'.repeat(64),source_id:'b'.repeat(64),semantic_id:'c'.repeat(64)};
  const lexleanBytes=encode({status:'verified',attestation_id:'d'.repeat(64),source_id:build.source_id,semantic_id:build.semantic_id});
  const modelBytes=encode({synthetic:true});
  const acceptance={schema:'prismpm/application-acceptance/1',status:'verified',artifact_closure:'verified',browser_projection:'verified',core_wasm:{status:'verified'},hologram_oracle:'verified',modeled_vectors:3,build_id:build.build_id,source_id:build.source_id,lexlean_attestation_id:'d'.repeat(64)};
  const acceptanceBytes=encode(acceptance);
  const processes=['lean-version','lake-version','rustfmt-version','rustc-version','timeout-version','hologram-oracle-build','hologram-oracle','core-wasm-validate','core-wasm-inspect','application-package-test','application-package-no-std','application-consumer-lock','application-generated-rust-corpus'].map(tool=>({tool,argv:[],executable_sha256:'e'.repeat(64),exit_code:0,stdout:tool==='hologram-oracle'?JSON.stringify({schema:'prismpm/hologram-oracle/1',footer_verified:true}):'',stderr:''}));
  const manifest={schema:'prismpm/application-verification-manifest/1',build_id:build.build_id,acceptance_sha256:hash(acceptanceBytes),lexlean_attestation_sha256:hash(lexleanBytes),model_sha256:hash(modelBytes),processes};
  const manifestBytes=encode(manifest), attestation_id=hash(manifestBytes);
  const verify={schema:'prismpm/verify-result/1',build_id:build.build_id,attestation_id,verified_root:`.prism/verified/${attestation_id}`};
  return {acceptance,manifest,acceptanceBytes,bundle:{build,verify,manifestBytes,lexleanBytes,modelBytes}};
}
function resealSynthetic(r) {
  r.acceptanceBytes=Buffer.from(JSON.stringify(r.acceptance));
  r.manifest.acceptance_sha256=createHash('sha256').update(r.acceptanceBytes).digest('hex');
  r.bundle.manifestBytes=Buffer.from(JSON.stringify(r.manifest));
  r.bundle.verify.attestation_id=createHash('sha256').update(r.bundle.manifestBytes).digest('hex');
  r.bundle.verify.verified_root=`.prism/verified/${r.bundle.verify.attestation_id}`;
}
function validateSynthetic(r) {validateAcceptance(r.acceptance,r.manifest,r.acceptanceBytes,r.bundle);}
test('synthetic complete receipt binds both real command result schemas and all evidence bytes', () => {
  validateSynthetic(syntheticReceipt());
  for(const target of ['build','verify']) {
    for(const field of ['schema','build_id']) {
      const r=syntheticReceipt(); r.bundle[target][field]='wrong';
      assert.throws(()=>validateSynthetic(r),CapabilityError,`${target}.${field}`);
    }
  }
  for(const field of ['attestation_id','verified_root']) {
    const r=syntheticReceipt(); r.bundle.verify[field]='wrong';assert.throws(()=>validateSynthetic(r),CapabilityError,field);
  }
  for(const field of ['manifestBytes','lexleanBytes','modelBytes']) {
    const r=syntheticReceipt();r.bundle[field]=Buffer.concat([r.bundle[field],Buffer.from(' ')]);assert.throws(()=>validateSynthetic(r),CapabilityError,field);
  }
  const r=syntheticReceipt();r.acceptanceBytes=Buffer.concat([r.acceptanceBytes,Buffer.from(' ')]);assert.throws(()=>validateSynthetic(r),CapabilityError);
});
test('every required SDK process appears once and completed successfully', () => {
  for(let i=0;i<13;i++) {
    for(const action of ['remove','replace','duplicate']) {
      const r=syntheticReceipt();
      if(action==='remove')r.manifest.processes.splice(i,1);
      if(action==='replace')r.manifest.processes[i].tool='unrelated';
      if(action==='duplicate')r.manifest.processes[i].tool=r.manifest.processes[(i+1)%13].tool;
      resealSynthetic(r);assert.throws(()=>validateSynthetic(r),CapabilityError,`${i}:${action}`);
    }
  }
  for(const [field,values] of Object.entries({exit_code:[1,'0',null,undefined],executable_sha256:['',null,'F'.repeat(64)],argv:[null,[1]],stdout:[null,1],stderr:[null,1]})) {
    for(const value of values){const r=syntheticReceipt();r.manifest.processes[0][field]=value;resealSynthetic(r);assert.throws(()=>validateSynthetic(r),CapabilityError,field);}
  }
});
test('empty or unrelated build identity cannot admit an otherwise labeled receipt', () => {
  const r=syntheticReceipt();r.acceptance.build_id='';r.manifest.build_id='';resealSynthetic(r);assert.throws(()=>validateSynthetic(r),CapabilityError);
  const other=syntheticReceipt();other.bundle.build.build_id='f'.repeat(64);assert.throws(()=>validateSynthetic(other),CapabilityError);
});

test('synthetic source or oracle mismatches fail even when receipt digests are updated', () => {
  for(const field of ['status','attestation_id','source_id','semantic_id']) {
    const r=syntheticReceipt(),lexlean=JSON.parse(r.bundle.lexleanBytes);lexlean[field]='wrong';
    r.bundle.lexleanBytes=Buffer.from(JSON.stringify(lexlean));r.manifest.lexlean_attestation_sha256=createHash('sha256').update(r.bundle.lexleanBytes).digest('hex');resealSynthetic(r);
    assert.throws(()=>validateSynthetic(r),CapabilityError,field);
  }
  for(const stdout of ['', '{}', JSON.stringify({schema:'prismpm/hologram-oracle/1',footer_verified:false})]) {
    const r=syntheticReceipt();r.manifest.processes.find(p=>p.tool==='hologram-oracle').stdout=stdout;resealSynthetic(r);assert.throws(()=>validateSynthetic(r),CapabilityError);
  }
});

test('offline oracle dependency failure never counts as runtime rejection', () => {
  const result={status:1,stdout:JSON.stringify({schema:'prismpm/error-result/1',diagnostic:{code:'PP5301',message:'hologram-oracle-build exited 101: stdout=""; stderr="error: no matching package named `tokio` found"'}}),stderr:''};
  const observed=classifyResult(result,{expectedExit:1,requiredDiagnostic:'PP5301',requiredOracleMismatch:true});
  assert.equal(observed.status,'dependency_blocked');assert.equal(observed.success,false);
});
test('oracle falsifier requires actual modeled-vector disagreement from executed oracle', () => {
  const result=message=>({status:1,stdout:JSON.stringify({schema:'prismpm/error-result/1',diagnostic:{code:'PP5301',message}}),stderr:''});
  const expected={expectedExit:1,requiredDiagnostic:'PP5301',requiredOracleMismatch:true};
  const matched=classifyResult(result('hologram-oracle exited 1: stdout=""; stderr="upstream direct execution disagrees with a modeled vector"'),expected);
  assert.equal(matched.success,true);assert.equal(matched.status,'passed');
  for(const message of ['hologram-oracle-build exited 101: upstream direct execution disagrees with a modeled vector','hologram-oracle exited 1: failed to load archive','pinned Hologram oracle did not return complete acceptance'])assert.equal(classifyResult(result(message),expected).success,false);
});
test('compiler failure remains distinct from an offline acquisition failure', () => {
  const result={status:1,stdout:JSON.stringify({schema:'prismpm/error-result/1',diagnostic:{code:'PP4102',message:'application-cargo-check exited 101: error[E0599]: no method named extend_from_slice found'}}),stderr:''};
  assert.equal(classifyResult(result).status,'failed');
});

test('persistent store exposes parameterized primitive operations and request-fed version checks', () => {
  const source=readFileSync(new URL('../probes/capabilities/Store.lex.tex',import.meta.url),'utf8');
  const module=JSON.parse(source.match(/\\semanticdata\{(.*)\}\n/)[1]);
  for(const [name,operation,arity] of [['insertStore','map_insert',3],['lookupStore','map_lookup',2],['removeStore','map_remove',2]]) {
    const definition=module.declarations.find(d=>d.name===name);
    assert(definition,`${name} must be authored`);assert.equal(definition.parameters.length,arity);
    assert.equal(definition.body.operation,operation);
    assert.deepEqual(definition.body.arguments,definition.parameters.map(p=>({kind:'var',name:p.name})));
  }
  const exercise=module.declarations.find(d=>d.name==='exerciseStore');
  assert.deepEqual(exercise.parameters.map(p=>p.name),['address','request']);
  const body=JSON.stringify(exercise.body);
  for(const name of ['original','written','replaced','removed'])assert(body.includes(`"name":"${name}"`));
  assert(body.includes('"name":"request"'));
  for(const name of ['insertStore','lookupStore','removeStore'])assert(body.includes(`"name":"${name}"`));
  const calls=[];const walk=value=>{if(!value||typeof value!=='object')return;if(value.kind==='call')calls.push(value);for(const child of Object.values(value))if(Array.isArray(child))child.forEach(walk);else walk(child);};walk(exercise.body);
  for(const name of ['original','written','replaced','removed','cleared'])assert(calls.filter(c=>c.function.name==='lookupStore'&&c.arguments[0].kind==='var'&&c.arguments[0].name===name).length>=2,`${name} must be read after later operations`);
  assert(calls.some(c=>c.function.name==='insertStore'&&c.arguments[2].kind==='var'&&c.arguments[2].name==='request'));
  const runtime=module.declarations.find(d=>d.name==='runtimeStore');
  assert.deepEqual(runtime.parameters,[{name:'request',type:{kind:'bytes'}}]);
  assert.equal(runtime.body.function.name,'exerciseStore');assert.deepEqual(runtime.body.arguments[1],{kind:'var',name:'request'});

});
test('store application executes request-dependent checks and retains all store operation roots', () => {
  const source=readFileSync(new URL('../probes/capabilities/StoreApplication.lex.tex',import.meta.url),'utf8');
  const module=JSON.parse(source.match(/\\semanticdata\{(.*)\}\n/)[1]);
  const run=module.declarations.find(d=>d.name==='run');
  assert.deepEqual(run.body.condition.right,{arguments:[{kind:'var',name:'request'}],function:{module:'Main',name:'runtimeStore'},kind:'call'});
  const application=JSON.stringify(module.declarations.find(d=>d.name==='application'));
  for(const name of ['insertStore','lookupStore','removeStore','exerciseStore','runtimeStore'])assert(application.includes(`UorcProbe.Main.${name}`));
});
