import assert from 'node:assert/strict';
import { test } from 'node:test';
import { readFileSync, mkdtempSync, mkdirSync, writeFileSync, symlinkSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { loadContract, makeReport, validateAcceptance, CapabilityError, confined, negateRegisteredRoot } from './capability-probes.mjs';

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
  for (const file of ['Core','Store','Application','StoreApplication','BinaryResponse','RejectedAcceptance']) {
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

test('runtime requests reach parameterized source assertions and exported primitives', () => {
  const parse=file=>JSON.parse(readFileSync(new URL(`../probes/capabilities/${file}.lex.tex`,import.meta.url),'utf8').match(/\\semanticdata\{(.*)\}\n/)[1]);
  const core=parse('Core'), app=parse('Application');
  const run=app.declarations.find(d=>d.name==='run');
  assert.deepEqual(run.body.condition.left,{arguments:[{kind:'var',name:'request'}],function:{module:'Main',name:'runtimeAcceptance'},kind:'call'});
  assert(!core.declarations.some(d=>d.kind==='theorem'));
  const runtime=core.declarations.find(d=>d.name==='runtimeAcceptance');
  for(const name of ['runtimeBytes','runtimeU64','runtimeRecords','runtimeIndexed','runtimeScan'])assert(JSON.stringify(runtime.body).includes(`"name":"${name}"`));
  let roots=app.declarations.find(d=>d.name==='application').body.fields.find(f=>f.field==='libraryRoots').value;const names=[];
  while(roots.kind==='cons'){names.push(roots.head.value);roots=roots.tail;}
  for(const name of ['appendBytes','indexBytes','sliceBytes','addU64','subtractU64','multiplyU64','countSteps','scan','recordVariantValue']) {
    assert(core.declarations.find(d=>d.name===name).parameters.length>0);
    assert(names.includes(`UorcProbe.Main.${name}`));
  }
});

test('each project profile binds only declarations in its own authored module', () => {
  const parse=file=>JSON.parse(readFileSync(new URL(`../probes/capabilities/${file}.lex.tex`,import.meta.url),'utf8').match(/\\semanticdata\{(.*)\}\n/)[1]);
  for(const [source,profile] of [['Core','Application'],['Store','StoreApplication']]) {
    const main=parse(source), app=parse(profile);let roots=app.declarations.find(d=>d.name==='application').body.fields.find(f=>f.field==='libraryRoots').value;
    while(roots.kind==='cons') {
      const name=roots.head.value;
      assert(name==='UorcProbe.Application.run'||main.declarations.some(d=>`UorcProbe.Main.${d.name}`===name),name);
      roots=roots.tail;
    }
  }
});

test('profile vectors fit their separate input and response bounds and cover allocation boundary', () => {
  for(const name of ['Application','StoreApplication','BinaryResponse','RejectedAcceptance']) {
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
