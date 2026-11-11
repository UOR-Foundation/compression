// Export the uorc client roots (client/src/Uorc/Client.lex.tex) to certified Rust packages
// through `lexlean production build`, into generated/rust/.
//
// The client's own modules live under client/src, outside the authority project:
// the codec (client/src/Uorc/Codec) writes its tables with the in-place byte
// update primitive, which only the pinned LexLean below has. The export stages
// client/src over a copy of src, so the client imports authority modules freely.
//
// The authority graph marks only proof-relevant definitions executable, so the
// export runs on a staged copy where two mechanical, meaning-preserving
// projections are applied to the client's import closure:
//   1. every definition is marked `executable: true` (the Rust lowering needs the
//      whole call closure executable), and Client's compress and decompress
//      gain their rust-std production annotation;
//   2. a match arm that returns a boxed-field binder directly (a field whose
//      type is a datatype with a field-carrying constructor, which rust-std
//      stores behind a shared box) is rewritten to `let x : T := binder; x`.
//      LexLean rejects the direct form in its own Rust preservation certificate
//      (LLV7015); the let form certifies.
// Repository sources are never modified.
//
// The export uses LexLean pinned at LEXLEAN_REV, the branch claude/uorc-pin of
// UOR-Foundation/LexLean: the rust-std shared-box lowering (LexLean#441) keeps
// search states behind reference-counted boxes instead of copying them on every
// field read, and the in-place byte update primitive (LexLean#443) gives the
// codec O(1) table writes. The pin moves to a LexLean release once both ship.
// Set LEXLEAN to an already built binary of the same revision to skip the build.
//
// The export is the factory obligation `export` (factory/graph.json): its
// identity covers only what it reads (the import closure of Uorc.Client, the
// LexLean project files and these scripts), recorded in generated/rust/factory.identity.
// When the recorded identity matches, the export is already proved and this
// exits without running; pass --force to export anyway.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {dataForm, notation} from './semantic-source.mjs';
import {loadGraph, identities, workingTree} from './factory.mjs';

const root = process.cwd();
const out = path.join(root, 'generated/rust');
const entry = 'src/Uorc/Client.lex.tex'; // staged path
const roots = [['uorc_compress', 'Uorc.Uorc.Client.compress'], ['uorc_decompress', 'Uorc.Uorc.Client.decompress']];
const LEXLEAN_REPO = 'https://github.com/UOR-Foundation/LexLean';
const LEXLEAN_REV = '4c41d395286826faf568817f46e181c22b09cba1';
const fail = m => { throw new Error(m); };
const run = (cmd, args, cwd) => {
  const r = spawnSync(cmd, args, {cwd, encoding: 'utf8', maxBuffer: 256 * 1024 * 1024});
  if (r.error || r.status !== 0) fail(`${cmd} ${args.join(' ')}: ${r.error ?? ''}${r.stdout}${r.stderr}`);
  return r.stdout;
};
function lexleanBinary() {
  if (process.env.LEXLEAN) return process.env.LEXLEAN;
  const tool = path.join(root, '.lexlean/tool', LEXLEAN_REV);
  const bin = path.join(tool, 'bin', 'lexlean');
  if (!fs.existsSync(bin)) run('cargo', ['install', '--locked', '--git', LEXLEAN_REPO, '--rev', LEXLEAN_REV, '--root', tool, 'lexlean'], root);
  return bin;
}
const moduleFile = (dir, m) => path.join(dir, 'src', ...m.split('.')) + '.lex.tex';

function closure(dir) {
  const seen = new Set(); const stack = ['Uorc.Client'];
  while (stack.length) {
    const m = stack.pop(); if (seen.has(m)) continue; seen.add(m);
    for (const i of fs.readFileSync(moduleFile(dir, m), 'utf8').matchAll(/\\importmodule\{([^}]+)\}/g)) stack.push(i[1]);
  }
  return [...seen];
}

function markExecutable(file) {
  const lines = fs.readFileSync(file, 'utf8').split('\n'); const result = [];
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (/^definition\{name: .*\}$/.test(line)) { result.push(line.includes('executable:') ? line : line.slice(0, -1) + ', executable: true}'); continue; }
    result.push(line);
    if (line !== 'definition{') continue;
    let j = i + 1; while (lines[j] !== '}') j++;
    const block = lines.slice(i + 1, j);
    if (!block.some(b => b.startsWith('  executable:'))) { block[block.length - 1] = block[block.length - 1].replace(/,$/, '') + ','; block.push('  executable: true'); }
    result.push(...block, '}'); i = j;
  }
  fs.writeFileSync(file, result.join('\n'));
}

// The client's two roots become production roots only in the staged export:
// the authority graph keeps the closure non-executable, so it cannot carry them.
function markRoots(file, names) {
  const lines = fs.readFileSync(file, 'utf8').split('\n');
  for (const name of names) {
    const at = lines.indexOf(`  name: ${name},`); if (at < 0) fail(`root ${name} missing`);
    let end = at; while (lines[end] !== '}') end++;
    lines[end - 1] += ',';
    lines.splice(end, 0, '  production: {effects: [allocation, overflow, recursion], targets: [rust-std]}');
  }
  fs.writeFileSync(file, lines.join('\n'));
}

function wrapBoxedReturns(dir, modules) {
  const marker = /\\semanticdata\{(.*)\}/;
  const data = new Map(modules.map(m => [m, JSON.parse(marker.exec(dataForm(fs.readFileSync(moduleFile(dir, m), 'utf8')))[1])]));
  const decls = new Map();
  for (const [m, d] of data) for (const decl of d.declarations) if (decl.kind === 'inductive' || decl.kind === 'structure') decls.set(`${m}::${decl.name}`, decl);
  // A field rust-std stores behind a box: one whose type is a datatype with a
  // field-carrying constructor (structures count as one constructor).
  const boxed = (f, m) => {
    if (f.kind !== 'named' || (f.arguments ?? []).length) return null;
    const module = f.member.module ?? m; const decl = decls.get(`${module}::${f.member.name}`);
    if (!decl || (decl.type_parameters ?? []).length) return null;
    const carries = decl.kind === 'structure' ? decl.fields.length > 0 : decl.constructors.some(c => c.fields.length > 0);
    return carries ? {module, type: f.member.name} : null;
  };
  const fieldsOf = new Map();
  for (const [m, d] of data) for (const decl of d.declarations) if (decl.kind === 'inductive')
    for (const c of decl.constructors) fieldsOf.set(`${m}::${decl.name}.${c.name}`, c.fields.map(f => boxed(f, m)));
  for (const [m, d] of data) {
    let changed = false;
    const walk = x => {
      if (!x || typeof x !== 'object') return;
      if (Array.isArray(x)) { x.forEach(walk); return; }
      if (x.kind === 'match') for (const b of x.branches) {
        const fields = fieldsOf.get(`${b.constructor.module ?? m}::${b.constructor.name}`);
        const r = fields && b.body.kind === 'var' ? fields[b.binders.indexOf(b.body.name)] : null;
        if (r) {
          const local = `${b.body.name}Field`;
          const member = r.module === m ? {name: r.type} : {module: r.module, name: r.type};
          b.body = {kind: 'let', binder: {name: local, type: {arguments: [], kind: 'named', member}}, value: {kind: 'var', name: b.body.name}, body: {kind: 'var', name: local}};
          changed = true;
        }
      }
      for (const v of Object.values(x)) walk(v);
    };
    walk(d.declarations);
    if (changed) {
      const text = dataForm(fs.readFileSync(moduleFile(dir, m), 'utf8'));
      fs.writeFileSync(moduleFile(dir, m), notation(text.replace(marker, () => '\\semanticdata{' + JSON.stringify(d) + '}')));
    }
  }
}

const exportRecord = path.join(out, 'factory.identity');
const exportIdentity = identities(loadGraph(root), workingTree(root), {}).export;
if (!process.argv.includes('--force') && fs.existsSync(exportRecord) && fs.readFileSync(exportRecord, 'utf8').trim() === `export ${exportIdentity}`) {
  console.log(`export proved: generated/rust was exported from these inputs (${exportIdentity}); --force exports anyway`);
  process.exit(0);
}
const lexlean = lexleanBinary();
const stage = fs.mkdtempSync(path.join(os.tmpdir(), 'uorc-client-export-'));
try {
  for (const f of ['src', 'lexlean.toml', 'lakefile.toml', 'lake-manifest.json', 'lean-toolchain']) fs.cpSync(path.join(root, f), path.join(stage, f), {recursive: true});
  fs.cpSync(path.join(root, 'client/src'), path.join(stage, 'src'), {recursive: true});
  const config = path.join(stage, 'lexlean.toml');
  // The client closure outgrows two of the authority project's limits: its
  // extraction passes 64 MiB of child output, and the compress root's Rust
  // preservation certificate runs past 30 minutes on a 4-core machine.
  fs.writeFileSync(config, fs.readFileSync(config, 'utf8').replace(/^entrypoints = .*$/m, `entrypoints = ["${entry}"]`)
    .replace(/^max_child_output_bytes = .*$/m, 'max_child_output_bytes = 536870912')
    .replace(/^child_timeout_ms = .*$/m, 'child_timeout_ms = 5400000'));
  const modules = closure(stage);
  for (const m of modules) markExecutable(moduleFile(stage, m));
  markRoots(moduleFile(stage, 'Uorc.Client'), ['compress', 'decompress']);
  wrapBoxedReturns(stage, modules);
  run(lexlean, ['lock'], stage);
  run(lexlean, ['check'], stage);
  const target = JSON.parse(run(lexlean, ['production', 'targets'], stage)).targets.find(t => t.id === 'rust-std') ?? fail('rust-std target missing');
  const request = {roots: roots.map(([name, rootName]) => ({package: {name, version: '0.1.0'}, root: rootName, targets: [{digest: target.digest, id: 'rust-std'}]})), spec: 'lexlean/production-request/1'};
  fs.writeFileSync(path.join(stage, 'request.json'), JSON.stringify(request));
  const result = JSON.parse(run(lexlean, ['--diagnostic-format', 'json', 'production', 'build', '--request', 'request.json'], stage));
  if (!result.success) fail(`production build failed: ${JSON.stringify(result.diagnostics)}`);
  const bundle = path.resolve(stage, result.artifacts[0]);
  fs.rmSync(out, {recursive: true, force: true}); fs.mkdirSync(out, {recursive: true});
  for (const [name] of roots) fs.cpSync(path.join(bundle, 'packages/rust-std', name), path.join(out, name), {recursive: true});
  fs.writeFileSync(path.join(out, 'request.json'), JSON.stringify(request, null, 1) + '\n');
  fs.copyFileSync(path.join(bundle, 'result.json'), path.join(out, 'result.json'));
  fs.writeFileSync(exportRecord, `export ${exportIdentity}\n`);
  console.log(`exported ${roots.length} packages from ${modules.length} modules (result ${result.result_id ?? result.artifacts[0]})`);
} finally { fs.rmSync(stage, {recursive: true, force: true}); }
