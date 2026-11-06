// Targeted source-owned runtime evidence. Complete acceptance remains `just vv`.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {copySource,sha,canonical} from './offline-validation.mjs';

const script=fileURLToPath(import.meta.url);
const program='src/Uorc/Wire/ProgramEmit.lex.tex';
const owners=['ProgramEmitScenarios','FrameEmitScenarios'];
const requireFact=(condition,message)=>{if(!condition)throw new Error(`UC-WIRE-EMIT: ${message}`);};
function execute(root,command,args,env={}) {
  const r=spawnSync(command,args,{cwd:root,env:{...process.env,...env},encoding:'utf8',timeout:300000,maxBuffer:32*1024*1024});
  requireFact(!r.error&&r.status!==null&&!r.signal,`${command} did not complete`);return r;
}
function verify(root,name) {
  const r=execute(root,'/usr/local/bin/lexlean',['--diagnostic-format','json','verify',`src/Uorc/Wire/${name}.lex.tex`]);
  requireFact(r.status===0,`exact-source verification failed: ${r.stdout}${r.stderr}`);
  const result=JSON.parse(r.stdout);
  requireFact(result.success===true&&result.modules.includes('Uorc.Wire.ProgramEmit')&&result.modules.includes(`Uorc.Wire.${name}`),'verification omitted a required owner');
  const roots=(result.artifacts??[]).map(p=>path.join(path.resolve(root,p),'oleans')).filter(p=>fs.existsSync(path.join(p,`Uorc/Uorc/Wire/${name}.olean`)));
  requireFact(roots.length===1,'verified scenario olean root is missing or ambiguous');
  return {result,oleanRoot:roots[0]};
}
function harness(verified,temporary,name,gate) {
  requireFact(owners.includes(name)&&['run','runOpcodes'].includes(gate),'unregistered runtime owner');
  const file=path.join(temporary,`${name}-${gate}.lean`);
  fs.writeFileSync(file,`import Uorc.Uorc.Wire.${name}\nopen Uorc.Uorc.Wire.${name}\ndef main : IO UInt32 :=\n  match ${gate} with\n  | .ok _ => do IO.println "${gate}: pass"; return 0\n  | .error .mismatch => do IO.println "${gate}: mismatch"; return 1\n  | .error _ => do IO.println "${gate}: wrong failure"; return 2\n`);
  return execute(temporary,'lean',['--run',file],{LEAN_PATH:verified.oleanRoot});
}
export function plant(source) {
  const start=source.indexOf('  name: wireTag,');
  requireFact(start>=0&&source.indexOf('  name: wireTag,',start+1)<0,'missing/duplicate opcode mapping');
  const end=source.indexOf('\ndefinition{',start);
  requireFact(end>start,'mapping has no closed definition boundary');
  const body=source.slice(start,end);
  const pattern=/Uorc\.Core\.RuntimeOpcode::Opcode\.naturalConstant => \(0 : UInt8\)/g;
  requireFact([...body.matchAll(pattern)].length===1,'missing/duplicate natural wire tag');
  return source.slice(0,start)+body.replace(pattern,'Uorc.Core.RuntimeOpcode::Opcode.naturalConstant => (1 : UInt8)')+source.slice(end);
}
export function run(root) {
  requireFact(process.argv.length===2,'no arguments or reduced gate modes accepted');
  requireFact(fs.existsSync('/opt/prismpm/share/inventory.json'),'the digest-selected SDK is required');
  const lock=execute(root,'/usr/local/bin/prismpm',['--json','--project',root,'lock','check']);
  requireFact(lock.status===0,'SDK lock check failed');
  const source=fs.readFileSync(path.join(root,program),'utf8');
  const temporary=fs.mkdtempSync(path.join(os.tmpdir(),'uorc-wire-emitter-'));
  try {
    const baseline=[];
    for(const owner of owners) {
      const verified=verify(root,owner),green=harness(verified,temporary,owner,'run');
      requireFact(green.status===0&&green.stdout==='run: pass\n',`${owner} runtime scenarios failed`);
      baseline.push({owner,result:verified.result});
    }
    const project=path.join(temporary,'mutant');copySource(root,project);
    const mutant=plant(source);fs.writeFileSync(path.join(project,program),mutant);
    const mutated=verify(project,'ProgramEmitScenarios'),red=harness(mutated,temporary,'ProgramEmitScenarios','runOpcodes');
    requireFact(red.status===1&&red.stdout==='runOpcodes: mismatch\n','wrong opcode tag did not fail at the intended byte-comparison owner');
    fs.writeFileSync(path.join(project,program),source);
    fs.rmSync(path.join(project,'.lexlean'),{recursive:true,force:true});
    fs.rmSync(path.join(project,'.lake'),{recursive:true,force:true});
    const restored=[];
    for(const owner of owners) {
      const verified=verify(project,owner),green=harness(verified,temporary,owner,'run');
      requireFact(green.status===0&&green.stdout==='run: pass\n',`${owner} restored control failed`);
      restored.push({owner,result:verified.result});
    }
    const output=path.join(root,'.prism/uorc/wire-emitter');fs.mkdirSync(output,{recursive:true});
    fs.writeFileSync(path.join(output,'runtime.json'),canonical({schema:'uorc/wire-emitter-runtime/1',source_sha256:sha(source),baseline,mutation:{id:'wrong_natural_opcode_tag',owner:'Uorc.Wire.ProgramEmitScenarios.runOpcodes',mutant_sha256:sha(mutant),exit_code:red.status,stdout:red.stdout},restored,release_acceptance:false})+'\n');
    console.log('Typed program/frame emitter scenarios passed; wrong opcode tag rejected by runOpcodes; restored controls passed. Targeted evidence only.');
  } finally {fs.rmSync(temporary,{recursive:true,force:true});}
}
if(process.argv[1]&&path.resolve(process.argv[1])===script){try{run(process.cwd());}catch(error){console.error(error.message);process.exitCode=1;}}
