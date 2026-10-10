import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {spawnSync} from 'node:child_process';
import {mutations, plant, expectedFailure, auditGate} from './claims-validation.mjs';

const source=fs.readFileSync(new URL('../src/Uorc/Claims.lex.tex',import.meta.url),'utf8');
const parse=s=>JSON.parse(s.match(/\\semanticdata\{(.*)\}\n/)[1]);
const changed=f=>{const m=parse(source);f(m);return source.replace(source.match(/\\semanticdata\{(.*)\}\n/)[1],JSON.stringify(m));};
const cases=mutations(source);

test('every source mutation changes precisely its designated field',()=>{
  assert.ok(cases.length>0);
  for(const row of cases) {
    const before=parse(source),after=parse(plant(source,row));
    const original=before.declarations.find(d=>d.name===row.definition).body.fields.find(f=>f.field===row.field);
    const target=after.declarations.find(d=>d.name===row.definition).body.fields.find(f=>f.field===row.field);
    assert.notDeepEqual(target,original);
    Object.assign(target,original);
    assert.deepEqual(after,before);
  }
  assert.throws(()=>plant(source,{...cases[0],definition:'missing'}),/not unique/);
  assert.throws(()=>plant(source,{...cases[0],field:'missing'}),/not unique/);
});
test('empty, duplicate, malformed and unowned source cases are rejected',()=>{
  const isCase=d=>d.result?.member?.name==='ClaimMutation';
  assert.throws(()=>mutations(changed(m=>{m.declarations=m.declarations.filter(d=>!isCase(d));})),/empty/);
  assert.throws(()=>mutations(changed(m=>m.declarations.push(m.declarations.find(isCase)))),/duplicate/);
  assert.throws(()=>mutations(changed(m=>m.declarations.find(isCase).body.fields.pop())),/inventory/);
  assert.throws(()=>mutations(changed(m=>{m.declarations=m.declarations.filter(d=>d.name!==cases[0].owner);})),/owning theorem/);
});
function rejection() {
  const row=cases[0];
  const generated=`public theorem ${row.owner} : True := by\n  rfl\n\npublic theorem other : True := by\n  rfl\n`;
  const diagnostic={code:'LLV7002',primary:{path:'src/Uorc/Claims.lex.tex'},message:'Tactic `rfl` failed',notes:[{message:'generated location: $STAGING/lean-src/Uorc/Uorc/Claims.lean:2:3'}]};
  const value={spec:'lexlean/command-result/1',success:false,diagnostics:[diagnostic]};
  return {row,generated,value,result:{status:1,signal:null,stdout:JSON.stringify(value)}};
}
test('only a completed kernel rejection at the exact owning theorem counts',()=>{
  const f=rejection();expectedFailure(f.result,f.row,f.generated);
  for(const patch of [{status:0},{status:null},{error:new Error('missing SDK')},{signal:'SIGTERM'},{stdout:'not JSON'}]) {
    assert.throws(()=>expectedFailure({...f.result,...patch},f.row,f.generated));
  }
  for(const edit of [
    v=>{v.success=true;},v=>{v.spec='foreign';},v=>{v.diagnostics=[];},
    v=>{v.diagnostics[0].code='LLT4001';},v=>{v.diagnostics[0].primary.path='src/Uorc/Registry.lex.tex';},
    v=>{v.diagnostics[0].message='syntax error';},v=>{v.diagnostics[0].notes=[];},
    v=>{v.diagnostics[0].notes[0].message='generated location: $STAGING/lean-src/Uorc/Uorc/Claims.lean:5:3';},
    v=>{v.diagnostics[0].notes[0].message='generated location: /stale/Claims.lean:2:3';}
  ]) {
    const value=structuredClone(f.value);edit(value);
    assert.throws(()=>expectedFailure({...f.result,stdout:JSON.stringify(value)},f.row,f.generated));
  }
  assert.throws(()=>expectedFailure(f.result,f.row,f.generated+f.generated),/ambiguous/);
  assert.throws(()=>expectedFailure(f.result,{...f.row,owner:'wrong'},f.generated),/absent/);
});
test('the complete mutation run is mandatory and its own source rejects omission',()=>{
  const just=fs.readFileSync(new URL('../Justfile',import.meta.url),'utf8');
  const harness=fs.readFileSync(new URL('./claims-validation.mjs',import.meta.url),'utf8');
  auditGate(just,harness);
  for(const changed of [just.replace('lexlean-artifacts claim-dispositions','lexlean-artifacts'),just.replace('node scripts/claims-validation.mjs','node scripts/claims-validation.mjs || true')]) assert.throws(()=>auditGate(changed,harness));
  for(const changed of [harness.replaceAll('for(const mutation of cases)', 'for(const mutation of cases.slice(1))'),harness.replaceAll('expectedFailure(result,mutation,generated);',''),harness.replaceAll('receipts.length===cases.length','true'),harness.replaceAll('for(const mutation of cases)','if(false) for(const mutation of cases)'),harness.replaceAll('for(const mutation of cases)','return; for(const mutation of cases)')]) assert.throws(()=>auditGate(just,changed));
  const own=fs.readFileSync(new URL('./claims-validation.test.mjs',import.meta.url),'utf8');
  assert.doesNotMatch(own,/test\.(skip|todo|only)\s*\(|\{\s*(skip|todo|only)\s*:/);
  const wrapper=fs.readFileSync(new URL('../xtask/src/claims.rs',import.meta.url),'utf8');
  assert.match(wrapper,/"--test",\s*"--test-reporter=tap",\s*"scripts\/claims-validation\.test\.mjs"/);
  assert.doesNotMatch(wrapper,/#\[ignore\]|--test-name-pattern|--test-skip-pattern/);
});
test('CLI rejects filters and unknown arguments before running the SDK',()=>{
  const result=spawnSync(process.execPath,['scripts/claims-validation.mjs','--skip'],{encoding:'utf8'});
  assert.equal(result.status,1);assert.match(result.stderr,/no harness arguments/);
});
