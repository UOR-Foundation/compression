// Project validation orchestration only. SDK commands own compilation and proof.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const SCRIPT = fileURLToPath(import.meta.url);
const SDK = '/usr/local/bin/prismpm';
const INVENTORY = '/opt/prismpm/share/inventory.json';
const OMIT = new Set(['.git', '.lake', '.lexlean', '.prism', 'target', 'node_modules', '.tmp', 'reports']);
export class ValidationFailure extends Error {
  constructor(code, message) { super(`${code}: ${message}`); this.code = code; }
}
const fail = (code, message) => { throw new ValidationFailure(code, message); };
export const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
export const canonical = value => JSON.stringify(value, (_key, item) => item && typeof item === 'object' && !Array.isArray(item)
  ? Object.fromEntries(Object.entries(item).sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0)) : item);
const writeJson = (file, value) => { fs.mkdirSync(path.dirname(file), { recursive: true }); fs.writeFileSync(file, canonical(value) + '\n'); };
const readJson = file => JSON.parse(fs.readFileSync(file, 'utf8'));

export function files(root, omit = new Set()) {
  const result = [];
  function walk(relative) {
    for (const name of fs.readdirSync(path.join(root, relative)).sort()) {
      if (omit.has(name)) continue;
      const rel = path.join(relative, name), full = path.join(root, rel), stat = fs.lstatSync(full);
      if (stat.isSymbolicLink() || (!stat.isFile() && !stat.isDirectory())) fail('URP001', `non-regular input: ${full}`);
      if (stat.isDirectory()) walk(rel);
      else result.push({ path: rel.replaceAll(path.sep, '/'), sha256: sha(fs.readFileSync(full)), bytes: stat.size });
    }
  }
  walk('');
  return result;
}
export function copySource(source, destination) {
  fs.mkdirSync(destination, { recursive: true });
  for (const row of files(source, OMIT)) {
    const target = path.join(destination, row.path);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.copyFileSync(path.join(source, row.path), target);
    fs.chmodSync(target, fs.statSync(path.join(source, row.path)).mode & 0o777);
  }
}
export function cleanOutputs(root) {
  for (const name of ['target', '.lake', '.lexlean', '.prism']) {
    if (fs.existsSync(path.join(root, name))) fail('URP002', `clean build already contains ${name}`);
  }
}
export function compare(left, right) {
  if (canonical(left) !== canonical(right)) fail('URP003', 'canonical artifact inventory or bytes differ');
}
function run(root, command, args, options = {}) {
  const result = spawnSync(command, args, { cwd: root, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024,
    env: { ...process.env, ...options.env }, timeout: 30 * 60 * 1000 });
  if (options.log) {
    fs.mkdirSync(path.dirname(options.log), { recursive: true });
    fs.writeFileSync(options.log, `$ ${command} ${args.join(' ')}\n${result.stdout ?? ''}${result.stderr ?? ''}`);
  }
  if (result.error) fail('URX001', `${command}: ${result.error.message}`);
  if (!options.failure && result.status !== 0) fail('URX001', `${command} ${args.join(' ')} exited ${result.status}\n${result.stdout}\n${result.stderr}`);
  return result;
}
function sdk(root) {
  // An environment variable or a fixture inventory cannot establish SDK identity.
  if (!fs.existsSync(INVENTORY) || !fs.existsSync(SDK)) fail('URD001', 'the digest-selected SDK is required');
  return JSON.parse(run(root, SDK, ['--json', '--project', root, 'lock', 'check']).stdout);
}
function projectionPaths(root) {
  const paths = JSON.parse(run(root, 'cargo', ['run', '--locked', '-q', '-p', 'xtask', '--', 'projection-inventory']).stdout);
  if (!Array.isArray(paths) || !paths.length || new Set(paths).size !== paths.length || paths.some(p => typeof p !== 'string' || path.isAbsolute(p) || p.split('/').includes('..'))) {
    fail('URP004', 'invalid source-derived projection inventory');
  }
  return paths;
}
function projections(root, paths) {
  return paths.map(relative => {
    const full = path.join(root, relative);
    if (!fs.existsSync(full) || !fs.lstatSync(full).isFile()) fail('URP004', `missing projection ${relative}`);
    return { path: relative, sha256: sha(fs.readFileSync(full)), bytes: fs.statSync(full).size };
  }).sort((a, b) => a.path.localeCompare(b.path));
}
function lexlean(root, command) {
  const result = JSON.parse(run(root, 'lexlean', ['--diagnostic-format', 'json', command, '--all']).stdout);
  if (result.spec !== 'lexlean/command-result/1' || !result.success || result.exit_code !== 0 || !Array.isArray(result.modules) || !result.modules.length) {
    fail('URP004', `incomplete ${command} result`);
  }
  return result;
}
export function artifactInventory(root, projectionList, build, verified) {
  for (const [result, field] of [[build, 'build_id'], [verified, 'attestation_id']]) {
    if (!/^[a-f0-9]{64}$/.test(result[field] ?? '')) fail('URP004', `missing ${field}`);
  }
  if (build.build_id !== verified.build_id) fail('URP004', 'verification belongs to a different build');
  const expected = [`.lexlean/build/${build.build_id}`, `.lexlean/verified/${verified.attestation_id}`];
  if (canonical(build.artifacts) !== canonical([expected[0]]) || canonical(verified.artifacts) !== canonical([expected[1]])) {
    fail('URP004', 'SDK returned an unexpected artifact path');
  }
  const rows = projections(root, projectionList);
  for (const relative of expected) {
    const tree = files(path.join(root, relative));
    if (!tree.length) fail('URP004', `empty artifact tree ${relative}`);
    for (const row of tree) {
      const bytes = fs.readFileSync(path.join(root, relative, row.path));
      if (bytes.includes(Buffer.from(root))) fail('URP003', `absolute checkout path in ${relative}/${row.path}`);
      rows.push({ ...row, path: `${relative}/${row.path}` });
    }
  }
  // No unlisted content-addressed build/verification may masquerade as current evidence.
  for (const [relative, id] of [['.lexlean/build', build.build_id], ['.lexlean/verified', verified.attestation_id]]) {
    compare(fs.readdirSync(path.join(root, relative)).sort(), [id]);
  }
  return { schema: 'uorc/canonical-artifacts/1', normalization: [], build, verified, files: rows.sort((a, b) => a.path.localeCompare(b.path)) };
}
function build(root, projectionList) {
  const previousTarget = process.env.CARGO_TARGET_DIR;
  process.env.CARGO_TARGET_DIR = path.join(root, 'target');
  try {
  cleanOutputs(root);
  sdk(root);
  run(root, 'just', ['model-write']);
  run(root, 'lexlean', ['lock', '--check']);
  run(root, 'lexlean', ['fmt', '--check', '--all']);
  run(root, 'lexlean', ['check', '--all']);
  const built = lexlean(root, 'build'), verified = lexlean(root, 'verify');
  return artifactInventory(root, projectionList, built, verified);
  } finally {
    if (previousTarget === undefined) delete process.env.CARGO_TARGET_DIR;
    else process.env.CARGO_TARGET_DIR = previousTarget;
  }
}
function requireRejected(root, command, args, pattern, env = {}) {
  const result = run(root, command, args, { failure: true, env });
  const output = `${result.stdout}\n${result.stderr}`;
  if (result.status === 0 || !pattern.test(output)) fail('URP005', `planted defect did not fail at its owning gate: ${command} ${args.join(' ')}\n${output}`);
  return { command: [command, ...args], exit_code: result.status, diagnostic: output };
}
export function negativeSdk(root, temporary) {
  const planted = path.join(temporary, 'negative');
  copySource(root, planted);
  const lock = fs.readFileSync(path.join(planted, 'prismpm.lock'));
  const calls = [], args = ['--json', '--project', planted, 'lock', 'check'];
  // A valid adjacent checkout is deliberately available while the local lock is missing.
  const adjacent = path.join(temporary, 'adjacent');
  copySource(root, adjacent);
  fs.unlinkSync(path.join(planted, 'prismpm.lock'));
  calls.push(requireRejected(planted, SDK, args, /PP5401/));
  fs.symlinkSync(path.join(adjacent, 'prismpm.lock'), path.join(planted, 'prismpm.lock'));
  calls.push(requireRejected(planted, SDK, args, /PP5401/));
  fs.unlinkSync(path.join(planted, 'prismpm.lock'));
  fs.writeFileSync(path.join(planted, 'prismpm.lock'), lock);
  const wrong = JSON.parse(lock);
  wrong.platforms[0].inventory_digest = 'sha256:' + '0'.repeat(64);
  fs.writeFileSync(path.join(planted, 'prismpm.lock'), canonical(wrong));
  calls.push(requireRejected(planted, SDK, args, /PP5401|PP7601/));
  fs.writeFileSync(path.join(planted, 'prismpm.lock'), lock);
  const lexLock = fs.readFileSync(path.join(planted, 'lexlean.lock'));
  fs.unlinkSync(path.join(planted, 'lexlean.lock'));
  calls.push(requireRejected(planted, 'lexlean', ['lock', '--check'], /LLC0102/));
  fs.writeFileSync(path.join(planted, 'lexlean.lock'), lexLock.toString('utf8').replace(/compiler_semantics = "[a-f0-9]{64}"/, `compiler_semantics = "${'0'.repeat(64)}"`));
  calls.push(requireRejected(planted, 'lexlean', ['lock', '--check'], /LLC0102/));
  fs.writeFileSync(path.join(planted, 'lexlean.lock'), lexLock);
  const cargoLock = fs.readFileSync(path.join(planted, 'Cargo.lock'));
  fs.unlinkSync(path.join(planted, 'Cargo.lock'));
  calls.push(requireRejected(planted, 'cargo', ['metadata', '--locked', '--offline', '--format-version', '1'], /lock file|lockfile|Cargo.lock/));
  fs.writeFileSync(path.join(planted, 'Cargo.lock'), cargoLock);
  const toolDir = path.join(temporary, 'host-tools');
  fs.mkdirSync(toolDir);
  const marker = path.join(temporary, 'host-tool-ran');
  fs.writeFileSync(path.join(toolDir, 'cargo'), `#!/bin/sh\nprintf planted > '${marker}'\nexit 0\n`, { mode: 0o755 });
  calls.push(requireRejected(planted, SDK, args, /PP5401/, { PATH: `${toolDir}:${process.env.PATH}` }));
  calls.push(requireRejected(planted, SDK, args, /PP5401/, { PATH: toolDir }));
  if (fs.existsSync(marker)) fail('URP005', 'a planted host tool executed');
  // The unmodified control must still pass after every defect has been removed.
  sdk(planted);
  return calls;
}
function reproduce(root) {
  sdk(root);
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'uorc-reproduce-'));
  try {
    const paths = projectionPaths(root), original = projections(root, paths);
    const left = path.join(temporary, 'left'), right = path.join(temporary, 'different', 'right-longer');
    copySource(root, left); copySource(root, right);
    const a = build(left, paths), b = build(right, paths);
    compare(a, b); compare(projections(left, paths), original);
    const negatives = negativeSdk(root, temporary);
    const report = { schema: 'uorc/reproduction/1', artifacts: a, negative_executions: negatives.length, normalization: [] };
    writeJson(path.join(root, '.prism/uorc/reproduction.json'), report);
    writeJson(path.join(root, '.prism/uorc/negative-executions.json'), negatives);
    console.log(`Two clean roots: ${a.files.length} identical canonical artifacts; ${negatives.length} rejected dependency/tool defects.`);
  } finally { fs.rmSync(temporary, { recursive: true, force: true }); }
}
export function checkSetup(root, acquisition) {
  const receiptPath = path.join(acquisition, 'setup.json');
  if (!fs.existsSync(receiptPath)) fail('URD001', 'explicit dependency setup is missing');
  const receipt = readJson(receiptPath);
  if (receipt.schema !== 'uorc/dependency-setup/1' || receipt.sdk_image !== readJson(path.join(root, 'prismpm.lock')).sdk_image
    || receipt.source_sha256 !== sha(canonical(files(root, OMIT)))) fail('URD002', 'dependency setup is stale for this source or SDK');
  if (canonical(receipt.cache) !== canonical(files(path.join(acquisition, 'cache')))) fail('URD003', 'dependency acquisition cache is incomplete or changed');
  return receipt;
}
function setup(root, acquisition) {
  const lock = sdk(root), cache = path.join(acquisition, 'cache');
  if (fs.existsSync(path.join(acquisition, 'setup.json'))) fail('URD002', 'setup destination is already sealed');
  fs.mkdirSync(cache, { recursive: true });
  fs.cpSync('/opt/prismpm/cargo-home', path.join(cache, 'cargo'), { recursive: true });
  function writable(directory) {
    fs.chmodSync(directory, 0o700);
    for (const item of fs.readdirSync(directory)) {
      const child = path.join(directory, item);
      if (fs.lstatSync(child).isDirectory()) writable(child);
      else fs.chmodSync(child, 0o600);
    }
  }
  writable(cache);
  const env = cacheEnvironment(cache);
  fs.mkdirSync(env.HOME, { recursive: true }); fs.mkdirSync(env.XDG_CACHE_HOME, { recursive: true });
  run(root, 'cargo', ['fetch', '--locked'], { env, log: path.join(acquisition, 'fetch.log') });
  run(root, 'cargo', ['deny', '--locked', '--all-features', 'fetch'], { env, log: path.join(acquisition, 'deny-fetch.log') });
  // Acquisition never compiles, generates, or verifies UORC outputs.
  cleanOutputs(root);
  writeJson(path.join(acquisition, 'setup.json'), { schema: 'uorc/dependency-setup/1', sdk_image: lock.sdk_image,
    sdk_inventory_sha256: sha(fs.readFileSync(INVENTORY)), source_sha256: sha(canonical(files(root, OMIT))), cache: files(cache) });
}
function cacheEnvironment(cache) {
  return { CARGO_HOME: path.join(cache, 'cargo'), HOME: path.join(cache, 'home'), XDG_CACHE_HOME: path.join(cache, 'xdg') };
}
export function offlineBoundary(interfaces, socketExists) {
  if (Object.keys(interfaces).some(name => name !== 'lo')) fail('URD004', 'offline verification has a non-loopback network interface');
  if (socketExists) fail('URD004', 'offline verification exposes a Docker daemon socket');
}
function offline(root, acquisition) {
  const receipt = checkSetup(root, acquisition);
  sdk(root);
  if (receipt.sdk_inventory_sha256 !== sha(fs.readFileSync(INVENTORY))) fail('URD002', 'setup used a different executing SDK');
  offlineBoundary(os.networkInterfaces(), fs.existsSync('/var/run/docker.sock'));
  cleanOutputs(root);
  const cache = fs.mkdtempSync(path.join(os.tmpdir(), 'uorc-acquired-'));
  fs.cpSync(path.join(acquisition, 'cache'), cache, { recursive: true });
  const env = { ...cacheEnvironment(cache), CARGO_NET_OFFLINE: 'true', CARGO_INCREMENTAL: '0', CARGO_TARGET_DIR: path.join(root, 'target'),
    SOURCE_DATE_EPOCH: '0', TZ: 'UTC', LC_ALL: 'C.UTF-8' };
  Object.assign(process.env, env);
  // Current-build verification is always executed, never accepted from a cache receipt.
  run(root, 'just', ['vv'], { env, log: path.join(root, '.prism/uorc/full-vv.log') });
  const paths = projectionPaths(root), built = lexlean(root, 'build'), verified = lexlean(root, 'verify');
  writeJson(path.join(root, '.prism/uorc/offline-artifacts.json'), artifactInventory(root, paths, built, verified));
  writeJson(path.join(root, '.prism/uorc/offline-execution.json'), { schema: 'uorc/offline-execution/1', source_sha256: receipt.source_sha256,
    sdk_image: receipt.sdk_image, sdk_inventory_sha256: receipt.sdk_inventory_sha256, network: 'none', docker_socket: false,
    command: ['just', 'vv'], exit_code: 0, acquisition_sha256: sha(canonical(receipt.cache)), normalization: [] });
}
function docker(root, image, args, command) {
  return run(root, 'docker', ['run', '--rm', '--pull=never', '--user', `${process.getuid()}:${process.getgid()}`,
    '--cap-drop=ALL', '--security-opt=no-new-privileges', '--tmpfs', '/tmp:rw,exec,nosuid,size=4g', '--entrypoint', '/usr/local/bin/node',
    ...args, image, ...command]);
}
function isolated(root) {
  const lock = sdk(root);
  // This directory is source-excluded and is on the same host-preserved bind mount.
  // Docker's daemon cannot see a temporary directory in the orchestration container.
  fs.mkdirSync(path.join(root, '.tmp'), { recursive: true });
  const workOutside = fs.mkdtempSync(path.join(root, '.tmp', 'uorc-isolated-'));
  try {
    const source = path.join(workOutside, 'setup-source'), acquisition = path.join(workOutside, 'acquisition');
    copySource(root, source); cleanOutputs(source); fs.mkdirSync(acquisition);
    const script = path.join(source, 'scripts/offline-validation.mjs');
    try {
      docker(root, lock.sdk_image, ['--volume', `${source}:${source}`, '--volume', `${acquisition}:${acquisition}`, '--workdir', source], [script, 'setup', acquisition]);
    } finally {
      fs.mkdirSync(path.join(root, '.prism/uorc/setup'), { recursive: true });
      for (const name of ['fetch.log', 'deny-fetch.log', 'setup.json']) {
        if (fs.existsSync(path.join(acquisition, name))) fs.copyFileSync(path.join(acquisition, name), path.join(root, '.prism/uorc/setup', name));
      }
    }
    const results = [];
    for (const label of ['first', 'second-distinct-root']) {
      const buildRoot = path.join(workOutside, label); copySource(root, buildRoot);
      try {
        docker(root, lock.sdk_image, ['--network=none', '--volume', `${buildRoot}:${buildRoot}`, '--volume', `${acquisition}:${acquisition}:ro`, '--workdir', buildRoot],
          [path.join(buildRoot, 'scripts/offline-validation.mjs'), 'offline', acquisition]);
      } finally {
        if (fs.existsSync(path.join(buildRoot, '.prism/uorc'))) {
          fs.mkdirSync(path.join(root, '.prism/uorc', label), { recursive: true });
          fs.cpSync(path.join(buildRoot, '.prism/uorc'), path.join(root, '.prism/uorc', label), { recursive: true });
        }
      }
      results.push({ artifacts: readJson(path.join(buildRoot, '.prism/uorc/offline-artifacts.json')), execution: readJson(path.join(buildRoot, '.prism/uorc/offline-execution.json')) });
      fs.mkdirSync(path.join(root, '.prism/uorc', label), { recursive: true });
      fs.cpSync(path.join(buildRoot, '.prism/uorc'), path.join(root, '.prism/uorc', label), { recursive: true });
    }
    compare(results[0], results[1]);
    // Setup failures are executed inside the same offline SDK, before any build.
    for (const defect of ['missing', 'stale', 'damaged-cache']) {
      const planted = path.join(workOutside, `setup-${defect}`); fs.cpSync(acquisition, planted, { recursive: true });
      if (defect === 'missing') fs.unlinkSync(path.join(planted, 'setup.json'));
      if (defect === 'stale') { const data = readJson(path.join(planted, 'setup.json')); data.source_sha256 = '0'.repeat(64); writeJson(path.join(planted, 'setup.json'), data); }
      if (defect === 'damaged-cache') fs.writeFileSync(path.join(planted, 'cache/planted'), 'bad');
      const result = run(root, 'docker', ['run', '--rm', '--pull=never', '--network=none', '--user', `${process.getuid()}:${process.getgid()}`,
        '--cap-drop=ALL', '--security-opt=no-new-privileges', '--entrypoint', '/usr/local/bin/node', '--volume', `${source}:${source}:ro`,
        '--volume', `${planted}:${planted}:ro`, '--workdir', source, lock.sdk_image, script, 'check-setup', planted], { failure: true });
      const expected = { missing: 'URD001', stale: 'URD002', 'damaged-cache': 'URD003' }[defect];
      if (result.status === 0 || !`${result.stdout}${result.stderr}`.includes(expected)) fail('URP005', `setup defect ${defect} was not rejected: ${result.stderr}`);
      writeJson(path.join(root, '.prism/uorc', `negative-setup-${defect}.json`), { defect, code: expected, exit_code: result.status, stderr: result.stderr });
    }
    checkSetup(source, acquisition);
    writeJson(path.join(root, '.prism/uorc/offline-comparison.json'), { schema: 'uorc/offline-comparison/1', status: 'passed',
      distinct_roots: true, shared_build_outputs: false, normalization: [], artifacts: results[0].artifacts, execution: results[0].execution });
    console.log('Two full offline just vv runs passed with byte-identical artifacts and rejected setup negatives.');
  } finally { fs.rmSync(workOutside, { recursive: true, force: true }); }
}
export function main(args) {
  const arity = { reproduce: 1, isolated: 1, deny: 1, setup: 2, offline: 2, 'check-setup': 2, 'export-projections': 2 };
  if (!Object.hasOwn(arity, args[0]) || args.length !== arity[args[0]]) fail('URX002', 'invalid operation or argument count');
  const root = fs.realpathSync(process.cwd());
  switch (args[0]) {
    case 'reproduce': return reproduce(root);
    case 'deny':
      sdk(root);
      return run(root, 'cargo', ['deny', process.env.CARGO_NET_OFFLINE === 'true' ? '--frozen' : '--locked', '--all-features', 'check']);
    case 'isolated': return isolated(root);
    case 'setup': return setup(root, path.resolve(args[1]));
    case 'offline': return offline(root, path.resolve(args[1]));
    case 'check-setup': return checkSetup(root, path.resolve(args[1]));
    case 'export-projections':
      for (const relative of projectionPaths(root)) {
        const destination = path.join(path.resolve(args[1]), relative);
        fs.mkdirSync(path.dirname(destination), { recursive: true });
        fs.copyFileSync(path.join(root, relative), destination);
      }
      return;
    default: fail('URX002', 'expected reproduce, isolated, setup, offline, or check-setup');
  }
}
if (process.argv[1] && fs.realpathSync(process.argv[1]) === SCRIPT) {
  try { main(process.argv.slice(2)); } catch (error) {
    const failure = error instanceof ValidationFailure ? error : new ValidationFailure('URX001', error.message);
    console.error(failure.message); process.exitCode = 1;
  }
}
