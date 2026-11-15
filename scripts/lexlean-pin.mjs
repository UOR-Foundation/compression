// The one LexLean revision every uorc recipe runs: the authority gate, the
// formal legs, and the client export (scripts/client-export.mjs) all take it
// from lexlean.pin.json, so the model is checked by the same compiler it is
// exported with. The pin is a branch of UOR-Foundation/LexLean until a LexLean
// release carries what it adds (see lexlean.pin.json `carries`).
//
//   node scripts/lexlean-pin.mjs dir      prints the pinned tool's bin directory
//   node scripts/lexlean-pin.mjs install  builds it there once, then prints it
//
// Set LEXLEAN to an already built binary of the same revision to skip the build,
// or LEXLEAN_TOOL_ROOT to the directory it is installed under (the offline
// construction installs it into its sealed acquisition this way).
import fs from 'node:fs';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '..');
const pin = JSON.parse(fs.readFileSync(path.join(root, 'lexlean.pin.json'), 'utf8'));
export const LEXLEAN_REPO = pin.repository;
export const LEXLEAN_REV = pin.revision;

export function lexleanDir() {
  if (process.env.LEXLEAN) return path.dirname(path.resolve(process.env.LEXLEAN));
  return path.join(toolRoot(), 'bin');
}

function toolRoot() {
  return process.env.LEXLEAN_TOOL_ROOT ? path.resolve(process.env.LEXLEAN_TOOL_ROOT) : path.join(root, '.lexlean/tool', LEXLEAN_REV);
}

export function lexleanBinary() {
  if (process.env.LEXLEAN) return path.resolve(process.env.LEXLEAN);
  const bin = path.join(lexleanDir(), 'lexlean');
  if (!fs.existsSync(bin)) {
    const r = spawnSync('cargo', ['install', '--locked', '--git', LEXLEAN_REPO, '--rev', LEXLEAN_REV, '--root', toolRoot(), 'lexlean'], {cwd: root, stdio: ['ignore', 2, 2]});
    if (r.error || r.status !== 0) throw new Error(`cargo install lexlean ${LEXLEAN_REV} failed: ${r.error ?? r.status}`);
  }
  return bin;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const command = process.argv[2];
  if (command === 'dir') console.log(lexleanDir());
  else if (command === 'install') console.log(path.dirname(lexleanBinary()));
  else { console.error('usage: lexlean-pin.mjs dir|install'); process.exit(64); }
}
