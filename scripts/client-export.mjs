// Export the uorc client roots (client/src/Uorc/Client.lex.tex) to certified Rust packages
// through `lexlean production build`, into generated/rust/.
//
// The client's own modules live under client/src, outside the authority project:
// the codec (client/src/Uorc/Codec) writes its tables with the in-place byte
// update primitive, which only the pinned LexLean below has. The export stages
// client/src over a copy of src, so the client imports authority modules freely.
//
// The authority graph marks only proof-relevant definitions executable, so the
// export runs on a staged copy where three mechanical, meaning-preserving
// projections are applied to the client's import closure:
//   1. every definition is marked `executable: true` (the Rust lowering needs the
//      whole call closure executable), and Client's compress and decompress
//      gain their rust-std production annotation;
//   2. a match arm that returns a boxed-field binder directly (a field whose
//      type is a datatype with a field-carrying constructor, which rust-std
//      stores behind a shared box) is rewritten to `let x : T := binder; x`.
//      LexLean rejects the direct form in its own Rust preservation certificate
//      (LLV7015); the let form certifies;
//   3. in definition bodies, Nat addition and `prim multiply` whose second
//      operand is a natural literal and whose first is not are written literal
//      first (`x + 1` -> `1 + x`, `prim multiply x 256` -> `prim multiply 256 x`).
//      The Lean kernel unfolds Nat.add and Nat.mul by recursion on the second
//      argument, so a literal there is peeled one successor at a time: in the
//      certificates `67108864 * (mb + 1)` became `_ + 67108864` and recursed
//      67 million times (past 13 GB, killed). Literal first, the recursion
//      stops at the symbolic operand. Definitions that a theorem with
//      parameters can reach keep their form, since its proof may need `x + 1`
//      to reduce to a successor.
// Repository sources are never modified.
//
// Subtraction recurses on its second argument too and cannot be reordered, so
// before anything slow runs the export checks the projected closure (`--check`
// runs the same check alone on the working tree, in seconds): a definition
// holding `prim subtract x N`, or an addition or multiplication left with a
// literal second, with x not a literal and N above SECOND_LITERAL_BOUND, stops
// the export with its module and definition named.
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
import {fileURLToPath} from 'node:url';
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

// `file` maps a module name to its source: the stage, or for --check the
// working tree with client/src over src, as the stage lays them out.
function closure(file) {
  const seen = new Set(); const stack = ['Uorc.Client'];
  while (stack.length) {
    const m = stack.pop(); if (seen.has(m)) continue; seen.add(m);
    for (const i of fs.readFileSync(file(m), 'utf8').matchAll(/\\importmodule\{([^}]+)\}/g)) stack.push(i[1]);
  }
  return [...seen];
}
const workingFile = m => [moduleFile(path.join(root, 'client'), m), moduleFile(root, m)].find(f => fs.existsSync(f)) ?? fail(`module ${m} missing`);

// The literal-second check over the working tree's client closure, after
// projection 3 (in memory): what the export would refuse, without staging.
export function checkWorkingTree() {
  const modules = closure(workingFile);
  const data = new Map(modules.map(m => [m, readData(workingFile(m))]));
  const {swaps} = literalFirst(data);
  return {modules: modules.length, swaps, hazards: literalSecondHazards(data)};
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

// Projections 2 and 3 edit the data form: `data` maps each module to its parsed
// `\semanticdata{...}`, and each projection returns the modules it changed.
const marker = /\\semanticdata\{(.*)\}/;
const readData = file => JSON.parse(marker.exec(dataForm(fs.readFileSync(file, 'utf8')))[1]);
function writeData(file, d) {
  const text = dataForm(fs.readFileSync(file, 'utf8'));
  fs.writeFileSync(file, notation(text.replace(marker, () => '\\semanticdata{' + JSON.stringify(d) + '}')));
}

export function wrapBoxedReturns(data) {
  const touched = new Set();
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
    if (changed) touched.add(m);
  }
  return touched;
}

// Nat.add and Nat.mul recurse on their second argument, Nat.sub likewise and
// it cannot be reordered (the kernel's GMP shortcut needs both sides literal).
// The comparisons (Nat.beq, Nat.ble, Nat.blt) match both arguments together
// and stop at the first symbolic one, so a literal on either side is safe.
const natLiteral = x => x?.kind === 'nat' && typeof x.value === 'string';
const secondRecursive = x =>
  x.kind === 'add' ? {operation: 'add', operands: [x.left, x.right], swap: () => { [x.left, x.right] = [x.right, x.left]; }}
  : x.kind === 'primitive' && x.arguments?.length === 2 && (x.operation === 'multiply' || x.operation === 'subtract')
    ? {operation: x.operation, operands: x.arguments, swap: x.operation === 'multiply' ? () => x.arguments.reverse() : null}
  : null;
// Each definition body node of the closure, with its module and definition.
function* definitionNodes(data) {
  for (const [module, d] of data) for (const decl of d.declarations) if (decl.kind === 'definition') {
    const stack = [decl.body];
    while (stack.length) {
      const x = stack.pop();
      if (!x || typeof x !== 'object') continue;
      if (Array.isArray(x)) { stack.push(...x); continue; }
      yield {module, definition: decl.name, node: x};
      stack.push(...Object.values(x));
    }
  }
}
const literalSecond = ({operands: [a, b]}) => natLiteral(b) && !natLiteral(a);

// The definitions a theorem with parameters can unfold, through any chain of
// calls. Such a theorem's proof may rely on `x + 1` reducing to a successor
// (`f (n + 1) = ...` by reflexivity), which `1 + x` does not, so these keep
// their form; they are authority definitions the export already certified.
function theoremReach(data) {
  const refs = (x, module, out) => {
    if (!x || typeof x !== 'object') return out;
    if (Array.isArray(x)) { for (const y of x) refs(y, module, out); return out; }
    if (x.function?.name) out.push(`${x.function.module ?? module}::${x.function.name}`);
    for (const d of Array.isArray(x.definitions) ? x.definitions : []) out.push(`${d.module ?? module}::${d.name}`);
    for (const v of Object.values(x)) refs(v, module, out);
    return out;
  };
  const bodies = new Map(); const stack = [];
  for (const [module, d] of data) for (const decl of d.declarations) {
    if (decl.kind === 'definition') bodies.set(`${module}::${decl.name}`, [decl.body, module]);
    if (decl.kind === 'theorem' && (decl.parameters ?? []).length) refs([decl.statement, decl.proof], module, stack);
  }
  const reach = new Set();
  while (stack.length) {
    const key = stack.pop(); if (reach.has(key) || !bodies.has(key)) continue;
    reach.add(key); refs(...bodies.get(key), stack);
  }
  return reach;
}

export function literalFirst(data) {
  const pinned = theoremReach(data);
  const touched = new Set(); let swaps = 0;
  for (const {module, definition, node} of definitionNodes(data)) {
    const op = secondRecursive(node);
    if (op?.swap && literalSecond(op) && !pinned.has(`${module}::${definition}`)) { op.swap(); swaps++; touched.add(module); }
  }
  return {touched, swaps};
}

// The largest literal second operand the check lets through, measured on a
// one-module production build of the match-table loop that blew up (a loop
// argument `x` reaching `x - N` in the certificate): N = 4096 adds nothing,
// N = 65536 adds 0.3 GB to the Lean child, N = 2^20 and 2^26 pass 6 GB.
export const SECOND_LITERAL_BOUND = 65536n;
// Literal second operands above the bound that an export certified as they
// stand: the certificate never unfolds them there. Keyed by module, operation
// and literal, so a regenerated definition name keeps its entry.
export const CERTIFIED_LITERAL_SECONDS = [
  // Under a remainder in the ncm encoder's bit step, not on a loop argument;
  // certified by export b9e27c6d (generated/rust/factory.identity).
  {module: 'Uorc.Codec.NcmEncode', operation: 'subtract', literal: '70368744177664'},
];
export function literalSecondHazards(data, bound = SECOND_LITERAL_BOUND, certified = CERTIFIED_LITERAL_SECONDS) {
  const hazards = [];
  for (const {module, definition, node} of definitionNodes(data)) {
    const op = secondRecursive(node);
    if (!op || !literalSecond(op) || BigInt(op.operands[1].value) <= bound) continue;
    const hazard = {module, definition, operation: op.operation, literal: op.operands[1].value};
    if (!certified.some(c => c.module === module && c.operation === hazard.operation && c.literal === hazard.literal)) hazards.push(hazard);
  }
  return hazards;
}
const describeHazards = hazards => hazards.map(h => `  ${h.module}.${h.definition}: ${h.operation} x ${h.literal}`).join('\n');

function main() {
  if (process.argv.includes('--check')) {
    const {modules, swaps, hazards} = checkWorkingTree();
    if (hazards.length) fail(`literal second operands above ${SECOND_LITERAL_BOUND} (write the literal first or bound the operand):\n${describeHazards(hazards)}`);
    console.log(`check: ${modules} modules, ${swaps} operations written literal first, no literal second operand above ${SECOND_LITERAL_BOUND}`);
    return;
  }
  const exportRecord = path.join(out, 'factory.identity');
  const exportIdentity = identities(loadGraph(root), workingTree(root), {}).export;
  if (!process.argv.includes('--force') && fs.existsSync(exportRecord) && fs.readFileSync(exportRecord, 'utf8').trim() === `export ${exportIdentity}`) {
    console.log(`export proved: generated/rust was exported from these inputs (${exportIdentity}); --force exports anyway`);
    return;
  }
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
    const staged = m => moduleFile(stage, m);
    const modules = closure(staged);
    for (const m of modules) markExecutable(staged(m));
    markRoots(staged('Uorc.Client'), ['compress', 'decompress']);
    const data = new Map(modules.map(m => [m, readData(staged(m))]));
    const {touched, swaps} = literalFirst(data);
    for (const m of new Set([...wrapBoxedReturns(data), ...touched])) writeData(staged(m), data.get(m));
    const hazards = literalSecondHazards(data);
    if (hazards.length) fail(`literal second operands above ${SECOND_LITERAL_BOUND} would unfold in the certificates (write the literal first or bound the operand):\n${describeHazards(hazards)}`);
    console.log(`staged ${modules.length} modules; ${swaps} operations written literal first`);
    const lexlean = lexleanBinary();
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
}
if (process.argv[1] && fs.realpathSync(process.argv[1]) === fileURLToPath(import.meta.url)) main();
