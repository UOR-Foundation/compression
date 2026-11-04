import fs from 'node:fs/promises';
const document=JSON.parse(await fs.readFile(process.argv[2],'utf8'));
if(document.spec!=='lexlean/target-descriptors/1') throw Error('unexpected target schema');
const target=document.targets.filter(t=>t.id==='rust-std');
if(target.length!==1 || !/^[a-f0-9]{64}$/.test(target[0].digest)) throw Error('missing exact rust-std descriptor');
// Atomic table writes and the old table/index controls are diagnostic only.
const roots=['Heap.allocate','Heap.releaseStep','HeapTests.suite',
  'RuntimeTable.beginPut','RuntimeTable.beginErase','RuntimeTable.step','RuntimeTable.cancel',
  'RuntimeTableCursorTests.suite',
  'RuntimeSequence.beginConcat','RuntimeSequence.beginSlice','RuntimeSequence.beginIndex',
  'RuntimeSequence.beginSpan','RuntimeSequence.step','RuntimeSequence.cancel',
  'RuntimeStore.step','RuntimeStore.cancel',
  'RuntimeByteBuilder.begin','RuntimeByteBuilder.beginAppendOwned',
  'RuntimeByteBuilder.beginFinishOwned','RuntimeByteBuilder.cancel','RuntimeByteBuilder.step'];
console.log(JSON.stringify({spec:'lexlean/production-request/1', roots:roots.sort().map(root=>({root:'Uorc.Uorc.Core.'+root, package:{name:'uorc_'+root.replace('.','_').toLowerCase(), version:'0.1.0'},targets:[{id:'rust-std',digest:target[0].digest}]}))}));
