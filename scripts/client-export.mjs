// Export the uorc client roots (src/Uorc/Client.lex.tex) to certified Rust packages
// through `lexlean production build`, into generated/rust/.
//
// The authority graph marks only proof-relevant definitions executable, so the
// export runs on a staged copy where two mechanical, meaning-preserving
// projections are applied to the client's import closure:
//   1. every definition is marked `executable: true` (the Rust lowering needs the
//      whole call closure executable);
//   2. a match arm that returns a recursive-field binder directly is rewritten
//      to `let x : T := binder; x`. LexLean 0.4.0 rejects the direct form in its
//      own Rust preservation certificate (LLV7015); the let form certifies.
// Repository sources are never modified.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {dataForm, notation} from './semantic-source.mjs';

const root = process.cwd();
const out = path.join(root, 'generated/rust');
const entry = 'src/Uorc/Client.lex.tex';
const roots = [['uorc_compress', 'Uorc.Uorc.Client.compress'], ['uorc_decompress', 'Uorc.Uorc.Client.decompress']];
const fail = m => { throw new Error(m); };
const run = (cmd, args, cwd) => {
  const r = spawnSync(cmd, args, {cwd, encoding: 'utf8', maxBuffer: 256 * 1024 * 1024});
  if (r.error || r.status !== 0) fail(`${cmd} ${args.join(' ')}: ${r.error ?? ''}${r.stdout}${r.stderr}`);
  return r.stdout;
};
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

function wrapRecursiveReturns(dir, modules) {
  const marker = /\\semanticdata\{(.*)\}/;
  const data = new Map(modules.map(m => [m, JSON.parse(marker.exec(dataForm(fs.readFileSync(moduleFile(dir, m), 'utf8')))[1])]));
  const recursive = new Map();
  for (const [m, d] of data) for (const decl of d.declarations) if (decl.kind === 'inductive') for (const c of decl.constructors) {
    const fields = new Set();
    c.fields.forEach((f, i) => { if (f.kind === 'named' && f.member.name === decl.name && (!f.member.module || f.member.module === m)) fields.add(i); });
    if (fields.size) recursive.set(`${m}::${decl.name}.${c.name}`, {fields, module: m, type: decl.name});
  }
  for (const [m, d] of data) {
    let changed = false;
    const walk = x => {
      if (!x || typeof x !== 'object') return;
      if (Array.isArray(x)) { x.forEach(walk); return; }
      if (x.kind === 'match') for (const b of x.branches) {
        const r = recursive.get(`${b.constructor.module ?? m}::${b.constructor.name}`);
        if (r && b.body.kind === 'var' && r.fields.has(b.binders.indexOf(b.body.name))) {
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

const stage = fs.mkdtempSync(path.join(os.tmpdir(), 'uorc-client-export-'));
try {
  for (const f of ['src', 'lexlean.toml', 'lakefile.toml', 'lake-manifest.json', 'lean-toolchain']) fs.cpSync(path.join(root, f), path.join(stage, f), {recursive: true});
  const config = path.join(stage, 'lexlean.toml');
  fs.writeFileSync(config, fs.readFileSync(config, 'utf8').replace(/^entrypoints = .*$/m, `entrypoints = ["${entry}"]`));
  const modules = closure(stage);
  for (const m of modules) markExecutable(moduleFile(stage, m));
  wrapRecursiveReturns(stage, modules);
  run('lexlean', ['lock'], stage);
  run('lexlean', ['check'], stage);
  const target = JSON.parse(run('lexlean', ['production', 'targets'], stage)).targets.find(t => t.id === 'rust-std') ?? fail('rust-std target missing');
  const request = {roots: roots.map(([name, rootName]) => ({package: {name, version: '0.1.0'}, root: rootName, targets: [{digest: target.digest, id: 'rust-std'}]})), spec: 'lexlean/production-request/1'};
  fs.writeFileSync(path.join(stage, 'request.json'), JSON.stringify(request));
  const result = JSON.parse(run('lexlean', ['--diagnostic-format', 'json', 'production', 'build', '--request', 'request.json'], stage));
  if (!result.success) fail(`production build failed: ${JSON.stringify(result.diagnostics)}`);
  const bundle = path.resolve(stage, result.artifacts[0]);
  fs.rmSync(out, {recursive: true, force: true}); fs.mkdirSync(out, {recursive: true});
  for (const [name] of roots) fs.cpSync(path.join(bundle, 'packages/rust-std', name), path.join(out, name), {recursive: true});
  fs.writeFileSync(path.join(out, 'request.json'), JSON.stringify(request, null, 1) + '\n');
  fs.copyFileSync(path.join(bundle, 'result.json'), path.join(out, 'result.json'));
  console.log(`exported ${roots.length} packages from ${modules.length} modules (result ${result.result_id ?? result.artifacts[0]})`);
} finally { fs.rmSync(stage, {recursive: true, force: true}); }
