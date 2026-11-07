// Language-1.2 sources are written in the LexLean notation (SPEC 17.21.1).
import fs from 'node:fs';
// Gates read module data from the SDK's canonical data form and write planted
// sources back in the notation, through the same lexlean::sdk conversions the
// Rust model uses (xtask data-form / notation). Nothing here parses notation.
import {spawnSync} from 'node:child_process';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
let binary;
function ensureBinary() {
  if (!binary) {
    const built = spawnSync('cargo', ['build', '--locked', '-q', '-p', 'xtask'], {cwd: root, stdio: ['ignore', 'ignore', 'inherit']});
    if (built.status !== 0) throw new Error('semantic source: xtask build failed');
    binary = path.join(process.env.CARGO_TARGET_DIR ? path.resolve(root, process.env.CARGO_TARGET_DIR) : path.join(root, 'target'), 'debug', 'xtask');
  }
  return binary;
}
function xtask(task, text) {
  ensureBinary();
  const run = spawnSync(binary, [task], {input: text, encoding: 'utf8', maxBuffer: 512 * 1024 * 1024});
  if (run.status !== 0) throw new Error(`semantic source: xtask ${task} failed: ${run.stderr || run.error}`);
  return run.stdout;
}
const forms = new Map();
// The canonical `\semanticdata{...}` / `\coredata{...}` text of a source.
export function dataForm(text) {
  const key = String(text);
  if (!forms.has(key)) forms.set(key, xtask('data-form', key));
  return forms.get(key);
}
// The canonical language-1.2 notation of a (possibly data-form) source.
export const notation = text => xtask('notation', String(text));

// The pinned LexLean runtime (LexLeanRuntime.*), which every generated
// language-1.2 module imports, elaborated under `dir` from the compiler's own
// runtime sources in import order. Published verification trees carry only
// `.olean` files, which a module file cannot import, so a standalone `lean`
// over generated modules stages the runtime this way. With `native`, each
// module is also compiled to C and to an object file for `leanc` linking.
// Lean 4.32.1's C emitter panics (LCNF getImpureDeclIndices) on LexLeanRuntime.Laws,
// whose content is the fold laws used by proofs. Natively it gets an object holding
// exactly the initializers its importers call; any executable Laws symbol a native
// gate actually reaches is then an unresolved link error, never a silent stub.
const initializerOnly = new Set(['LexLeanRuntime.Laws']);
function initializerSource(name, leanSource) {
  const symbol = module => module.replace(/\./g, '_');
  const imports = [...leanSource.matchAll(/^(?:public )?(?:meta )?import (?:all )?([\w.]+)$/gm)].map(m => m[1]);
  const call = f => `res = ${f}(builtin);\nif (lean_io_result_is_error(res)) return res;\nlean_dec_ref(res);\n`;
  const phase = (prefix, flag, body) => `static bool ${flag} = false;\nLEAN_EXPORT lean_object* ${prefix}initialize_${symbol(name)}(uint8_t builtin) {\nlean_object * res;\nif (${flag}) return lean_io_result_mk_ok(lean_box(0));\n${flag} = true;\n${body}return lean_io_result_mk_ok(lean_box(0));\n}\n`;
  const declare = imports.flatMap(m => ['', 'runtime_'].map(p => `lean_object* ${p}initialize_${symbol(m)}(uint8_t builtin);\n`)).join('');
  return `#include <lean/lean.h>\n${declare}`
    + phase('runtime_', '_G_runtime_initialized', imports.map(m => call('runtime_initialize_' + symbol(m))).join(''))
    + phase('meta_', '_G_meta_initialized', '')
    + phase('', '_G_initialized', imports.map(m => call('initialize_' + symbol(m))).join('')
      + call(`runtime_initialize_${symbol(name)}`) + call(`meta_initialize_${symbol(name)}`));
}
export function stageRuntime(dir, {native = false} = {}) {
  const listed = spawnSync(ensureBinary(), ['runtime-sources', dir], {encoding: 'utf8'});
  if (listed.status !== 0) throw new Error(`semantic source: runtime-sources failed: ${listed.stderr || listed.error}`);
  const modules = listed.stdout.trim().split('\n');
  if (!modules.includes('LexLeanRuntime.Portable')) throw new Error('semantic source: runtime inventory incomplete');
  const objects = [];
  for (const name of modules) {
    const base = path.join(dir, ...name.split('.'));
    const emitC = native && !initializerOnly.has(name);
    const args = ['-R', dir, '-o', base + '.olean', ...(emitC ? ['-c', base + '.c'] : []), base + '.lean'];
    const compiled = spawnSync('lean', args, {encoding: 'utf8', env: {...process.env, LEAN_PATH: dir}, maxBuffer: 64 * 1024 * 1024});
    if (compiled.status !== 0) throw new Error(`semantic source: runtime ${name} failed: ${compiled.stderr}${compiled.stdout}`);
    if (native) {
      if (!emitC) fs.writeFileSync(base + '.c', initializerSource(name, fs.readFileSync(base + '.lean', 'utf8')));
      const object = base + '.o';
      const linked = spawnSync('leanc', ['-O3', '-c', base + '.c', '-o', object], {encoding: 'utf8'});
      if (linked.status !== 0) throw new Error(`semantic source: runtime ${name} native compile failed: ${linked.stderr}`);
      objects.push(object);
    }
  }
  return {modules, objects};
}
