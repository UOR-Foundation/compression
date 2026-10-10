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
  for (const file of ['Core','Store','Application','BinaryResponse','RejectedAcceptance']) {
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
    assert(!mutant.declarations.some(d=>d.name==='finiteCases'));
  }
  assert.throws(()=>negateRegisteredRoot(source,'UorcProbe.Main.absent'),CapabilityError);
});
test('successful baseline without its per-capability falsifier remains unaccepted', () => {
  const observations=['core.verify','core.app.build','core.app.verify','core.app.evidence'].map(id=>({id,status:'passed'}));
  const report=makeReport(contract,{sdk:'test reporting only',observations,bindings:[],artifacts:[]});
  assert.equal(report.capabilities.find(c=>c.id==='bytes').runtime,'negative_not_passed');
  assert.equal(report.accepted,false);
});
