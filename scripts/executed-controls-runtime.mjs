// Concrete whole-machine control vectors (archive trials, replay, certification,
// Huffman packing, step-machine partitions) are executed as generated native
// code rather than reduced by the Lean elaborator: each control is a Boolean
// definition with its exact expected value, and every one must pass.
import fs from 'node:fs';
import path from 'node:path';
import {controlModules, execute} from './product-machine-runtime.mjs';
import {dataForm} from './semantic-source.mjs';
const requireFact=(b,m)=>{if(!b)throw new Error(m);};
function inventory(root,module){
 const m=/\\semanticdata\{(.*)\}/.exec(dataForm(fs.readFileSync(path.join(root,`src/Uorc/${module}.lex.tex`),'utf8')));
 requireFact(m,`source data required for ${module}`);
 const declaration=JSON.parse(m[1]).declarations.find(d=>d.name==='observations');
 requireFact(declaration,`${module} declares no observations`);
 let count=0;for(let p=declaration.body;p.kind==='cons';p=p.tail)count++;
 requireFact(count>0,`${module} has an empty observation inventory`);
 return count;
}
try{
 requireFact(process.argv.length===2,'no arguments accepted');
 const root=process.cwd();
 for(const module of controlModules){
  const expected=inventory(root,module);
  const {execution}=execute(root,undefined,undefined,module);
  const output=(execution.stdout+execution.stderr).trim();
  requireFact(execution.status===0&&execution.stdout.trim()===`${expected}/${expected} actual product-machine binding controls passed`,`${module}: ${output}`);
  console.log(`${module}: ${expected}/${expected} executed controls passed`);
 }
}catch(error){console.error(error.message);process.exitCode=1;}
