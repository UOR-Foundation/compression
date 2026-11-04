// Nonproduction acceptance infrastructure; the implementation is source-owned.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
test('maximum-frame implementation and distinct diagnostic seam exist',()=>{
  for(const name of ['StreamEngine','StreamMax','StreamDiagnostic'])assert.ok(fs.readFileSync(`src/Uorc/Integrity/${name}.lex.tex`,'utf8').includes('\\semanticdata{'));
});
import os from 'node:os';
import path from 'node:path';
import {inventory,leanDriver,records,sourcePaths} from './integrity-stream.mjs';
function changedContract(change){
 const root=fs.mkdtempSync(path.join(os.tmpdir(),'stream-contract-test-'));
 const target=path.join(root,'src/Uorc/Integrity/StreamContract.lex.tex');fs.mkdirSync(path.dirname(target),{recursive:true});
 const source=fs.readFileSync('src/Uorc/Integrity/StreamContract.lex.tex','utf8');
 const raw=[...source.matchAll(/\\semanticdata\{(.*)\}\n/g)][0][1];const model=JSON.parse(raw);change(model);
 fs.writeFileSync(target,source.replace(raw,JSON.stringify(model)));return {root,remove:()=>fs.rmSync(root,{recursive:true,force:true})};
}
test('all 94 cases and 189 independent frame records are complete',()=>{
 const {fixtures,frames}=inventory('.');assert.equal(fixtures.length,94);assert.equal(frames.length,189);
 assert.equal(new Set(fixtures.map(r=>r.frameLimit)).size,8);
 assert.equal(records('.').filter(r=>r.type==='StreamMutation').length,2);
});
test('dropped, duplicate and changed framing records are rejected',()=>{
 for(const change of [m=>m.declarations.splice(m.declarations.findIndex(d=>d.name==='frame_000_00'),1),m=>m.declarations.push(m.declarations.find(d=>d.name==='fixture_000')),m=>m.declarations.find(d=>d.name==='frame_000_00').body.fields.find(f=>f.field==='suffixHex').value.value='000000000000000000']){
  const fixture=changedContract(change);try{assert.throws(()=>inventory(fixture.root));}finally{fixture.remove();}
 }
});
test('production profile has no diagnostic capacity or type dependency',()=>{
 const text=fs.readFileSync(sourcePaths.find(p=>p.endsWith('/StreamMax.lex.tex')),'utf8');
 const m=JSON.parse([...text.matchAll(/\\semanticdata\{(.*)\}\n/g)][0][1]);
 assert.equal(m.declarations.find(d=>d.name==='init').parameters.length,0);
 for(const n of ['State','Update'])assert.ok(!m.declarations.find(d=>d.name===n).fields.some(f=>f.name==='frameLimit'));
 assert.ok(!text.includes('StreamDiagnostic'));
 assert.ok(JSON.stringify(m.declarations.find(d=>d.name==='init')).includes('productionLimit'));
});
test('driver retains every runtime case, all partitions, compression and cancellation owners',()=>{
 const driver=leanDriver('.');
 for(const row of inventory('.').fixtures)assert.ok(driver.includes(`"${row.name}"`));
 for(const marker of ['for width in #[max 1 (n/257), max 7 (n/17), max 64 (n/3)]','updateCompressionPending','compressionPending','actual compression-call mismatch','cancelUpdate retained chunk','first capacity bytes','continuation capacity bytes','fixture_000'])assert.ok(driver.includes(marker),marker);
 const just=fs.readFileSync('Justfile','utf8');assert.match(just,/^vv:.*\bintegrity\b/m);
 assert.ok(just.includes('    node --test scripts/integrity-stream.test.mjs\n    node scripts/integrity-stream.mjs\n'));
});
