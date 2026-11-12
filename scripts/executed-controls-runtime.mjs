// Concrete whole-machine control vectors (archive trials, replay, certification,
// Huffman packing, step-machine partitions) are executed as generated native
// code rather than reduced by the Lean elaborator: each control is a Boolean
// definition with its exact expected value, and every one must pass. One
// generated build covers every control module; its closure is compiled once.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {controlModules, driver} from './executed-controls.mjs';
import {dataForm, stageRuntime} from './semantic-source.mjs';
const requireFact=(b,m)=>{if(!b)throw new Error(m);};
function run(command,args,cwd,env={}){
 const r=spawnSync(command,args,{cwd,env:{...process.env,...env},encoding:'utf8',timeout:3600000,maxBuffer:64*1024*1024});
 requireFact(!r.error&&!r.signal,`${command}: ${r.error??r.signal}`);return r;
}
function checked(command,args,cwd,env={}){const r=run(command,args,cwd,env);requireFact(r.status===0,`${command}: ${r.stdout}${r.stderr}`);return r;}
function filesBelow(root,suffix){return fs.readdirSync(root,{withFileTypes:true}).flatMap(e=>e.isDirectory()?filesBelow(path.join(root,e.name),suffix):e.name.endsWith(suffix)?[path.join(root,e.name)]:[]);}
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
 const expected=new Map(controlModules.map(m=>[m,inventory(root,m)]));
 const built=JSON.parse(checked('lexlean',['--diagnostic-format','json','build',...controlModules.map(m=>`src/Uorc/${m}.lex.tex`)],root).stdout);
 requireFact(built.success&&built.artifacts.length===1,'one generated closure required');
 const generated=path.join(path.resolve(root,built.artifacts[0]),'modules');
 const runtime=fs.mkdtempSync(path.join(os.tmpdir(),'uorc-executed-controls-'));
 try{
  stageRuntime(runtime);
  const files=new Map(filesBelow(generated,'.lean').map(p=>[path.relative(generated,p).slice(0,-5).split(path.sep).join('.'),p]));
  const done=new Set(),active=new Set();
  const compile=name=>{
   if(done.has(name))return;requireFact(!active.has(name),'module cycle');active.add(name);
   const source=fs.readFileSync(files.get(name),'utf8');
   for(const m of source.matchAll(/^(?:public )?import (\S+)/gm))if(files.has(m[1]))compile(m[1]);
   const target=path.join(runtime,path.relative(generated,files.get(name)));fs.mkdirSync(path.dirname(target),{recursive:true});fs.writeFileSync(target,source);
   checked('lean',['-R',runtime,'-o',target.slice(0,-5)+'.olean',target],root,{LEAN_PATH:runtime});active.delete(name);done.add(name);
  };
  for(const name of files.keys())compile(name);
  for(const [module,count] of expected){
   const harness=path.join(runtime,`Run${module.replace('/','')}.lean`);fs.writeFileSync(harness,driver(undefined,module));
   const execution=run('lean',['--run',harness],root,{LEAN_PATH:runtime});
   requireFact(execution.status===0&&execution.stdout.trim()===`${count}/${count} executed controls passed`,`${module}: ${execution.stdout}${execution.stderr}`);
   console.log(`${module}: ${count}/${count} executed controls passed`);
  }
 }finally{fs.rmSync(runtime,{recursive:true,force:true});}
}catch(error){console.error(error.message);process.exitCode=1;}
