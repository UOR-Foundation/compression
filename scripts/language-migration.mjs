// Language-1.1 to 1.2 source migration bridge for retained evidence receipts.
// A receipt that pinned a source's 1.1 bytes still applies to the 1.2 source
// only when the SDK reads both to the same module model (the schema tag
// excepted). The gate recomputes the current model digest on every run; the
// historical digest is recorded from the named commit, where it can be
// re-derived. A changed model is never bridged: its evidence must be re-run.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {dataForm} from './semantic-source.mjs';

export const bridgePath = 'validation/language-1.2-migration.json';
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
export function modelDigest(text) {
  const match = dataForm(text).match(/\\(?:semantic|core|tooling)data\{(.*)\}\n/);
  if (!match) throw new Error('language migration: no module data');
  const model = JSON.parse(match[1]);
  delete model.spec;
  return sha(JSON.stringify(model));
}
let bridges = new Map();
function rows(root) {
  if (!bridges.has(root)) {
    const file = path.join(root, bridgePath);
    const value = fs.existsSync(file) ? JSON.parse(fs.readFileSync(file, 'utf8')) : {schema: 'uorc/language-migration/1', sources: {}};
    if (value.schema !== 'uorc/language-migration/1') throw new Error('language migration: schema');
    bridges.set(root, value);
  }
  return bridges.get(root);
}
// Whether `file` under `root` is the source a receipt pinned at `expected`.
export function sourceBound(root, file, expected) {
  const current = fs.readFileSync(path.join(root, file));
  if (sha(current) === expected) return true;
  const row = rows(root).sources[file];
  return Boolean(row && row.historical_sha256 === expected && row.current_sha256 === sha(current)
    && row.model_sha256 === modelDigest(current.toString('utf8')));
}
// Record bridges for every pinned source whose bytes changed but whose model did not.
function write(root, ref, pins) {
  const value = rows(root);
  value.historical_refs = [...new Set([...(value.historical_refs ?? []), ...ref.split(',')])].sort();
  const refused = [];
  for (const [file, expected] of pins) {
    const current = fs.readFileSync(path.join(root, file));
    if (sha(current) === expected) continue;
    const historical = ref.split(',').map(commit => spawnSync('git', ['show', `${commit}:${file}`], {cwd: root, maxBuffer: 1 << 30}))
      .find(shown => shown.status === 0 && sha(shown.stdout) === expected);
    if (!historical) { refused.push(`${file}: not pinned at ${ref}`); continue; }
    let model, now;
    try { model = modelDigest(historical.stdout.toString('utf8')); now = modelDigest(current.toString('utf8')); }
    catch { refused.push(`${file}: not a LexLean module`); continue; }
    if (model !== now) { refused.push(`${file}: model changed`); continue; }
    value.sources[file] = {historical_sha256: expected, current_sha256: sha(current), model_sha256: model};
  }
  value.sources = Object.fromEntries(Object.entries(value.sources).sort(([a], [b]) => a < b ? -1 : 1));
  fs.writeFileSync(path.join(root, bridgePath), JSON.stringify(value, null, 1) + '\n');
  return refused;
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [ref, ...receipts] = process.argv.slice(2);
  const pins = receipts.flatMap(r => Object.entries(JSON.parse(fs.readFileSync(r, 'utf8')).source_sha256 ?? {}));
  for (const line of write(process.cwd(), ref, pins)) console.log('not bridged:', line);
}
