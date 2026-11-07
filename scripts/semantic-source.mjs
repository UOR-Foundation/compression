// Language-1.2 sources are written in the LexLean notation (SPEC 17.21.1).
// Gates read module data from the SDK's canonical data form and write planted
// sources back in the notation, through the same lexlean::sdk conversions the
// Rust model uses (xtask data-form / notation). Nothing here parses notation.
import {spawnSync} from 'node:child_process';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
let binary;
function xtask(task, text) {
  if (!binary) {
    const built = spawnSync('cargo', ['build', '--locked', '-q', '-p', 'xtask'], {cwd: root, stdio: ['ignore', 'ignore', 'inherit']});
    if (built.status !== 0) throw new Error('semantic source: xtask build failed');
    binary = path.join(process.env.CARGO_TARGET_DIR ? path.resolve(root, process.env.CARGO_TARGET_DIR) : path.join(root, 'target'), 'debug', 'xtask');
  }
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
