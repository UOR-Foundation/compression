// Source-owned stream model execution only; independent fixture bytes come from
// StreamContract. No hashing/framing implementation or production fallback here.
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import crypto from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const script=fileURLToPath(import.meta.url);
export const sourceNames=['Sha256','StreamEngine','StreamMax','StreamDiagnostic','StreamContract'];
export const sourcePaths=sourceNames.map(n=>`src/Uorc/Integrity/${n}.lex.tex`);
export function required(b,m){if(!b)throw new Error(m);}
export function records(root){
 const s=fs.readFileSync(path.join(root,'src/Uorc/Integrity/StreamContract.lex.tex'),'utf8');
 const rows=[...s.matchAll(/\\semanticdata\{(.*)\}\n/g)];required(rows.length===1,'one stream source contract required');
 return JSON.parse(rows[0][1]).declarations.filter(d=>d.kind==='definition'&&d.body.kind==='record').map(d=>({name:d.name,type:d.body.type.name,...Object.fromEntries(d.body.fields.map(f=>[f.field,f.value.value]))}));
}
export function inventory(root){
 const all=records(root),fixtures=all.filter(r=>r.type==='StreamFixture'),frames=all.filter(r=>r.type==='FrameFixture');
 required(fixtures.length===94&&new Set(fixtures.map(r=>r.name)).size===94,'complete 94-case stream inventory required');
 required(frames.length===189,'complete 189-frame inventory required');
 required(fixtures.every(r=>typeof r.leanRuntime==='boolean')&&fixtures.filter(r=>!r.leanRuntime).map(r=>Number(r.inputBytes)).join(',')==='65536,1048576','source-owned diagnostic backend inventory changed');
 required(fixtures.filter(r=>r.frameLimit==='2305843009213693942').length===27,'production fixture inventory missing');
 for(const r of fixtures){
  required(/^[0-9a-f]{64}$/.test(r.expectedHex),'invalid expected digest');
  const own=frames.filter(f=>f.frameLimit===r.frameLimit&&f.inputBytes===r.inputBytes);
  required(own.length===Number(r.frames),'frame inventory mismatch');
  required(own.reduce((n,f)=>n+Number(f.compressionCalls),0)===Number(r.compressionCalls),'compression count mismatch');
  for(let i=0;i<own.length;i++){
   const f=own[i],prefix=i===0?27:59,terminal=i===own.length-1;
   required(Number(f.frameIndex)===i&&Number(f.prefixBytes)===prefix,'prefix/index mismatch');
   required(Number(f.messageBytes)===prefix+Number(f.payloadBytes)+9,'framing byte count mismatch');
   required(f.suffixHex===(terminal?'01':'00')+BigInt(f.payloadBytes).toString(16).padStart(16,'0'),'suffix byte count/role mismatch');
   required(BigInt(f.payloadBytes)<=(BigInt(r.frameLimit)-BigInt(prefix)),'payload exceeds frame limit');
   required(terminal?BigInt(f.payloadBytes)<BigInt(r.frameLimit)-BigInt(prefix):BigInt(f.messageBytes)===BigInt(r.frameLimit)+9n,'noncanonical full/terminal frame');
   required(Number(f.compressionCalls)===Math.ceil((Number(f.messageBytes)+9)/64),'SHA padding compression count mismatch');
  }
 }
 return {fixtures,frames};
}
export function nativeInput(root){
 const {fixtures,frames}=inventory(root);
 const rows=fixtures.map(r=>[r.name,r.frameLimit,r.inputBytes,r.expectedHex,r.compressionCalls].join('\t'));
 const trailers=frames.map(r=>[r.name,Number(r.frameIndex)===0?'true':'false',String(BigInt(r.payloadBytes)+BigInt(r.prefixBytes)),r.suffixHex.startsWith('01')?'true':'false',r.suffixHex].join('\t'));
 return [...rows,...trailers].join('\n')+'\n';
}
export function leanDriver(root){
 const {fixtures,frames}=inventory(root);
 const selected=fixtures.filter(r=>r.leanRuntime);required(selected.length===92,'source-owned Lean runtime inventory');
 const rows=selected.map(r=>`(${JSON.stringify(r.name)}, ${r.frameLimit}, ${r.inputBytes}, ${JSON.stringify(r.expectedHex)}, ${r.compressionCalls})`).join(',\n');
 const fr=frames.filter(f=>f.messageHex!=='').map(f=>`(${JSON.stringify(f.name)}, ${Number(f.frameIndex)===0?'true':'false'}, ${f.payloadBytes}, ${f.suffixHex.startsWith('01')?'true':'false'}, ${JSON.stringify(f.suffixHex)}, ${JSON.stringify(f.messageHex)}, ${JSON.stringify(f.digestHex)})`).join(',\n');
 return `import Uorc.Uorc.Integrity.StreamMax
import Uorc.Uorc.Integrity.StreamDiagnostic
set_option autoImplicit false
set_option maxRecDepth 100000
set_option maxHeartbeats 1000000000
namespace StreamOracle
open Uorc.Uorc.Integrity
private def fail {α : Type} (m : String) : IO α := throw (IO.userError m)
private def check (b : Bool) (m : String) : IO Unit := if b then pure () else fail m
private def hex (s : String) : ByteArray := Id.run do
 let chars := s.toList.toArray
 let mut out := ByteArray.empty
 for i in [:chars.size/2] do
  let nibble := fun (c : Char) => if c.toNat <= 57 then c.toNat-48 else c.toNat-87
  out := out.push (UInt8.ofNat (16*nibble chars[2*i]!+nibble chars[2*i+1]!))
 return out
private def pattern (n : Nat) : ByteArray := ByteArray.mk (Array.ofFn (n := n) fun i => UInt8.ofNat i.val)
private def canceled (s : StreamDiagnostic.State) : IO Unit := do
 let c := StreamDiagnostic.cancel s
 let .canceled := c.engine.stage | fail "cancel retained pending work"
 check (!c.engine.closing && c.engine.first) "cancel noncanonical"
 let .error .canceled := StreamDiagnostic.digest c | fail "canceled digest"
 let .error .canceled := StreamDiagnostic.step c | fail "canceled step"
 let .error .canceled := StreamDiagnostic.finalize c | fail "canceled finalize"
 let .error .canceled := StreamDiagnostic.beginUpdate c ByteArray.empty | fail "canceled update"
 let .error .canceled := StreamDiagnostic.updateStep {frameLimit:=c.frameLimit,engine:={state:=c.engine,input:=ByteArray.empty,offset:=0}} | fail "canceled update step"
 let again := StreamDiagnostic.cancel c
 let .canceled := again.engine.stage | fail "repeated cancel"
private def stageId (s : StreamEngine.State) : Nat := match s.stage with
 | .initializing _ => 0 | .ready _ => 1 | .suffix _ _ => 2
 | .finalizing _ _ => 3 | .finished _ => 4 | .canceled => 5
private def drive (s : StreamDiagnostic.State) (input : ByteArray) : IO (StreamDiagnostic.State × Nat) := do
 let .ok start := StreamDiagnostic.beginUpdate s input | fail "begin update"
 let mut job := start
 let mut count := 0
 for _ in [:600*input.size+1200] do
  if StreamDiagnostic.updateCompressionPending job then count := count+1
  match StreamDiagnostic.updateStep job with
  | .error _ => fail "update step error"
  | .ok (.advanced next) => job := next
  | .ok (.done done) => return (done,count)
 fail "update event bound"
private def finish (s : StreamDiagnostic.State) : IO (StreamDiagnostic.State × Nat) := do
 let .ok start := StreamDiagnostic.finalize s | fail "finalize"
 let mut state := start
 let mut count := 0
 for _ in [:1200] do
  let .ok repeated := StreamDiagnostic.finalize state | fail "repeated finalize"
  check (stageId repeated.engine == stageId state.engine) "repeated finalize changed stage"
  match StreamDiagnostic.digest state with
  | .ok _ => return (state,count)
  | .error .notFinalized => pure ()
  | .error _ => fail "digest protocol"
  if StreamDiagnostic.compressionPending state then count := count+1
  let .ok next := StreamDiagnostic.finalizeStep state | fail "finalize step"
  state := next
 fail "finalize event bound"
private def cancellationStages : IO Unit := do
 let .ok initial := StreamDiagnostic.init 96 | fail "cancel init"
 let .ok started := StreamDiagnostic.beginUpdate initial (pattern 200) | fail "cancel job"
 let mut job := started
 let mut seen : Array Nat := #[]
 for _ in [:120000] do
  let id := stageId job.engine.state
  if !seen.contains id then
   seen := seen.push id
   canceled {frameLimit:=job.frameLimit,engine:=job.engine.state}
   let c := StreamDiagnostic.cancelUpdate job
   let .canceled := c.engine.stage | fail "cancelUpdate retained chunk"
  match StreamDiagnostic.updateStep job with
  | .error _ => fail "cancel monitor step"
  | .ok (.advanced next) => job := next
  | .ok (.done _) => break
 check (seen.contains 0 && seen.contains 1 && seen.contains 2 && seen.contains 3) "cancellation stage coverage"
private def cases : Array (String × UInt64 × Nat × String × Nat) := #[${rows}]
private def frameCases : Array (String × Bool × UInt64 × Bool × String × String × String) := #[${fr}]
private def protocols : IO Unit := do
 let .error .invalidFrameLimit := StreamDiagnostic.init 59 | fail "small diagnostic limit"
 let .error .invalidFrameLimit := StreamDiagnostic.init 2305843009213693943 | fail "large diagnostic limit"
 let .ok initial := StreamDiagnostic.init 96 | fail "diagnostic init"
 let .error .notFinalized := StreamDiagnostic.digest initial | fail "unfinished digest"
 let .error .fuelExhausted := StreamDiagnostic.runFinalize 0 initial | fail "zero fuel"
 let .ok job := StreamDiagnostic.beginUpdate initial ByteArray.empty | fail "initial job"
 let .error .invalidOffset := StreamDiagnostic.updateStep {job with engine:={job.engine with offset:=1}} | fail "offset"
 let bad : StreamDiagnostic.State := {frameLimit:=96,engine:={first:=true,closing:=false,stage:=.initializing {state:=Sha256.init,input:=ByteArray.empty,offset:=0}}}
 let .error .invalidState := StreamDiagnostic.digest bad | fail "invalid state"
 let .ok ready := StreamDiagnostic.update initial ByteArray.empty | fail "empty update"
 let .error .workPending := StreamDiagnostic.step ready | fail "idle step"
 let .ok closing := StreamDiagnostic.finalize ready | fail "start finalization"
 let .error .updateAfterFinalize := StreamDiagnostic.beginUpdate closing ByteArray.empty | fail "late update"
 let .error .notFinalized := StreamDiagnostic.runFinalize 1200 ready | fail "unstarted finalization driver"
 let .error (.sha256 .invalidState) := StreamEngine.startSuffix ready.engine {Sha256.init with buffer:=pattern 1} true | fail "nested error propagation"
 canceled initial
 let (finished,_) ← finish ready
 canceled finished
 let .ok bytes := StreamDiagnostic.digest finished | fail "finished digest"
 let .ok unchanged := StreamDiagnostic.step finished | fail "finished step"
 let .ok same := StreamDiagnostic.digest unchanged | fail "finished step digest"
 check (bytes==same) "finished step changed bytes"
 let .error .updateAfterFinalize := StreamDiagnostic.beginUpdate finished ByteArray.empty | fail "finished update"
 -- Both terminal and full suffix roles and all eight byte-count octets.
 check (StreamEngine.byteLengthBytes 2305843009213693951 == hex "1fffffffffffffff") "max byte length"
 check (StreamEngine.footer true StreamEngine.productionLimit false == hex "001fffffffffffffdb") "first capacity bytes"
 check (StreamEngine.footer false StreamEngine.productionLimit false == hex "001fffffffffffffbb") "continuation capacity bytes"
private def main : IO Unit := do
 protocols
 cancellationStages
 for (name,first,payload,terminal,suffix,message,expected) in frameCases do
  check (StreamEngine.footer first (payload+(if first then 27 else 59)) terminal == hex suffix) (name++" footer bytes")
  let .ok observed := Sha256.hash (hex message) | fail (name++" standard hash")
  check (observed==hex expected) (name++" independent framed SHA digest")
 let mut partitions := 0
 for (name,limit,n,expected,calls) in cases do
  let input := pattern n
  let .ok initial := StreamDiagnostic.init limit | fail name
  let (updated,used) ← drive initial input
  let (done,padding) ← finish updated
  let .ok observed := StreamDiagnostic.digest done | fail name
  check (observed==hex expected) (name++" digest mismatch")
  check (used+padding==calls) (name++" actual compression-call mismatch")
  if limit==StreamEngine.productionLimit then
   let .ok production := StreamMax.hash input | fail (name++" production hash")
   check (production==observed) (name++" diagnostic/production mismatch")
  for width in #[max 1 (n/257), max 7 (n/17), max 64 (n/3)] do
   let mut state := initial
   let mut offset := 0
   let mut used := 0
   while offset<input.size do
    let (next,c) ← drive state (input.extract offset (offset+width))
    state := next; used := used+c; offset := offset+width
   let (done,extra) ← finish state
   let .ok partitioned := StreamDiagnostic.digest done | fail name
   check (partitioned==observed && used+extra==calls) (name++" partition/count mismatch")
   partitions := partitions+1
 IO.println s!"{cases.size} generated-Lean stream fixtures; {partitions} partitions; actual compression counts; framing and cancellation cases passed"
end StreamOracle
#eval StreamOracle.main
`;
}
export function plant(source,id){
 const raw=[...source.matchAll(/\\semanticdata\{(.*)\}\n/g)][0][1],module=JSON.parse(raw);
 const name=id==='production_limit'?'productionLimit':'footer';
 const owner=module.declarations.find(d=>d.name===name);required(owner,'missing mutation owner');
 let changes=0;
 function visit(x){
  if(!x||typeof x!=='object')return;
  if(id==='production_limit'&&x.kind==='integer'&&x.value==='2305843009213693942'){x.value='2305843009213693941';changes++;}
  if(id==='terminal_role'&&x.kind==='bytes'&&x.hex==='01'){x.hex='00';changes++;}
  for(const child of Object.values(x))visit(child);
 }
 visit(owner.body);required(changes===1,'mutation changed an unexpected number of literals');
 return source.replace(raw,JSON.stringify(module));
}
function runCommand(cmd,args,cwd,out,label,env={},timeout=300000){
 const r=spawnSync(cmd,args,{cwd,encoding:'utf8',timeout,maxBuffer:32*1024*1024,env:{...process.env,...env}});
 fs.writeFileSync(path.join(out,label+'.stdout'),r.stdout??'');fs.writeFileSync(path.join(out,label+'.stderr'),r.stderr??'');
 return r;
}
function passed(r,label){required(!r.error&&r.status===0&&!r.signal,`${label}: ${r.error??''} ${r.stdout??''} ${r.stderr??''}`);}
export function auditRun(just,source){
 required(/^vv:.*\bintegrity\b/m.test(just),'stream gate removed from vv');
 required(just.includes('    node --test scripts/integrity-stream.test.mjs\n    node scripts/integrity-stream.mjs\n'),'stream gate invocation changed');
 const body=source.slice(source.lastIndexOf('export function run(root){')).split('\nif(process.argv')[0];
 for(const marker of ["inventory(root);","for(const mutation of mutations){","plant(engineSource,mutation.id)","wrong limit mutation failure owner","terminal role mutation missed independent owner","'restored stream source'","leanDriver(root)","passed(vectors,'stream vectors');","92 generated-Lean stream fixtures; 276 partitions; actual compression counts; framing and cancellation cases passed"])
  required(body.includes(marker),'stream gate omitted '+marker);
 const conditions=body.match(/\bif\s*\(/g)??[];
 required(conditions.length===2&&body.includes("if(mutation.id==='production_limit'){")&&body.includes("if(/^(@\\[expose\\] )?public (def|theorem|inductive|structure) /.test(generated[i]))"),'conditional stream bypass');
 const bounded=body.replace("{end=i;break;}","{end=i;}");
 required(!/\b(switch|while|continue|break)\b|mutations\.(slice|filter|splice)/.test(bounded),'stream inventory filtered or skipped');
}
export function run(root){
 auditRun(fs.readFileSync(path.join(root,'Justfile'),'utf8'),fs.readFileSync(script,'utf8'));
 inventory(root);
 const out=fs.mkdtempSync(path.join(root,'.prism/uorc/stream.')),tmp=fs.mkdtempSync(path.join(os.tmpdir(),'uorc-stream-'));
 try{
  passed(runCommand('lexlean',['--project',path.join(root,'lexlean.toml'),'--diagnostic-format','json','verify',...sourcePaths],root,out,'source'),'source verify');
  const build=runCommand('lexlean',['--project',path.join(root,'lexlean.toml'),'--diagnostic-format','json','build',...sourcePaths],root,out,'build');passed(build,'build');
  const b=JSON.parse(build.stdout);required(b.success&&b.artifacts.length===1,'one stream build artifact');
  const modules=path.resolve(root,b.artifacts[0],'modules');required(modules.startsWith(path.resolve(root)+path.sep),'build escaped project');
  const nativeObjects=[];
  for(const n of sourceNames.slice(0,4)){
   const rel=`Uorc/Uorc/Integrity/${n}`,target=path.join(tmp,rel+'.lean');fs.mkdirSync(path.dirname(target),{recursive:true});fs.copyFileSync(path.join(modules,rel+'.lean'),target);
   passed(runCommand('lean',['-R',tmp,'-o',path.join(tmp,rel+'.olean'),'-c',path.join(tmp,rel+'.c'),target],root,out,'compile-'+n,{LEAN_PATH:tmp}),'compile '+n);
   const object=path.join(tmp,rel+'.o');passed(runCommand('leanc',['-O3','-c',path.join(tmp,rel+'.c'),'-o',object],root,out,'native-'+n),'native compile '+n);nativeObjects.push(object);
  }
  const engineFile='src/Uorc/Integrity/StreamEngine.lex.tex';
  const engineSource=fs.readFileSync(path.join(root,engineFile),'utf8');
  const mutations=records(root).filter(r=>r.type==='StreamMutation');required(mutations.length===2,'stream mutation inventory');
  for(const mutation of mutations){
   const project=path.join(tmp,mutation.id);fs.cpSync(root,project,{recursive:true,filter:p=>!path.relative(root,p).split(path.sep).some(p=>['.git','.lexlean','.prism','.lake','target','node_modules'].includes(p))});
   fs.writeFileSync(path.join(project,engineFile),plant(engineSource,mutation.id));
   const selected=['src/Uorc/Integrity/StreamMax.lex.tex','src/Uorc/Integrity/StreamDiagnostic.lex.tex'];
   const checked=runCommand('lexlean',['--project',path.join(project,'lexlean.toml'),'--diagnostic-format','json','verify',...selected],project,out,mutation.id);
   if(mutation.id==='production_limit'){
    required(!checked.error&&checked.status!==0&&checked.status!==null&&!checked.signal,'limit mutation did not complete with failure');
    const result=JSON.parse(checked.stdout);required(result.success===false,'limit mutation accepted');
    const builds=fs.readdirSync(path.join(project,'.lexlean/build'));required(builds.length===1,'ambiguous mutant build');
    const generated=fs.readFileSync(path.join(project,'.lexlean/build',builds[0],'modules/Uorc/Uorc/Integrity/StreamEngine.lean'),'utf8').split('\n');
    const start=generated.findIndex(l=>l.includes('public theorem production_limit_exact '));required(start>=0,'mutation theorem absent');
    let end=generated.length;for(let i=start+1;i<generated.length;i++)if(/^(@\[expose\] )?public (def|theorem|inductive|structure) /.test(generated[i])){end=i;break;}
    required(result.diagnostics.some(d=>d.code==='LLV7002'&&d.primary?.path===engineFile&&d.notes?.some(n=>{const m=n.message.match(/StreamEngine\.lean:(\d+):/);return m&&Number(m[1])-1>=start&&Number(m[1])-1<end;})),'wrong limit mutation failure owner');
   }else{
    passed(checked,'role mutant source');
    const built=runCommand('lexlean',['--project',path.join(project,'lexlean.toml'),'--diagnostic-format','json','build',...selected],project,out,mutation.id+'-build');passed(built,'role mutant build');
    const artifact=JSON.parse(built.stdout);const mr=path.join(project,'generated');
    for(const n of sourceNames.slice(0,4)){
     const rel=`Uorc/Uorc/Integrity/${n}`,target=path.join(mr,rel+'.lean');fs.mkdirSync(path.dirname(target),{recursive:true});fs.copyFileSync(path.join(project,artifact.artifacts[0],'modules',rel+'.lean'),target);
     passed(runCommand('lean',['-R',mr,'-o',path.join(mr,rel+'.olean'),target],project,out,mutation.id+'-'+n,{LEAN_PATH:mr}),'role mutant compile');
    }
    const driver=path.join(project,'Oracle.lean');fs.writeFileSync(driver,leanDriver(root));
    const failed=runCommand('lean',['-R',project,driver],project,out,mutation.id+'-runtime',{LEAN_PATH:mr});
    required(!failed.error&&failed.status!==0&&failed.status!==null&&!failed.signal&&(failed.stdout+failed.stderr).includes('frame_000_00 footer bytes'),'terminal role mutation missed independent owner');
   }
   fs.writeFileSync(path.join(project,engineFile),engineSource);
   passed(runCommand('lexlean',['--project',path.join(project,'lexlean.toml'),'--diagnostic-format','json','verify',...selected],project,out,mutation.id+'-restored'),'restored stream source');
  }
  const driver=path.join(tmp,'Oracle.lean');fs.writeFileSync(driver,leanDriver(root).replace('#eval StreamOracle.main','def main : IO Unit := StreamOracle.main'));
  passed(runCommand('lean',['-R',tmp,'-c',path.join(tmp,'Oracle.c'),driver],root,out,'oracle-native',{LEAN_PATH:tmp}),'native oracle compile');
  const executable=path.join(tmp,'stream-oracle');passed(runCommand('leanc',['-O3',path.join(tmp,'Oracle.c'),...nativeObjects,'-o',executable],root,out,'oracle-link'),'native oracle link');
  const vectors=runCommand(executable,[],root,out,'vectors');passed(vectors,'stream vectors');
  required(vectors.stdout.trim()==='92 generated-Lean stream fixtures; 276 partitions; actual compression counts; framing and cancellation cases passed','stream coverage omitted');
  fs.writeFileSync(path.join(out,'receipt.json'),JSON.stringify({schema:'uorc/stream-max-runtime/1',sources:Object.fromEntries(sourcePaths.map(p=>[p,crypto.createHash('sha256').update(fs.readFileSync(path.join(root,p))).digest('hex')])),build_id:b.build_id,cases:92,partitions:276,mutations:mutations.map(r=>({id:r.id,owner:r.owner})),scope:'source and compiled generated-Lean diagnostic execution; locked full SDK and production BinaryProgram acceptance remain separate'})+'\n');
  console.log(vectors.stdout.trim());console.log("2 stream source defects rejected; restored controls passed.");
 }finally{fs.rmSync(tmp,{recursive:true,force:true});}
}
if(process.argv[1]&&path.resolve(process.argv[1])===script){try{required(process.argv.length===2,'no arguments accepted');run(process.cwd());}catch(e){console.error(e.message);process.exitCode=1;}}
