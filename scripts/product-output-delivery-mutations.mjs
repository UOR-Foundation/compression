const requireFact=(ok,message)=>{if(!ok)throw new Error(message);};
const variable=name=>({kind:'var',name});
const project=(value,field)=>({kind:'project',value,field});
const record=(module,name,fields)=>({kind:'record',type:{module,name},fields:Object.entries(fields).map(([field,value])=>({field,value}))});
const declaration=(ds,name)=>{const d=ds.find(x=>x.name===name);requireFact(d,`missing actual declaration ${name}`);return d;};
function walk(value,transform){if(!value||typeof value!=='object')return;transform(value);for(const child of Object.values(value))Array.isArray(child)?child.forEach(v=>walk(v,transform)):walk(child,transform);}
// These source-derived fields construct only an intentionally forged verification flag.
// They implement no delivery behavior; actual typed compilation rejects API drift.
const controlFields=["instance", "session", "stream", "input", "request", "report", "progress", "staged", "publicationPending", "transferred", "error", "hostFailure", "checkpoint", "attached", "closed", "page", "cleanup", "jobRetained", "storage", "lastAccounting", "retired"];
const streamFields=["phase", "ended", "producedBytes", "verified", "publication"];
export function outputDeliveryMutationSpecimens(){return [
 ['output-report-stale-run-adopted','OutputDelivery','staleSettlementRetainsActualOwnerControl',ds=>{declaration(ds,'eligible').body={kind:'bool',value:true};}],
 ['output-report-work-addition-unchecked','OutputDelivery','checkedWorkAdditionControl',ds=>{declaration(ds,'checkedLaneAdd').body={kind:'constructor',constructor:{name:'Result.ok'},arguments:[{kind:'nat',value:'0'}],type_arguments:[{kind:'nat'},{kind:'named',member:{name:'Error'},arguments:[]}]};}],
 ['output-report-storage-adoption-omitted','OutputDelivery','liveStorageAdoptedControl',ds=>{declaration(ds,'adoptWriter').body=record('Uorc.Product.Control','Transition',{state:variable('current'),effects:{kind:'nil',element:{kind:'named',member:{module:'Uorc.Product.Control',name:'Effect'},arguments:[]}}});}],
 ['output-report-failed-write-forges-rollback','OutputDelivery','failureDoesNotInventNotPublishedControl',ds=>{
  let count=0;walk(declaration(ds,'reported').body,value=>{if(value.kind==='record'&&value.type?.name==='FailureReport'){value.fields.find(f=>f.field==='publication').value={kind:'constructor',constructor:{module:'Uorc.Product.StreamProtocol',name:'Publication.notPublished'},arguments:[]};count++;}});requireFact(count===3,'three actual report-publication substitutions');
 }],
 ['output-report-writer-forges-verification','OutputDelivery','successfulSettlementIsNotArchiveProofControl',ds=>{
  let count=0;walk(declaration(ds,'continued').body,value=>{
   if(value.kind==='record'&&value.type?.name==='Step'){
    const field=value.fields.find(f=>f.field==='current'),original=field.value,stream=project(original,'stream');
    const changedStream=record('Uorc.Product.StreamProtocol','State',Object.fromEntries(streamFields.map(name=>[name,name==='verified'?{kind:'bool',value:true}:project(stream,name)])));
    field.value=record('Uorc.Product.Control','State',Object.fromEntries(controlFields.map(name=>[name,name==='stream'?changedStream:project(original,name)])));count++;
   }
  });requireFact(count===2,'two actual continued/retained verification substitutions');
 }]
];}
