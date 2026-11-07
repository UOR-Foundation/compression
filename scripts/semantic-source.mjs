// Language-1.2 sources are written in the LexLean notation (SPEC 17.21.1).
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
export function stageRuntime(dir, {native = false} = {}) {
  const listed = spawnSync(ensureBinary(), ['runtime-sources', dir], {encoding: 'utf8'});
  if (listed.status !== 0) throw new Error(`semantic source: runtime-sources failed: ${listed.stderr || listed.error}`);
  const modules = listed.stdout.trim().split('\n');
  if (!modules.includes('LexLeanRuntime.Portable')) throw new Error('semantic source: runtime inventory incomplete');
  const objects = [];
  for (const name of modules) {
    const base = path.join(dir, ...name.split('.'));
    const args = ['-R', dir, '-o', base + '.olean', ...(native ? ['-c', base + '.c'] : []), base + '.lean'];
    const compiled = spawnSync('lean', args, {encoding: 'utf8', env: {...process.env, LEAN_PATH: dir}, maxBuffer: 64 * 1024 * 1024});
    if (compiled.status !== 0) throw new Error(`semantic source: runtime ${name} failed: ${compiled.stderr}${compiled.stdout}`);
    if (native) {
      const object = base + '.o';
      const linked = spawnSync('leanc', ['-O3', '-c', base + '.c', '-o', object], {encoding: 'utf8'});
      if (linked.status !== 0) throw new Error(`semantic source: runtime ${name} native compile failed: ${linked.stderr}`);
      objects.push(object);
    }
  }
  return {modules, objects};
}
