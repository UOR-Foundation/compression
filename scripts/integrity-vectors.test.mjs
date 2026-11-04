import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {loadCorpus,sourceRecords,parseRsp,leanDriver} from './integrity-vectors.mjs';
const root=process.cwd();
const record=sourceRecords(root).find(r=>r.name==='SHA256ShortMsg');
const original=fs.readFileSync(record.path);
test('complete locked independent corpus has 65 short and 64 long vectors',()=>{
  const cases=loadCorpus(root);assert.equal(cases.length,129);
  for(const n of [0,1,55,56,63,64])assert(cases.some(c=>c.bits===8*n));
  assert.equal(new Set(cases.filter(c=>c.id.startsWith("SHA256LongMsg:")).map(c=>c.bits/8%64)).size,64);
});
test('pinned bytes reject any altered or dropped corpus bytes',()=>{
  const changed=Buffer.from(original);changed[changed.length-1]^=1;
  for(const bytes of [changed,original.subarray(0,-1)])assert.throws(()=>parseRsp(bytes,record),/changed/);
});
test('structural parser rejects omissions even if digest checking is independently disabled',()=>{
  const text=original.toString('ascii');
  const variants=[text.replace(/Len = 0\r\nMsg = 00\r\nMD = [a-f0-9]+\r\n/,''),text.replace('Len = 8','Len = 9'),text.replace('Msg = d3','Msg = d'),text.replace('[L = 32]','[L = 31]'),text.replace('Len = 8','Len = 0'),text.replace('Msg = d3','MD = d3'),text.replace('MD = e3b0','MD = z3b0'),text.replace('Len = 512','Len = 0512'),text.slice(0,-80)];
  for(const text of variants)assert.throws(()=>parseRsp(Buffer.from(text),record,{verifyDigest:false}),/UC-SHA-01/);
});
test('generated client tests invoke the source-generated models and retain complete vector IDs',()=>{
  const cases=loadCorpus(root),driver=leanDriver(cases,sourceRecords(root).filter(r=>r.type==='ChainFixture'));
  for(const c of cases)assert.equal(driver.split(JSON.stringify(c.id)).length-1,1);
  assert(driver.includes('for width in [1, 2, 7, 31, 64, 257]'));
  assert(driver.includes('protocols\n  chains'));
  assert(!driver.includes('sorry'));assert(!driver.includes('native_decide'));
});

test('source mutations preserve every expected observation and require a computational change',async()=>{
  const {plant}=await import('./integrity-validation.mjs');
  const source=fs.readFileSync('src/Uorc/Integrity/Sha256.lex.tex','utf8');
  const mutation=sourceRecords(root).find(r=>r.id==='wrapping_word');
  const mutant=plant(source,mutation);
  const parse=text=>JSON.parse(text.match(/\\semanticdata\{(.*)\}\n/)[1]);
  const before=parse(source),after=parse(mutant);
  assert.notDeepEqual(before.declarations.find(d=>d.name===mutation.definition),after.declarations.find(d=>d.name===mutation.definition));
  assert.deepEqual(before.declarations.filter(d=>d.kind==='theorem'),after.declarations.filter(d=>d.kind==='theorem'));
  assert.throws(()=>plant(source,{...mutation,before:'999999999999999'}),/changed no literal/);
});
test('wrong source, wrong owner, success and unrelated failures cannot qualify a planted fault',async()=>{
  const {expectedFailure}=await import('./integrity-validation.mjs');
  const mutation={owner:'wrapping_double_max'};
  const generated='public theorem wrapping_double_max : True := by\n  rfl\npublic theorem other : True := by\n  rfl\n';
  const result={status:1,stdout:JSON.stringify({success:false,diagnostics:[{code:'LLV7002',primary:{path:'src/Uorc/Integrity/Sha256.lex.tex'},notes:[{message:'generated location: $STAGING/lean-src/Uorc/Uorc/Integrity/Sha256.lean:2:2'}]}]})};
  assert.doesNotThrow(()=>expectedFailure(result,mutation,generated));
  for(const bad of [{...result,status:0},{...result,signal:'SIGKILL'},{...result,stdout:result.stdout.replace('LLV7002','LLC0102')},{...result,stdout:result.stdout.replace('Sha256.lex.tex','Other.lex.tex')},{...result,stdout:result.stdout.replace('.lean:2:2','.lean:4:2')}])assert.throws(()=>expectedFailure(bad,mutation,generated),/UC-SHA-01/);
});
test('gate audits reject skipped, conditional and prematurely ended execution',async()=>{
  const {auditRun}=await import('./integrity-validation.mjs');
  const just=fs.readFileSync('Justfile','utf8'),harness=fs.readFileSync('scripts/integrity-validation.mjs','utf8');
  assert.doesNotThrow(()=>auditRun(just,harness));
  for(const bad of [harness.replaceAll('for(const mutation of mutations){','for(const mutation of mutations.filter(()=>false)){'),harness.replaceAll('expectedFailure(failed,mutation,generated);',''),harness.replace('const cases=loadCorpus(root)','return; const cases=loadCorpus(root)'),harness.replaceAll('expectedFailure(failed,mutation,generated);','if(false) expectedFailure(failed,mutation,generated);')])assert.throws(()=>auditRun(just,bad),/UC-SHA-01/);
  assert.throws(()=>auditRun(just.replace(/^vv:.*$/m,line=>line.replace(/\bintegrity\b/,'')),harness),/UC-SHA-01/);
});
