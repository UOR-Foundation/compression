// Validation only. Historical bytes remain the source of the old native receipt;
// this bridge proves model equivalence, never execution of current native bytes.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {isDeepStrictEqual} from 'node:util';
const receiptPath='validation/integrity/source-equivalence-v1.json';
const sha=bytes=>crypto.createHash('sha256').update(bytes).digest('hex');
function required(ok,message){if(!ok)throw new Error(`integrity source provenance: ${message}`);}
function model(bytes){
 const rows=[...bytes.toString('utf8').matchAll(/\\semanticdata\{(.*)\}\n/g)];
 required(rows.length===1,'one SDK semanticdata module required');return JSON.parse(rows[0][1]);
}
export function validateSourceProvenance(root){
 const read=file=>fs.readFileSync(path.join(root,file));
 const evidence=JSON.parse(read('validation/integrity/stream-evidence.json'));
 const receipt=JSON.parse(read(receiptPath));
 required(receipt.schema==='uorc/historical-source-equivalence/1','receipt schema');
 required(receipt.role==='historical native observation; current computational-model equivalence only','receipt role');
 required(receipt.historical_evidence_sha256===sha(read('validation/integrity/stream-evidence.json')),'historical evidence changed');
 const sourcePaths=Object.keys(evidence.source_sha256).sort();
 required(isDeepStrictEqual(Object.keys(receipt.sources).sort(),sourcePaths),'complete historical source inventory');
 for(const file of sourcePaths){
  const row=receipt.sources[file];
  required(row.historical_path===`validation/integrity/historical-stream-source/${path.basename(file)}`,'historical snapshot path');
  const historical=read(row.historical_path),current=read(file);
  required(sha(historical)===evidence.source_sha256[file],`${file}: historical source hash`);
  required(sha(current)===row.current_source_sha256,`${file}: current source hash`);
  required(Array.isArray(row.executable_additions)&&new Set(row.executable_additions).size===row.executable_additions.length,`${file}: unique annotation inventory`);
  const before=model(historical),after=model(current);
  if(row.executable_additions.length===0){
   required(historical.equals(current),`${file}: unmodified source bytes`);
  }else{
   required(before.spec==='lexlean/semantic-module/1'&&after.spec==='lexlean/semantic-module/2',`${file}: exact schema migration`);
   after.spec=before.spec;
   for(const name of row.executable_additions){
    const old=before.declarations.filter(d=>d.name===name),now=after.declarations.filter(d=>d.name===name);
    required(old.length===1&&now.length===1&&old[0].kind==='definition'&&now[0].kind==='definition',`${file}: computational annotation owner ${name}`);
    required(!Object.hasOwn(old[0],'executable')&&now[0].executable===true,`${file}: exact executable addition ${name}`);
    delete now[0].executable;
   }
   required(isDeepStrictEqual(before,after),`${file}: computational model changed`);
  }
 }
 return true;
}
