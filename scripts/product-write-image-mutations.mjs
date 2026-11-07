const requireFact=(ok,message)=>{if(!ok)throw new Error(message);};
const uint64=value=>({kind:'integer',representation:'uint64',value:String(value)});
const bool=value=>({kind:'bool',value});
const declaration=(ds,name)=>{const d=ds.find(x=>x.name===name);requireFact(d,`missing actual declaration ${name}`);return d;};
function walk(value,transform){
 if(!value||typeof value!=='object')return;
 transform(value);
 for(const child of Object.values(value))Array.isArray(child)?child.forEach(v=>walk(v,transform)):walk(child,transform);
}
export function writeImageMutationSpecimens(){return [
 ['output-control-prepayment-bypassed','WriteImage','controlPrepaymentControl',ds=>{
  declaration(ds,'controlWork').body=uint64(0);
 }],
 ['output-copy-prepayment-bypassed','WriteImage','copyPrepaymentControl',ds=>{
  declaration(ds,'materializationWork').body={kind:'constructor',constructor:{name:'Result.ok'},arguments:[uint64(0)],type_arguments:[{kind:'uint64'},{kind:'named',member:{module:'Uorc.Core.RuntimeImage',name:'ImageFailure'},arguments:[]}]};
 }],
 ['output-mutable-attachment-accepted','WriteImage','immutableAttachmentControl',ds=>{
  declaration(ds,'immutableAttachment').body=bool(true);
 }],
 ['output-cancellation-latch-forgotten','WriteImage','cancellationLatchControl',ds=>{
  let count=0;
  walk(declaration(ds,'cancel').body,value=>{
   if(value.kind==='record'&&value.type?.name==='Cursor'){
    const field=value.fields.find(f=>f.field==='cancelled');requireFact(field,'missing actual cancellation field');field.value=bool(false);count++;
   }
  });
  requireFact(count===1,'one actual output cancellation mutation');
 }],
 ['output-stale-same-run-receipt-accepted','WriteImage','staleSameRunReceiptControl',ds=>{
  let count=0;
  walk(declaration(ds,'acknowledgeDelta').body,value=>{
   if(value.kind==='primitive'&&value.operation==='equal'&&value.arguments.some(argument=>argument.kind==='project'&&argument.field==='receiptSerial')){
    for(const key of Object.keys(value))delete value[key];Object.assign(value,bool(true));count++;
   }
  });
  requireFact(count===1,'one actual output receipt-serial mutation');
 }]
];}
