// Validation infrastructure; all production behavior is source-owned.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {moduleData,validateFixtures,sourceAudit,calledMembers} from './integrity-binding.mjs';
import {leanDriver} from './integrity-binding-runtime.mjs';
test('independent oracle checks all format, input, basis and checkpoint byte identities',()=>{
 assert.equal(validateFixtures('.').length,20);assert.equal(sourceAudit('.'),true);
});
test('owned source consumes tickets and exposes exact cursor/outcome aggregates',()=>{
 const owned=moduleData('.','OwnedHash');for(const name of ['beginSpan','borrowSpan','ownedRoots','outcomeRoots'])assert.ok(owned.declarations.some(d=>d.name===name));
 const moved=calledMembers(owned,'beginSpan'),borrowed=calledMembers(owned,'borrowSpan');
 assert.ok(moved.has('Uorc.Core.Heap::checkMoveRoot'));assert.ok(!moved.has('Uorc.Core.Heap::retain'));
 assert.ok(borrowed.has('Uorc.Core.Heap::retain'));
 const changed=structuredClone(owned);
 function drop(node){
  if(!node||typeof node!=='object')return;
  if(node.kind==='call'&&node.function.module==='Uorc.Core.Heap'&&node.function.name==='checkMoveRoot')node.function.name='invalidMovedRootCheck';
  for(const value of Object.values(node))if(Array.isArray(value))value.forEach(drop);else drop(value);
 }
 changed.declarations.filter(d=>d.kind==='definition').forEach(d=>drop(d.body));
 assert.ok(!calledMembers(changed,'beginSpan').has('Uorc.Core.Heap::checkMoveRoot'));
});
test('changed digest, truncated archive and dropped fixture fail the independent owner',()=>{
 const source=fs.readFileSync('src/Uorc/Integrity/BindingFixtures.lex.tex','utf8');const raw=[...source.matchAll(/\\semanticdata\{(.*)\}\n/g)][0][1];
 for(const change of [m=>m.declarations.find(d=>d.name==='fixture_12').body.fields.find(f=>f.field==='expectedHex').value.value='00'.repeat(32),m=>{const f=m.declarations.find(d=>d.name==='fixture_12').body.fields.find(f=>f.field==='archiveHex');f.value.value=f.value.value.slice(0,-2);},m=>m.declarations.splice(m.declarations.findIndex(d=>d.name==='fixture_12'),1)]){
  const tmp=fs.mkdtempSync(path.join(os.tmpdir(),'binding-mutant-'));try{const m=JSON.parse(raw);change(m);const dest=path.join(tmp,'src/Uorc/Integrity/BindingFixtures.lex.tex');fs.mkdirSync(path.dirname(dest),{recursive:true});fs.writeFileSync(dest,source.replace(raw,JSON.stringify(m)));assert.throws(()=>validateFixtures(tmp));}finally{fs.rmSync(tmp,{recursive:true,force:true});}
 }
});
test('collision evidence retains independent successful-SHA validation and scoped claims',()=>{
 const m=moduleData('.','CollisionEvidence');for(const name of ['extract','checkCandidate','runtimeChecks'])assert.ok(m.declarations.some(d=>d.name===name));
 const text=fs.readFileSync('src/Uorc/Integrity/CollisionEvidence.lex.tex','utf8');assert.ok(!/"kind":"axiom"|"kind":"sorry"/.test(text));
});
test('runtime client executes source controls and every independent binding fixture',()=>{
 const driver=leanDriver('.');
 for(const entry of ['BindingControls.runtimeChecks.all id','BindingControls.negativeChecks.all id','CollisionEvidence.runtimeChecks','HeapAudit.auditRoots','RuntimeSequence.beginSpan','RuntimeSequence.beginSlice'])assert.ok(driver.includes(entry),entry);
 for(const row of validateFixtures('.'))assert.ok(driver.includes(JSON.stringify(row.name)),row.name);
 assert.equal((driver.match(/private def cases :/g)||[]).length,1);
 assert.ok(!driver.includes('FIXTURES')&&!driver.includes('RECONSTRUCTIONS'));
});
