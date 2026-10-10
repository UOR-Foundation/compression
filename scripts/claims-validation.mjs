// SDK process orchestration. Mutation intent and expected proof owners are LexLean data.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {copySource, sha, canonical} from './offline-validation.mjs';

const script = fileURLToPath(import.meta.url);
const sourcePath = 'src/Uorc/Claims.lex.tex';
function requireFact(condition, message) { if (!condition) throw new Error(`UC-HON-01: ${message}`); }
export function auditGate(just, harness) {
  requireFact(just.split('\n').filter(line=>/^vv:/.test(line)).length===1 && /^vv:.*\bclaim-dispositions\b/m.test(just), 'required gate omitted');
  requireFact(just.split('\n').filter(line=>/^claim-dispositions:/.test(line)).length===1 && /^claim-dispositions:\n    node scripts\/claims-validation\.mjs\n(?:\n|$)/m.test(just), 'mutation command altered or conditional');
  const runBody=harness.slice(harness.lastIndexOf('export function run(root)')).split('\nif(process.argv')[0];
  for(const required of ['const cases = mutations(source);', 'for(const mutation of cases)', 'expectedFailure(result,mutation,generated);', "verified.modules.includes('Uorc.Claims')", 'receipts.length===cases.length']) requireFact(runBody.includes(required), `harness omitted ${required}`);
  requireFact(!/\b(if|switch|while|return|continue|break)\b|cases\.(filter|slice|splice)|process\.env/.test(runBody.replace('return result;','')), 'mutation execution is filtered or conditional');
}
export function mutations(source) {
  const matches = [...source.matchAll(/\\semanticdata\{(.*)\}\n/g)];
  requireFact(matches.length === 1, 'expected one canonical source module');
  const module = JSON.parse(matches[0][1]);
  const rows = module.declarations.filter(d => d.kind === 'definition' && d.result?.member?.name === 'ClaimMutation').map(d => {
    requireFact(d.body.kind === 'record', 'mutation is not a source record');
    const fields = d.body.fields;
    requireFact(fields.length === 5 && new Set(fields.map(f=>f.field)).size === 5, 'mutation field inventory differs');
    const row = Object.fromEntries(fields.map(f => {requireFact(f.value.kind === 'string','mutation field is not a string');return [f.field,f.value.value];}));
    requireFact(Object.keys(row).sort().join() === ['id','definition','field','replacement','owner'].sort().join(), 'unexpected mutation fields');
    requireFact(Object.values(row).every(v=>/^[A-Za-z][A-Za-z0-9_]*$/.test(v)), 'invalid mutation identifier');
    requireFact(module.declarations.filter(d=>d.kind==='theorem'&&d.name===row.owner).length===1,'mutation has no unique owning theorem');
    return row;
  });
  requireFact(rows.length > 0 && new Set(rows.map(r=>r.id)).size === rows.length, 'empty/duplicate mutation inventory');
  return rows;
}
export function plant(source, mutation) {
  const payload = source.match(/\\semanticdata\{(.*)\}\n/)[1];
  const module = JSON.parse(payload);
  const declarations = module.declarations.filter(d=>d.kind==='definition'&&d.name===mutation.definition);
  requireFact(declarations.length===1, 'mutation definition is not unique');
  const fields = declarations[0].body.fields.filter(f=>f.field===mutation.field);
  requireFact(fields.length===1,'mutation field is not unique');
  const value = fields[0].value;
  if(value.kind==='constructor') value.constructor.name = value.constructor.name.replace(/[^.]+$/,mutation.replacement);
  else if(value.kind==='string') value.value = mutation.replacement;
  else requireFact(false, 'unsupported source-owned mutation value');
  const result = source.replace(payload,JSON.stringify(module));
  requireFact(result!==source,'mutation changed no source');
  return result;
}
export function expectedFailure(result, mutation, generated) {
  requireFact(!result.error && result.status !== null && result.status !== 0 && !result.signal, 'mutation did not produce a completed failure');
  const value = JSON.parse(result.stdout);
  requireFact(value.spec==='lexlean/command-result/1' && value.success===false, 'failure is not an SDK diagnostic result');
  const lines = generated.split('\n');
  const start = lines.findIndex(line=>line.startsWith(`public theorem ${mutation.owner} :`));
  requireFact(start>=0 && lines.filter(line=>line.startsWith(`public theorem ${mutation.owner} :`)).length===1, 'generated theorem owner is absent or ambiguous');
  let end=start+1;
  while(end<lines.length && !/^(public |end\b)/.test(lines[end]))end++;
  requireFact((value.diagnostics??[]).some(d=>d.code==='LLV7002' && d.primary?.path===sourcePath && d.message.includes('Tactic `rfl` failed') && (d.notes??[]).some(note=>{
    const match=note.message.match(/^generated location: \$STAGING\/lean-src\/Uorc\/Uorc\/Claims\.lean:(\d+):\d+$/);
    return match && Number(match[1])>start && Number(match[1])<=end;
  })), 'failure did not reach its owning kernel theorem');
}
export function run(root) {
  auditGate(fs.readFileSync(path.join(root,'Justfile'),'utf8'),fs.readFileSync(script,'utf8'));
  const source = fs.readFileSync(path.join(root,sourcePath),'utf8');
  const cases = mutations(source);
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(),'uorc-claim-mutations-'));
  fs.mkdirSync(path.join(root,'.prism/uorc'),{recursive:true});
  const output = fs.mkdtempSync(path.join(root,'.prism/uorc/claims.'));
  const receipts=[];
  const execute=(project,label,command='verify')=>{
    const args=['--project',path.join(project,'lexlean.toml'),'--diagnostic-format','json',command,'--all'];
    const result=spawnSync('/usr/local/bin/lexlean',args,{cwd:project,encoding:'utf8',timeout:300000,maxBuffer:32*1024*1024});
    fs.writeFileSync(path.join(output,`${label}.stdout`),result.stdout??'');
    fs.writeFileSync(path.join(output,`${label}.stderr`),result.stderr??'');
    return result;
  };
  try {
    for(const mutation of cases) {
      const project=path.join(temporary,mutation.id);copySource(root,project);
      const mutant=plant(source,mutation);fs.writeFileSync(path.join(project,sourcePath),mutant);
      const built=execute(project,`${mutation.id}-build`,'build');
      requireFact(!built.error && built.status===0,'mutant did not build before kernel rejection');
      const build=JSON.parse(built.stdout);
      requireFact(build.success===true && /^[a-f0-9]{64}$/.test(build.build_id),'invalid current mutant build');
      const generated=fs.readFileSync(path.join(project,'.lexlean/build',build.build_id,'modules/Uorc/Uorc/Claims.lean'),'utf8');
      fs.writeFileSync(path.join(output,`${mutation.id}.generated.lean`),generated);
      const result=execute(project,mutation.id);expectedFailure(result,mutation,generated);
      fs.writeFileSync(path.join(project,sourcePath),source);
      fs.rmSync(path.join(project,'.lexlean'),{recursive:true,force:true});
      fs.rmSync(path.join(project,'.lake'),{recursive:true,force:true});
      const restored=execute(project,`${mutation.id}-restored`);
      requireFact(!restored.error && restored.status===0 && !restored.signal,'restored source failed verification');
      const verified=JSON.parse(restored.stdout);
      requireFact(verified.success===true && verified.exit_code===0 && verified.modules.includes('Uorc.Claims'),'restored run omitted the claim module');
      receipts.push({...mutation,source_sha256:sha(source),mutant_sha256:sha(mutant),generated_sha256:sha(generated),mutant_build_id:build.build_id,exit_code:result.status,diagnostic:'LLV7002',restored_exit_code:restored.status,restored_build_id:verified.build_id,restored_attestation_id:verified.attestation_id,stdout_sha256:sha(result.stdout),restored_stdout_sha256:sha(restored.stdout)});
    }
    requireFact(receipts.length===cases.length,'mutation execution omitted a source case');
    fs.writeFileSync(path.join(output,'mutations.json'),canonical({schema:'uorc/claim-mutations/1',command:['lexlean','verify','--all'],cases:receipts})+'\n');
    console.log(`${receipts.length} source-owned evidence promotions rejected by their exact kernel theorems; every restored control passed.`);
  } finally {fs.rmSync(temporary,{recursive:true,force:true});}
}
if(process.argv[1] && path.resolve(process.argv[1])===script) {
  try {requireFact(process.argv.length===2,'no harness arguments are accepted');run(process.cwd());}
  catch(error){console.error(error.message);process.exitCode=1;}
}
