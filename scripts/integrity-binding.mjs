// Independent validation oracle and source/evidence importer. Not a compressor,
// hash implementation, production adapter, or archive parser.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
export function requireFact(b,message){if(!b)throw new Error(message);}
export function moduleData(root,name){
 const text=fs.readFileSync(path.join(root,`src/Uorc/Integrity/${name}.lex.tex`),'utf8');
 const rows=[...text.matchAll(/\\semanticdata\{(.*)\}\n/g)];requireFact(rows.length===1,'one source semantic module required');
 return JSON.parse(rows[0][1]);
}
export function fixtures(root){
 const rows=moduleData(root,'BindingFixtures').declarations.filter(d=>d.kind==='definition'&&d.body.kind==='record'&&d.body.type.name==='Fixture').map(d=>Object.fromEntries(d.body.fields.map(f=>[f.field,f.value.value])));
 requireFact(rows.length===20&&new Set(rows.map(r=>r.name)).size===20,'complete unique binding inventory required');return rows;
}
const sha=b=>crypto.createHash('sha256').update(b).digest();
function digest(format,input){
 if(format===2){
  const d=Buffer.from('UORC-SHA256-CHAIN-v1\0');let chain=sha(Buffer.concat([d,Buffer.from([0])])),offset=0;
  while(input.length-offset>=64){chain=sha(Buffer.concat([d,Buffer.from([1]),chain,input.subarray(offset,offset+64)]));offset+=64;}
  const tail=input.subarray(offset);return sha(Buffer.concat([d,Buffer.from([2]),chain,Buffer.from([tail.length]),tail]));
 }
 requireFact(format===3,'unsupported fixture profile');const length=Buffer.alloc(8);length.writeBigUInt64BE(BigInt(input.length));
 return sha(Buffer.concat([Buffer.from('UORC-SHA256-STREAM-MAX-v1\0'),Buffer.from([0]),input,Buffer.from([1]),length]));
}
function prefix(format,purpose){
 if(purpose==='archivePreimage')return Buffer.alloc(0);
 if(purpose==='inputBytes')return Buffer.from(`UORC-INPUT-BYTES-F${format}-M1-v1\0`);
 if(purpose==='basisObjects')return Buffer.from(format===2?'UORC-BASIS-V1\0':'UORC-BASIS-F3-M1-v1\0');
 requireFact(purpose==='checkpointBytes','unknown fixture purpose');return Buffer.from(`UORC-CHECKPOINT-F${format}-M1-v1\0`);
}
export function validateFixtures(root){
 const rows=fixtures(root);let archives=0;
 for(const r of rows){
  const format=Number(r.format);for(const key of ['inputHex','expectedHex','archiveHex','bodyHex','basisStreamHex','basisIdHex'])requireFact(/^(?:[0-9a-f]{2})*$/.test(r[key]),'invalid fixture octets');
  requireFact(r.expectedHex.length===64,'digest width');
  const input=Buffer.from(r.inputHex,'hex');requireFact(digest(format,Buffer.concat([prefix(format,r.purpose),input])).toString('hex')===r.expectedHex,`${r.name}: independent digest`);
  if(r.archiveHex!==''){
   archives++;requireFact(r.purpose==='archivePreimage'&&r.archiveHex===r.inputHex+r.expectedHex,'archive integrity boundary');
   requireFact(input.subarray(0,6).equals(Buffer.from([85,79,82,67,format,1])),'fixed format binding prefix');
   const offset=Number(r.bodyOffset);requireFact(Number.isSafeInteger(offset)&&offset>=6&&offset<=input.length&&input.subarray(offset).toString('hex')===r.bodyHex,'independent raw reconstruction bytes');
   if(r.basisIdHex!=='')requireFact(digest(format,Buffer.concat([prefix(format,'basisObjects'),Buffer.from(r.basisStreamHex,'hex')])).toString('hex')===r.basisIdHex,'basis identity domain');
  }
 }
 requireFact(archives===10,'complete raw reconstruction inventory');return rows;
}
export function sourceAudit(root){
 const owned=moduleData(root,'OwnedHash'),profile=moduleData(root,'Profile');
 const declarations=new Map(owned.declarations.map(d=>[d.name,d]));
 function closure(name,seen=new Set()){
  if(seen.has(name))return '';seen.add(name);
  const d=declarations.get(name);requireFact(d,'missing local source dependency '+name);
  let text=JSON.stringify(d.body);
  function walk(node){
   if(!node||typeof node!=='object')return;
   if(node.kind==='call'&&!node.function.module&&declarations.has(node.function.name))text+=closure(node.function.name,seen);
   for(const value of Object.values(node))if(Array.isArray(value))value.forEach(walk);else walk(value);
  }
  walk(d.body);return text;
 }
 for(const name of ['start','beginSpan','borrowSpan','step','finalize','cancel','ownedRoots','outcomeRoots']){
  const d=owned.declarations.find(d=>d.name===name);requireFact(d,'missing ownership entry '+name);
  requireFact(!d.parameters.some(p=>p.type.kind==='bytes'),'production entry accepts unowned bytes');
 }
 const constructors=profile.declarations.find(d=>d.name==='Kind').constructors.map(c=>c.name);requireFact(constructors.join(',')==='format2Chain,format3Stream','profile map changed');
 const start=closure('start');requireFact(start.includes('allocateControl')&&start.includes('init'),'initial storage charge absent');
 const finalize=closure('finalize');requireFact(finalize.includes('reserveImage'),'digest reservation absent');
 return true;
}
