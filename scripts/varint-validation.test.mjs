import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {spawnSync} from 'node:child_process';
import {parse,plant,mutations,expectedFailure,auditGate} from './varint-validation.mjs';
const source=fs.readFileSync(new URL('../src/Uorc/Varint.lex.tex',import.meta.url),'utf8');
const cases=mutations(process.cwd());
test('core mutations copy the target DAG without changing proof or vector inputs',()=>{
  const original=parse(source);
  for(const row of cases){
    const changed=parse(plant(source,row));
    assert.deepEqual(changed.nodes.slice(0,original.nodes.length),original.nodes);
    assert.deepEqual(changed.proof_nodes,original.proof_nodes);
    let touched=0;
    for(let i=0;i<original.declarations.length;i++){
      const before=original.declarations[i],after=changed.declarations[i];
      if(before.name===row.definition){
        assert.notEqual(before.value,after.value);assert.ok(after.value>=original.nodes.length);
        after.value=before.value;touched++;
      }
      assert.deepEqual(before,after);
    }
    assert.equal(touched,1);
  }
  assert.throws(()=>plant(source,{...cases[0],definition:'missing'}),/unique/);
  assert.throws(()=>plant(source,{...cases[0],before:'9876543212345678987654321'}),/no intended literal/);
});
function rejection(){
  const row=cases[0],diagnostic={code:'LLV7002',primary:{path:'src/Uorc/Varint.lex.tex'},message:`Lean rejected: native core declaration '${row.owner}':\n (kernel) application type mismatch`};
  const value={spec:'lexlean/command-result/1',success:false,diagnostics:[diagnostic]};
  return {row,value,result:{status:1,signal:null,stdout:JSON.stringify(value)}};
}
test('failure attribution requires the exact kernel owner and a type mismatch',()=>{
  const f=rejection();expectedFailure(f.result,f.row);
  for(const patch of [{status:0},{status:null},{signal:'SIGTERM'},{error:new Error('missing SDK')},{stdout:'not JSON'}])assert.throws(()=>expectedFailure({...f.result,...patch},f.row));
  for(const edit of [v=>{v.success=true;},v=>{v.spec='foreign';},v=>{v.diagnostics=[];},v=>{v.diagnostics[0].code='LLI9001';},v=>{v.diagnostics[0].primary.path='src/Uorc/Claims.lex.tex';},v=>{v.diagnostics[0].message=`native core declaration 'wrong': (kernel) application type mismatch ${f.row.owner}`;},v=>{v.diagnostics[0].message=`native core declaration '${f.row.owner}': (kernel) unknown constant`; }]){
    const value=structuredClone(f.value);edit(value);assert.throws(()=>expectedFailure({...f.result,stdout:JSON.stringify(value)},f.row));
  }
});
test('the required gate and full execution cannot be filtered or silently omitted',()=>{
  const just=fs.readFileSync(new URL('../Justfile',import.meta.url),'utf8'),harness=fs.readFileSync(new URL('./varint-validation.mjs',import.meta.url),'utf8');
  auditGate(just,harness);
  for(const altered of [just.replace('claim-dispositions varint-proofs','claim-dispositions'),just.replace('node scripts/varint-validation.mjs','node scripts/varint-validation.mjs || true')])assert.throws(()=>auditGate(altered,harness));
  for(const altered of [harness.replaceAll('for(const mutation of cases)','for(const mutation of cases.slice(1))'),harness.replaceAll('expectedFailure(result,mutation);',''),harness.replaceAll('receipts.length===cases.length','true'),harness.replaceAll('for(const mutation of cases)','if(false) for(const mutation of cases)'),harness.replaceAll('for(const mutation of cases)','return; for(const mutation of cases)')])assert.throws(()=>auditGate(just,altered));
  const own=fs.readFileSync(new URL('./varint-validation.test.mjs',import.meta.url),'utf8');
  assert.doesNotMatch(own,/test\.(skip|todo|only)\s*\(|\{\s*(skip|todo|only)\s*:/);
  const wrapper=fs.readFileSync(new URL('../xtask/src/varint.rs',import.meta.url),'utf8');
  assert.match(wrapper,/"--test",\s*"--test-reporter=tap",\s*"scripts\/varint-validation\.test\.mjs"/);
  assert.doesNotMatch(wrapper,/#\[ignore\]|--test-name-pattern|--test-skip-pattern/);
});
test('unknown CLI arguments cannot filter the mutation inventory',()=>{
  const result=spawnSync(process.execPath,['scripts/varint-validation.mjs','--skip'],{encoding:'utf8'});assert.equal(result.status,1);assert.match(result.stderr,/no harness arguments/);
});
