// Native assertion harness: the implementation comes exclusively from generated crates.
import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';
const [bundle, target] = process.argv.slice(2);
if (!bundle || !target || !path.isAbsolute(bundle) || !path.isAbsolute(target))
  throw Error('pass absolute bundle and new test-output paths');
await fs.mkdir(target, {recursive:true});
if ((await fs.readdir(target)).length) throw Error('test-output directory must be empty');
async function manifests(root) {
  const found=[];
  for (const entry of await fs.readdir(root,{withFileTypes:true})) {
    if(entry.isSymbolicLink()) throw Error('unexpected symlink in generated bundle');
    const full=path.join(root,entry.name);
    if(entry.isDirectory()) found.push(...await manifests(full));
    else if(entry.name==='Cargo.toml') found.push(full);
  }
  return found;
}
const expected = new Set(['uorc_heaptests_suite', 'uorc_runtimetabletests_suite']);
const receipts=[];
for (const manifest of await manifests(bundle)) {
  const toml=await fs.readFile(manifest,'utf8');
  const name=/^name\s*=\s*"([^"]+)"/m.exec(toml)?.[1];
  if (!expected.has(name)) continue;
  expected.delete(name);
  const crate=path.dirname(manifest), out=path.join(target,name);
  await fs.cp(crate,out,{recursive:true});
  const original=await fs.readFile(path.join(crate,'src/lib.rs'));
  const copy=await fs.readFile(path.join(out,'src/lib.rs'));
  if (!original.equals(copy)) throw Error('generated implementation copy differs');
  const source=copy.toString('utf8');
  const assertion=/pub fn run\(\) -> bool\s*\{/.test(source) ? 'assert!(run());' : 'assert!(matches!(run(), Ok(true)));';
  await fs.mkdir(path.join(out,'tests'),{recursive:true});
  await fs.writeFile(path.join(out,'tests/ownership.rs'),`use ${name}::run;\n#[test]\nfn source_owned_ownership_suite() { ${assertion} }\n`);
  for (const args of [['generate-lockfile','--offline'],['test','--offline','--locked']]) {
    const result=spawnSync('cargo',args,{cwd:out,encoding:'utf8',maxBuffer:16*1024*1024});
    await fs.writeFile(path.join(out,args[0]+'.log'),(result.stdout??'')+(result.stderr??''));
    if (result.status!==0) throw Error(`${name}: cargo ${args[0]} failed; see ${out}`);
  }
  receipts.push({name, generatedLibSha256:crypto.createHash('sha256').update(original).digest('hex'), passed:true});
}
if(expected.size) throw Error('missing exact suite crate(s): '+[...expected].join(', '));
await fs.writeFile(path.join(target,'receipt.json'),JSON.stringify({schema:'uorc/heap-native-tests/1',receipts},null,2)+'\n');
console.log(JSON.stringify(receipts));
