// Targeted source-owned runtime/mutation evidence; this never sets release acceptance.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {copySource,sha,canonical} from './offline-validation.mjs';

const script=fileURLToPath(import.meta.url);
const parser='src/Uorc/Wire/ProgramParse.lex.tex';
const scenarios='src/Uorc/Wire/ProgramParseScenarios.lex.tex';
const owner='Uorc.Uorc.Wire.ProgramParseScenarios';
const requireFact=(condition,message)=>{if(!condition)throw new Error(`UC-WIRE-PARSE: ${message}`);};
function execute(root,command,args,env={}) {
  const r=spawnSync(command,args,{cwd:root,env:{...process.env,...env},encoding:'utf8',timeout:300000,maxBuffer:32*1024*1024});
  requireFact(!r.error&&r.status!==null&&!r.signal,`${command} did not complete`);return r;
}
function verify(root) {
  const r=execute(root,'/usr/local/bin/lexlean',['--diagnostic-format','json','verify',scenarios]);
  requireFact(r.status===0,`exact-source verification failed: ${r.stdout}${r.stderr}`);
  const result=JSON.parse(r.stdout);
  requireFact(result.success===true&&result.modules.includes('Uorc.Wire.ProgramParse')&&result.modules.includes('Uorc.Wire.ProgramParseScenarios'),'verification omitted the parser or scenario owner');
  const artifacts=(result.artifacts??[]).map(p=>path.resolve(root,p));
  const roots=artifacts.map(p=>path.join(p,'oleans')).filter(p=>fs.existsSync(path.join(p,'Uorc/Uorc/Wire/ProgramParseScenarios.olean')));
  requireFact(roots.length===1,'verified scenario olean root is missing or ambiguous');
  return {result,oleanRoot:roots[0]};
}
function harness(root,verified,temporary,gate) {
  requireFact(['run','runTransferOwnership'].includes(gate),'unregistered runtime owner');
  const source=`import ${owner}\nopen ${owner}\ndef main : IO UInt32 :=\n  match ${gate} with\n  | .ok _ => do IO.println "${gate}: pass"; return 0\n  | .error .ownership => do IO.println "${gate}: ownership"; return 1\n  | .error _ => do IO.println "${gate}: wrong failure"; return 2\n`;
  const file=path.join(temporary,`${gate}.lean`);fs.writeFileSync(file,source);
  return execute(temporary,'lean',['--run',file],{LEAN_PATH:verified.oleanRoot});
}
export function plant(source) {
  const start=source.indexOf('  name: acknowledgePaid,');
  requireFact(start>=0&&source.indexOf('  name: acknowledgePaid,',start+1)<0,'missing/duplicate mutation target');
  const end=source.indexOf('\ndefinition{',start);
  requireFact(end>start,'mutation target has no closed definition boundary');
  const body=source.slice(start,end);
  const pattern=/\(Uorc\.Core\.Heap::retain \(project x arena\) \(project stLocal[0-9]+ pendingImage\)\)/g;
  requireFact([...body.matchAll(pattern)].length===1,'missing/duplicate admission retain');
  const mutant=body.replace(pattern,'(Uorc.Core.Heap::retain (project x arena) (Uorc.Core.RuntimeValue::nullHandle))');
  return source.slice(0,start)+mutant+source.slice(end);
}
export function run(root) {
  requireFact(process.argv.length===2,'no arguments or reduced gate modes accepted');
  requireFact(fs.existsSync('/opt/prismpm/share/inventory.json'),'the digest-selected SDK is required');
  const lock=execute(root,'/usr/local/bin/prismpm',['--json','--project',root,'lock','check']);
  requireFact(lock.status===0,'SDK lock check failed');
  const source=fs.readFileSync(path.join(root,parser),'utf8');
  const scenarioSource=fs.readFileSync(path.join(root,scenarios),'utf8');
  requireFact(scenarioSource.includes('name: runTransferOwnership,')&&scenarioSource.includes('name: run,'),'typed runtime owners are missing');
  const temporary=fs.mkdtempSync(path.join(os.tmpdir(),'uorc-program-parser-'));
  try {
    const baseline=verify(root),green=harness(root,baseline,temporary,'run');
    requireFact(green.status===0&&green.stdout==='run: pass\n','baseline runtime scenarios failed');
    const project=path.join(temporary,'mutant');copySource(root,project);
    const mutant=plant(source);fs.writeFileSync(path.join(project,parser),mutant);
    const mutated=verify(project),red=harness(project,mutated,temporary,'runTransferOwnership');
    requireFact(red.status===1&&red.stdout==='runTransferOwnership: ownership\n','missing-retain defect was not rejected by its intended typed ownership owner');
    fs.writeFileSync(path.join(project,parser),source);
    fs.rmSync(path.join(project,'.lexlean'),{recursive:true,force:true});
    fs.rmSync(path.join(project,'.lake'),{recursive:true,force:true});
    const restored=verify(project),restoredGreen=harness(project,restored,temporary,'run');
    requireFact(restoredGreen.status===0&&restoredGreen.stdout==='run: pass\n','restored control failed');
    const output=path.join(root,'.prism/uorc/program-parser');fs.mkdirSync(output,{recursive:true});
    fs.writeFileSync(path.join(output,'runtime.json'),canonical({schema:'uorc/program-parser-runtime/1',source_sha256:sha(source),scenario_sha256:sha(scenarioSource),baseline:baseline.result,mutation:{id:'missing_admission_ticket_retain',owner:'Uorc.Wire.ProgramParseScenarios.runTransferOwnership',mutant_sha256:sha(mutant),exit_code:red.status,stdout:red.stdout},restored:restored.result,release_acceptance:false})+'\n');
    console.log('Typed parser scenarios passed; missing admission retain rejected by runTransferOwnership; restored control passed. Targeted evidence only.');
  } finally {fs.rmSync(temporary,{recursive:true,force:true});}
}
if(process.argv[1]&&path.resolve(process.argv[1])===script){try{run(process.cwd());}catch(error){console.error(error.message);process.exitCode=1;}}
