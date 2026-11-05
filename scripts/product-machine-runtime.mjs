// Execute the generated, source-owned product/engine controls. No product codec or UI lives here.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {writeImageMutationSpecimens} from './product-write-image-mutations.mjs';
import {inspectionMutationSpecimens} from './product-inspection-mutations.mjs';
import {readControllerMutationSpecimens} from './product-read-controller-mutations.mjs';
import {outputDeliveryMutationSpecimens} from './product-output-delivery-mutations.mjs';
import {mutationTargets as semanticMutationTargets, mutateMachineSemanticMigration} from './product-machine-semantic-mutations.mjs';
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const requireFact=(b,m)=>{if(!b)throw new Error(m);};
const defaultSuite='MachineBindingScenarios';
function selection(suite){requireFact(['MachineBindingScenarios','ReadImageScenarios','WriteImageScenarios','ApplicationScenarios','InspectionReportScenarios','ReadControllerScenarios','OutputDeliveryScenarios'].includes(suite),'unknown typed runtime suite');return `src/Uorc/Product/${suite}.lex.tex`;}
function run(command,args,cwd,env={}){
 const r=spawnSync(command,args,{cwd,env:{...process.env,...env},encoding:'utf8',timeout:600000,maxBuffer:32*1024*1024});
 requireFact(!r.error&&!r.signal,`${command}: ${r.error??r.signal}`);return r;
}
function checked(command,args,cwd,env={}){const r=run(command,args,cwd,env);requireFact(r.status===0,`${command}: ${r.stdout}${r.stderr}`);return r;}
function filesBelow(root,suffix){return fs.readdirSync(root,{withFileTypes:true}).flatMap(e=>e.isDirectory()?filesBelow(path.join(root,e.name),suffix):e.name.endsWith(suffix)?[path.join(root,e.name)]:[]);}
export function driver(only,suite=defaultSuite){
 if(only)return `import Uorc.Uorc.Product.${suite}\nopen Uorc.Uorc.Product\ndef main : IO Unit := do\n if !${suite}.${only} then throw (IO.userError "FAIL ${only}")\n IO.println "PASS ${only}"\n`;
 return `import Uorc.Uorc.Product.${suite}\nopen Uorc.Uorc.Product\ndef main : IO Unit := do\n let mut passed := 0\n let mut total := 0\n let mut names : List String := []\n for observation in ${suite}.observations do\n  if names.contains observation.id then throw (IO.userError "duplicate observation")\n  names := observation.id :: names\n  total := total + 1\n  if observation.passed then passed := passed + 1 else IO.println s!"FAIL {observation.id}"\n IO.println s!"{passed}/{total} actual product-machine binding controls passed"\n if total == 0 || passed != total then throw (IO.userError "binding controls failed")\n`;
}
export function execute(root,only,runtimeLibrary,suite=defaultSuite){
 const selected=selection(suite);
 const r=checked('lexlean',['--diagnostic-format','json',runtimeLibrary?'build':'verify',selected],root);
 const result=JSON.parse(r.stdout);requireFact(result.success&&result.artifacts.length===1,'one actual generated closure required');
 const verified=path.resolve(root,result.artifacts[0]);requireFact(verified.startsWith(path.resolve(root)+path.sep),'artifact escaped project');
 const generated=path.join(verified,'modules'),runtime=fs.mkdtempSync(path.join(os.tmpdir(),'uorc-machine-runtime-'));
 try{
  const files=new Map(filesBelow(generated,'.lean').map(p=>[path.relative(generated,p).slice(0,-5).split(path.sep).join('.'),p]));
  const done=new Set(),active=new Set();const leanPath=[runtime,runtimeLibrary??path.join(verified,'oleans')].join(path.delimiter);
  function compile(name){
   if(done.has(name))return;requireFact(!active.has(name),'module cycle');active.add(name);
   const source=fs.readFileSync(files.get(name),'utf8');
   for(const m of source.matchAll(/^(?:public )?import (\S+)/gm))if(files.has(m[1]))compile(m[1]);
   const target=path.join(runtime,path.relative(generated,files.get(name)));fs.mkdirSync(path.dirname(target),{recursive:true});fs.writeFileSync(target,source);
   checked('lean',['-R',runtime,'-o',target.slice(0,-5)+'.olean',target],root,{LEAN_PATH:leanPath});active.delete(name);done.add(name);
  }
  for(const name of files.keys())compile(name);
  const harness=path.join(runtime,'RunProductMachine.lean');fs.writeFileSync(harness,driver(only,suite));
  const execution=run('lean',['--run',harness],root,{LEAN_PATH:leanPath});
  return {execution,source_id:result.source_id,semantic_id:result.semantic_id,build_id:result.build_id,attestation_id:result.attestation_id??null,runtime_library:runtimeLibrary??path.join(verified,'oleans'),modules:files.size,harness_sha256:sha(fs.readFileSync(harness))};
 }finally{fs.rmSync(runtime,{recursive:true,force:true});}
}
function mutate(root,module,transform,authoring){
 const p=path.join(root,`src/Uorc/Product/${module}.lex.tex`);const original=fs.readFileSync(p,'utf8');
 const data=authoring.read(p);transform(data.declarations);authoring.write(p,data);
 return ()=>fs.writeFileSync(p,original);
}
const sourceData={
 read(p){const m=/\\semanticdata\{(.*)\}/.exec(fs.readFileSync(p,'utf8'));requireFact(m,'source data required; a diagnostic notation projection must supply its official SDK authoring API');return JSON.parse(m[1]);},
 write(p,data){const text=fs.readFileSync(p,'utf8');fs.writeFileSync(p,text.replace(/\\semanticdata\{(.*)\}/,'\\semanticdata{'+JSON.stringify(data)+'}'));}
};
export function mutationSpecimens(){
 return [
   ['length-mismatch-stalled','OwnedCompare','lengthMismatchControl',ds=>{
    let count=0;const walk=x=>{if(!x||typeof x!=='object')return;if(x.kind==='constructor'&&x.constructor.name==='Disposition.lengthMismatch'){x.constructor.name='Disposition.working';x.arguments=[];count++;}else for(const v of Object.values(x))Array.isArray(v)?v.forEach(walk):walk(v);};walk(ds.find(d=>d.name==='fromReaders').body);requireFact(count===1,'one actual length mismatch mutation');
   }],
   ['byte-equality-forged','OwnedCompare','byteMismatchControl',ds=>{
    let count=0;const walk=x=>{if(!x||typeof x!=='object')return;if(x.kind==='if'&&x.condition.kind==='primitive'&&x.condition.operation==='equal'){x.condition={kind:'bool',value:true};count++;}for(const v of Object.values(x))Array.isArray(v)?v.forEach(walk):walk(v);};walk(ds.find(d=>d.name==='completeByte').body);requireFact(count===1,'one actual byte comparison mutation');
   }],
   ['returned-output-ticket-omitted','MachineBinding','outputCustodyControl',ds=>{const d=ds.find(d=>d.name==='addOutputRoot');d.body={kind:'var',name:'roots'};}],
   ['comparison-initialization-unpaid','ComparisonOwner','comparisonMismatchPaidInitialization',ds=>{ds.find(d=>d.name==='beginWork').body={kind:'integer',representation:'uint64',value:'0'};}],
   ['left-getter-prepayment-bypassed','OwnedCompare','getterRefusalResumeControl',ds=>{
    const body=ds.find(d=>d.name==='paidReadLeft').body;
    const event=body.scrutinee.arguments[2];event.fields.find(f=>f.field==='work').value={kind:'integer',representation:'uint64',value:'0'};
   }],
   ...semanticMutationTargets.map(({id,control})=>[id,'MachineBinding',control,ds=>{
    const mutated=mutateMachineSemanticMigration({declarations:ds},id);
    ds.splice(0,ds.length,...mutated.declarations);
   }])
  ];
}
export function imageMutationSpecimens(){return [
 ['payload-commit-prepayment-bypassed','ReadImage','commitPrepaymentControl',ds=>{
  ds.find(d=>d.name==='commitWork').body={kind:'constructor',constructor:{name:'Option.some'},arguments:[{kind:'integer',representation:'uint64',value:'0'}],type_arguments:[{kind:'uint64'}]};
 }],
 ['payload-image-ticket-transferred-twice','ReadImage','oneTimeTransferControl',ds=>{
  let count=0;const walk=x=>{if(!x||typeof x!=='object')return;if(x.kind==='record'&&x.type.name==='Transition'){
   const outcome=x.fields.find(f=>f.field==='outcome').value;
   if(outcome.kind==='constructor'&&outcome.constructor.name==='Outcome.transferred'){
    const cursor=x.fields.find(f=>f.field==='cursor').value;
    cursor.fields.find(f=>f.field==='image').value={kind:'project',value:{kind:'var',name:'s'},field:'image'};
    cursor.fields.find(f=>f.field==='lease').value={kind:'project',value:{kind:'var',name:'s'},field:'lease'};count++;
   }
  }for(const v of Object.values(x))Array.isArray(v)?v.forEach(walk):walk(v);};walk(ds.find(d=>d.name==='takeImage').body);requireFact(count===1,'one actual payload transfer mutation');
 }],
 ['payload-cancel-forgets-inflight-receipt','ReadImage','cancellationPendingControl',ds=>{
  let count=0;const walk=x=>{if(!x||typeof x!=='object')return;if(x.kind==='record'&&x.type.name==='Cursor'){
   x.fields.find(f=>f.field==='payloadPending').value={kind:'bool',value:false};count++;
  }for(const v of Object.values(x))Array.isArray(v)?v.forEach(walk):walk(v);};walk(ds.find(d=>d.name==='cancel').body);requireFact(count===1,'one actual payload cancellation mutation');
 }],
 ['payload-foreign-instance-accepted','ReadImage','wrongLeaseInstanceControl',ds=>{
  let count=0;const walk=x=>{if(!x||typeof x!=='object')return;if(x.kind==='if'&&x.condition.kind==='beq'&&x.condition.left?.kind==='project'&&x.condition.left.field==='instance'&&x.condition.left.value?.kind==='project'&&x.condition.left.value.field==='lease'){x.condition={kind:'bool',value:true};count++;}
   for(const v of Object.values(x))Array.isArray(v)?v.forEach(walk):walk(v);};walk(ds.find(d=>d.name==='leaseReady').body);requireFact(count===1,'one actual payload instance mutation');
 }]
];}
export function applicationMutationSpecimens(){return [
 ['frontend-stale-action-promoted','Application','staleControl',ds=>{
  ds.find(d=>d.name==='fromInvocation').body.arguments[2]={kind:'constructor',constructor:{name:'Disposition.applied'},arguments:[]};
 }],
 ['frontend-generated-focus-erased','Application','factoryControl',ds=>{
  ds.find(d=>d.name==='frame').body.fields.find(f=>f.field==='focusId').value={kind:'string',value:''};
 }],
 ['frontend-invalid-number-message-erased','Application','invalidNumberControl',ds=>{
  ds.find(d=>d.name==='fromNumber').body.arguments[3]={kind:'string',value:''};
 }]
];}
function suiteMutations(suite){
 if(suite===defaultSuite)return mutationSpecimens();
 if(suite==='ReadImageScenarios')return imageMutationSpecimens();
 if(suite==='WriteImageScenarios')return writeImageMutationSpecimens();
 if(suite==='ApplicationScenarios')return applicationMutationSpecimens();
 if(suite==='InspectionReportScenarios')return inspectionMutationSpecimens();
 if(suite==='ReadControllerScenarios')return readControllerMutationSpecimens();
 if(suite==='OutputDeliveryScenarios')return outputDeliveryMutationSpecimens();
 throw new Error('unregistered typed scenario suite');
}
export function validate(root,authoring=sourceData,suite=defaultSuite){
 const selected=selection(suite);
 const inventory=authoring.read(path.join(root,selected)).declarations.find(d=>d.name==='observations');
 let expected=0;for(let p=inventory.body;p.kind==='cons';p=p.tail)expected++;
 requireFact(expected>0,'empty source inventory');
 const restored=execute(root,undefined,undefined,suite);requireFact(restored.execution.status===0,restored.execution.stdout+restored.execution.stderr);
 requireFact(restored.execution.stdout.trim()===`${expected}/${expected} actual product-machine binding controls passed`,'incomplete source runtime inventory');
 const temp=fs.mkdtempSync(path.join(os.tmpdir(),'uorc-machine-mutation-'));const mutations=[];
 try{
  fs.cpSync(path.join(root,'src'),path.join(temp,'src'),{recursive:true});
  for(const name of ['lexlean.toml','lexlean.lock','lakefile.toml','lake-manifest.json','lean-toolchain'])fs.copyFileSync(path.join(root,name),path.join(temp,name));
  const specimens=suiteMutations(suite);
  requireFact(specimens.length>0,'empty mutation inventory');
  for(const [id,module,owner,transform] of specimens){
   const restore=mutate(temp,module,transform,authoring);
   try{const r=execute(temp,owner,restored.runtime_library,suite);requireFact(r.execution.status!==0&&(r.execution.stdout+r.execution.stderr).includes('FAIL '+owner),`planted ${id} did not fail at ${owner}: ${r.execution.stdout}${r.execution.stderr}`);mutations.push({id,owner,exit:r.execution.status});}finally{restore();}
  }
 }finally{fs.rmSync(temp,{recursive:true,force:true});}
 const out=path.join(root,'.prism/uorc');fs.mkdirSync(out,{recursive:true});
 fs.writeFileSync(path.join(out,({MachineBindingScenarios:'product-machine-receipt.json',ReadImageScenarios:'product-image-receipt.json',WriteImageScenarios:'product-write-image-receipt.json',ApplicationScenarios:'product-application-receipt.json',InspectionReportScenarios:'product-inspection-receipt.json',ReadControllerScenarios:'product-read-controller-receipt.json',OutputDeliveryScenarios:'product-output-delivery-receipt.json'})[suite]),JSON.stringify({spec:'uorc/product-machine-runtime/1',suite,scope:'Actual source linking, generated Lean kernel compilation, VM controls and owning planted negatives. Exact axiom audit, leanchecker, accepted production export, native library, browser, closed archive operation and full pinned SDK acceptance remain separate.',source_id:restored.source_id,semantic_id:restored.semantic_id,build_id:restored.build_id,attestation_id:restored.attestation_id,modules:restored.modules,total:expected,passed:expected,mutations,gate_sha256:sha(fs.readFileSync(fileURLToPath(import.meta.url))),mutation_helper_sha256:suite===defaultSuite?sha(fs.readFileSync(new URL('./product-machine-semantic-mutations.mjs',import.meta.url))):suite==='WriteImageScenarios'?sha(fs.readFileSync(new URL('./product-write-image-mutations.mjs',import.meta.url))):suite==='InspectionReportScenarios'?sha(fs.readFileSync(new URL('./product-inspection-mutations.mjs',import.meta.url))):suite==='ReadControllerScenarios'?sha(fs.readFileSync(new URL('./product-read-controller-mutations.mjs',import.meta.url))):suite==='OutputDeliveryScenarios'?sha(fs.readFileSync(new URL('./product-output-delivery-mutations.mjs',import.meta.url))):null,harness_sha256:restored.harness_sha256})+'\n');
 console.log(restored.execution.stdout.trim());console.log(`${mutations.length} actual product-machine defects rejected`);
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)){try{requireFact(process.argv.length===2,'no arguments accepted');validate(process.cwd());}catch(e){console.error(e.message);process.exitCode=1;}}
