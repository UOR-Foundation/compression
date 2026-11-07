// Oracle import and generated-model test orchestration; contains no hash implementation.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {dataForm, notation} from './semantic-source.mjs';
const script=fileURLToPath(import.meta.url);
const contract='src/Uorc/Integrity/Contract.lex.tex';
export function requireFact(value,message){if(!value)throw Error(`UC-SHA-01: ${message}`);}
export function sourceRecords(root){
  const source=dataForm(fs.readFileSync(path.join(root,contract),'utf8'));
  const matches=[...source.matchAll(/\\semanticdata\{(.*)\}\n/g)];
  requireFact(matches.length===1,'one source contract is required');
  const module=JSON.parse(matches[0][1]);
  return module.declarations.filter(d=>d.kind==='definition').map(d=>({name:d.name,type:d.result.member.name,...Object.fromEntries(d.body.fields.map(f=>[f.field,f.value.kind==='nat'?Number(f.value.value):f.value.value]))}));
}
export function parseRsp(bytes,record,{verifyDigest=true}={}){
  requireFact(Buffer.isBuffer(bytes),'response file must be bytes');
  if(verifyDigest){
    requireFact(bytes.length===record.bytes,'response file byte count changed');
    requireFact(crypto.createHash('sha256').update(bytes).digest('hex')===record.sha256,'response file digest changed');
  }
  requireFact(bytes.every(x=>x===9||x===10||x===13||(x>=32&&x<=126)),'non-ASCII response data');
  const lines=bytes.toString('ascii').split(/\r?\n/);let header=false;let pending=[];const cases=[];
  for(const line of lines){
    if(line===''||line.startsWith('#'))continue;
    if(line==='[L = 32]'){requireFact(!header&&pending.length===0&&cases.length===0,'duplicate or misplaced header');header=true;continue;}
    requireFact(header,'missing SHA-256 output length header');
    const expected=['Len','Msg','MD'][pending.length];
    const match=line.match(/^(Len|Msg|MD) = ([0-9a-f]+)$/);
    requireFact(match&&match[1]===expected,`malformed or out-of-order ${expected} record`);
    pending.push(match[2]);
    if(pending.length===3){
      const [len,msg,md]=pending;requireFact(/^(0|[1-9][0-9]*)$/.test(len),'noncanonical bit length');
      const bits=Number(len);requireFact(Number.isSafeInteger(bits)&&bits%8===0,'unsupported bit length');
      requireFact(msg.length%2===0&&md.length===64,'invalid hex width');
      requireFact(bits===0?msg==='00':msg.length===bits/4,'message length mismatch');
      const expectedBits=record.firstBits+cases.length*record.strideBits;
      requireFact(bits===expectedBits,'missing, duplicated, reordered or unexpected vector');
      cases.push({id:`${record.name}:${bits}`,bits,message:bits===0?'':msg,expected:md});pending=[];
    }
  }
  requireFact(header&&pending.length===0,'incomplete response file');
  requireFact(cases.length===record.cases,'dropped or extra vectors');
  requireFact(cases.at(-1)?.bits===record.lastBits,'last length mismatch');
  return cases;
}
export function loadCorpus(root){
  const records=sourceRecords(root);const files=records.filter(r=>r.type==='CorpusFile');
  requireFact(files.length===2&&new Set(files.map(r=>r.path)).size===2,'corpus inventory is incomplete or duplicated');
  requireFact(files.map(r=>r.name).sort().join(',')==='SHA256LongMsg,SHA256ShortMsg','wrong corpus scope');
  const provenance=records.filter(r=>r.type==='Provenance');requireFact(provenance.length===1,'missing provenance');
  const citations=records.filter(r=>r.type==='CitedAuthority');
  requireFact(citations.length===files.length&&new Set(citations.map(r=>r.id)).size===files.length,'cited authority inventory incomplete');
  for(const file of files){
    const citation=citations.find(r=>r.id==='NIST-SHA256-'+(file.name.includes('Short')?'SHORT':'LONG'));
    requireFact(citation?.checksum==='sha256:'+file.sha256&&citation.realizedBy==='UC-SHA-01'&&citation.statement.includes(file.path),'cited authority disagrees with the locked response file');
  }
  const readme=fs.readFileSync(path.join(root,'standards/nist-sha256/Readme.txt'));
  requireFact(crypto.createHash('sha256').update(readme).digest('hex')===provenance[0].readmeSha256,'upstream Readme changed');
  return files.flatMap(record=>parseRsp(fs.readFileSync(path.join(root,record.path)),record));
}
export function leanDriver(cases,chainFixtures){
  requireFact(chainFixtures?.length===7,'complete seven-case bounded chain fixture inventory required');
  // Every digest is evaluated by the source-generated module. No host algorithm is substituted.
  const byteLiteral=hex=>`fromHex ${JSON.stringify(hex)}`;
  const chainRows=chainFixtures.map(c=>`(${byteLiteral(c.inputHex)}, ${byteLiteral(c.expectedHex)})`).join(',\n');
  const rows=cases.map(c=>`(${JSON.stringify(c.id)}, ${byteLiteral(c.message)}, ${byteLiteral(c.expected)})`).join(',\n');
  return `import Uorc.Uorc.Integrity.Sha256
import Uorc.Uorc.Integrity.BoundedChain
set_option autoImplicit false
set_option maxRecDepth 100000
set_option maxHeartbeats 1000000000
namespace IntegrityOracle
abbrev S := Uorc.Uorc.Integrity.Sha256.State
abbrev E := Uorc.Uorc.Integrity.Sha256.Error
open Uorc.Uorc.Integrity.Sha256
private def fail (message : String) : IO Unit := throw (IO.userError message)
private def check (condition : Bool) (message : String) : IO Unit := if condition then pure () else fail message
private def parts (input : ByteArray) (width : Nat) : Except E S := Id.run do
  let mut state := init
  let mut offset := 0
  for _ in [:input.size+1] do
    if offset < input.size then
      match update state (input.extract offset (offset+width)) with
      | .error e => return .error e
      | .ok next => state := next; offset := offset+width
  return .ok state
private def finish (state : S) : Except E ByteArray := do
  let state <- finalize state
  runFinalize 300 state
private def finishState : Nat -> S -> Except E S
| 0, _ => .error .fuelExhausted
| n+1, state => match state.phase with
  | .finished => .ok state
  | _ => do let next <- finalizeStep state; finishState n next
private def successful (result : Except E ByteArray) (expected : ByteArray) : Bool :=
  match result with | .ok bytes => bytes == expected | .error _ => false
private def fromHex (hex : String) : ByteArray := Id.run do
  let chars := hex.toList.toArray
  let mut bytes := ByteArray.empty
  for i in [:chars.size/2] do
    let nibble := fun (c : Char) => if c.toNat <= 57 then c.toNat-48 else c.toNat-87
    bytes := bytes.push (UInt8.ofNat (16 * nibble chars[2*i]! + nibble chars[2*i+1]!))
  return bytes
private def cases : Array (String × ByteArray × ByteArray) := #[
${rows}]
private def protocols : IO Unit := do
  check (successful (hash (ByteArray.mk #[97,98,99])) (ByteArray.mk #[186,120,22,191,143,1,207,234,65,65,64,222,93,174,34,35,176,3,97,163,150,23,122,156,180,16,255,97,242,0,21,173])) "FIPS abc fixture mismatch"
  let .ok initialJob := beginUpdate init ByteArray.empty | fail "beginUpdate failed"
  let .ok (.done _) := updateStep initialJob | fail "empty update failed"
  let .error .notFinalized := digest init | fail "unfinished digest accepted"
  let .error .notFinalized := finalizeStep init | fail "unstarted finalize accepted"
  let .ok pending := finalize init | fail "finalize failed"
  let .ok pendingAgain := finalize pending | fail "repeated pending finalize failed"
  check (pendingAgain.buffer == pending.buffer) "pending finalization changed bytes"
  let .error .updateAfterFinalize := beginUpdate pending ByteArray.empty | fail "update after finalize accepted"
  let .ok completed := finishState 300 pending | fail "finalize did not finish"
  let .ok again := finalize completed | fail "repeated finalize failed"
  check (successful (digest again) (match digest completed with | .ok b => b | _ => ByteArray.empty)) "repeated finalize changed digest"
  let .ok completedAgain := finalizeStep completed | fail "repeated finalize step failed"
  check (successful (digest completedAgain) (match digest completed with | .ok b => b | _ => ByteArray.empty)) "completed step changed digest"
  let .error .updateAfterFinalize := updateStep { state := completed, input := ByteArray.empty, offset := 0 } | fail "forged completed update accepted"
  let .error .invalidState := finalize {init with buffer := ByteArray.mk #[0]} | fail "counter/buffer mismatch accepted"
  let .error .invalidState := digest {init with work := some {base := initialWords, registers := initialWords, schedule := [], round := 0}} | fail "invalid schedule accepted"
  let .error .invalidState := digest {init with work := some {base := initialWords, registers := initialWords, schedule := List.replicate 17 0, round := 0}} | fail "oversized schedule accepted"
  let .error .invalidState := finalize {init with phase := .finishing (ByteArray.mk #[0])} | fail "misaligned padding state accepted"
  let .error .fuelExhausted := runFinalize 0 pending | fail "zero finalization fuel accepted"
  let .ok full := beginUpdate init (ByteArray.mk (Array.replicate 64 0)) | fail "full block update failed"
  let mut job := full
  for _ in [:64] do
    let .ok (.advanced next) := updateStep job | fail "byte stepping failed"
    job := next
  let .error .workPending := absorbByte job.state 0 | fail "pending work overwritten"
  let .ok finalWhileBusy := finalize job.state | fail "busy finalize failed"
  let .ok finalWhileBusyAgain := finalize finalWhileBusy | fail "busy repeated finalize failed"
  let .ok out := runFinalize 400 finalWhileBusyAgain | fail "busy finalization failed"
  check (successful (hash (ByteArray.mk (Array.replicate 64 0))) out) "busy finalization mismatch"
private def chainCases : Array (ByteArray × ByteArray) := #[
${chainRows}]
private def chains : IO Unit := do
  for (input, expected) in chainCases do
    let .ok observed := Uorc.Uorc.Integrity.BoundedChain.hash input | fail "chain golden hash failed"
    check (observed == expected) "independent bounded-chain golden mismatch"
  let input := ByteArray.mk (Array.ofFn (n := 257) fun i => UInt8.ofNat i.val)
  let .ok one := Uorc.Uorc.Integrity.BoundedChain.hash input | fail "chain hash failed"
  let .ok empty := Uorc.Uorc.Integrity.BoundedChain.hash ByteArray.empty | fail "empty chain failed"
  check (!(one == empty)) "chain framing fixture collided"
  let .ok initial := Uorc.Uorc.Integrity.BoundedChain.init | fail "chain init failed"
  let .error .invalidState := Uorc.Uorc.Integrity.BoundedChain.digest {initial with stage := .finished, closing := false} | fail "chain invalid finished phase accepted"
  let .error .invalidState := Uorc.Uorc.Integrity.BoundedChain.finalize {initial with stage := .idle, closing := true} | fail "chain invalid closing phase accepted"
  let mut state := initial
  for i in [:input.size] do
    let .ok next := Uorc.Uorc.Integrity.BoundedChain.update state (input.extract i (i+1)) | fail "chain partition update failed"
    state := next
  let .ok closing := Uorc.Uorc.Integrity.BoundedChain.finalize state | fail "chain finalize failed"
  let .error .updateAfterFinalize := Uorc.Uorc.Integrity.BoundedChain.beginUpdate closing ByteArray.empty | fail "chain accepted late input"
  let .ok again := Uorc.Uorc.Integrity.BoundedChain.finalize closing | fail "chain repeat finalize failed"
  let .ok many := Uorc.Uorc.Integrity.BoundedChain.runFinalize 600 again | fail "chain partition finalization failed"
  check (many == one) "chain partition changed digest"
  check (!(successful (hash input) one)) "chain mislabeled as standard SHA-256"
def main : IO Unit := do
  protocols
  chains
  let mut count := 0
  let mut partitions := 0
  for (id, input, expected) in cases do
    check (successful (hash input) expected) (id ++ ": known answer mismatch")
    for width in [1, 2, 7, 31, 64, 257] do
      check (successful (do let state <- parts input width; finish state) expected) (id ++ ": partition mismatch")
      partitions := partitions + 1
    if input.size <= 64 then
      for split in [:input.size+1] do
        check (successful (do let left <- update init (input.extract 0 split); let right <- update left (input.extract split input.size); finish right) expected) (id ++ ": split mismatch")
        partitions := partitions + 1
    count := count + 1
  IO.println s!"{count} complete CAVP cases; {partitions} partitions; protocol and chain cases passed"
end IntegrityOracle
#eval IntegrityOracle.main
`;
}
if(process.argv[1]&&path.resolve(process.argv[1])===script){
  try{
    requireFact(process.argv.length===2||process.argv.length===4&&process.argv[2]==='emit-lean-driver','expected no arguments or emit-lean-driver PATH');
    const cases=loadCorpus(process.cwd());
    if(process.argv.length===4)fs.writeFileSync(process.argv[3],leanDriver(cases,sourceRecords(process.cwd()).filter(r=>r.type==='ChainFixture')));
    console.log(`${cases.length} locked CAVP SHA-256 short/long cases imported without omissions.`);
  }catch(error){console.error(error.message);process.exitCode=1;}
}
