// The one LexLean revision every uorc recipe runs: the authority gate, the
// formal legs, and the client export (scripts/client-export.mjs) all take it
// from lexlean.pin.json, so the model is checked by the same compiler it is
// exported with. The pin is a branch of UOR-Foundation/LexLean until a LexLean
// release carries what it adds (see lexlean.pin.json `carries`).
//
//   node scripts/lexlean-pin.mjs dir      prints the pinned tool's bin directory
//   node scripts/lexlean-pin.mjs install  builds it there once, then prints it
//   node scripts/lexlean-pin.mjs exec ARGS runs it (building it once) with ARGS
//
// The pin is named, never put on PATH: the SDK refuses a PATH that shadows the
// tools it declares (PP5401), so recipes run `lexlean` through `exec`.
//
// Set LEXLEAN to an already built binary of the same revision to skip the build,
// or LEXLEAN_TOOL_ROOT to the directory it is installed under (the offline
// construction installs it into its sealed acquisition this way). The build
// itself runs beside the tool root (on the workspace disk) and is removed once the binary
// is installed: the recipe containers mount a small /tmp, which cargo install's
// default build directory would fill.
import fs from 'node:fs';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '..');
const pin = JSON.parse(fs.readFileSync(path.join(root, 'lexlean.pin.json'), 'utf8'));
export const LEXLEAN_REPO = pin.repository;
export const LEXLEAN_REV = pin.revision;

// The Rust model and gates link the LexLean crate (Cargo.toml's workspace
// `lexlean`), whose source conversions and runtime modules must be those of the
// compiler that runs: a crate of another revision stages a runtime without the
// primitives the pinned compiler emits. Wherever the workspace is present, its
// dependency is this pin.
export function cargoRevision(text) {
  const m = /^lexlean = \{ version = "=[^"]+", git = "([^"]+)", rev = "([0-9a-f]{40})" \}$/m.exec(text);
  return m && {repository: m[1], revision: m[2]};
}
const manifest = path.join(root, 'Cargo.toml');
if (fs.existsSync(manifest)) {
  const cargo = cargoRevision(fs.readFileSync(manifest, 'utf8'));
  if (!cargo || cargo.repository !== LEXLEAN_REPO || cargo.revision !== LEXLEAN_REV)
    throw new Error(`Cargo.toml's lexlean dependency ${JSON.stringify(cargo)} is not the pin ${LEXLEAN_REPO}@${LEXLEAN_REV} (lexlean.pin.json)`);
}

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
    const build = `${toolRoot()}.build`;
    const r = spawnSync('cargo', ['install', '--locked', '--git', LEXLEAN_REPO, '--rev', LEXLEAN_REV, '--root', toolRoot(), '--target-dir', build, 'lexlean'], {cwd: root, stdio: ['ignore', 2, 2]});
    fs.rmSync(build, {recursive: true, force: true});
    if (r.error || r.status !== 0) throw new Error(`cargo install lexlean ${LEXLEAN_REV} failed: ${r.error ?? r.status}`);
  }
  return bin;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const command = process.argv[2];
  if (command === 'dir') console.log(lexleanDir());
  else if (command === 'install') console.log(path.dirname(lexleanBinary()));
  else if (command === 'exec') {
    const r = spawnSync(lexleanBinary(), process.argv.slice(3), {cwd: process.cwd(), stdio: 'inherit'});
    process.exit(r.error ? 70 : r.status ?? 70);
  } else { console.error('usage: lexlean-pin.mjs dir|install|exec ARGS'); process.exit(64); }
}
