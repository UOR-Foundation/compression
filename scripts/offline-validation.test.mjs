import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { files, copySource, cleanOutputs, compare, artifactInventory, checkSetup, offlineBoundary, canonical, main, sha } from './offline-validation.mjs';

function temporary(t) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'uorc-validation-test-'));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  return root;
}
function put(root, name, bytes = 'canonical') {
  const file = path.join(root, name); fs.mkdirSync(path.dirname(file), { recursive: true }); fs.writeFileSync(file, bytes);
}
function fixture(t) {
  const root = temporary(t), id = 'a'.repeat(64), attestation = 'b'.repeat(64);
  put(root, 'model/ids.toml'); put(root, `.lexlean/build/${id}/modules/Uorc.lean`);
  put(root, `.lexlean/verified/${attestation}/attestation.json`, '{}');
  return { root, projections: ['model/ids.toml'], build: { build_id: id, artifacts: [`.lexlean/build/${id}`] },
    verified: { build_id: id, attestation_id: attestation, artifacts: [`.lexlean/verified/${attestation}`] } };
}
const inventory = f => artifactInventory(f.root, f.projections, f.build, f.verified);

test('all canonical bytes and filenames must agree', () => {
  const rows = [{ path: 'a', sha256: sha('one'), bytes: 3 }];
  compare(rows, structuredClone(rows));
  assert.throws(() => compare(rows, []), /URP003/);
  assert.throws(() => compare(rows, [{ ...rows[0], path: 'b' }]), /URP003/);
  assert.throws(() => compare(rows, [{ ...rows[0], sha256: sha('two') }]), /URP003/);
});
test('source snapshots exclude all acquisition and build output directories', t => {
  const root = temporary(t), target = temporary(t); put(root, 'src/authority.lex.tex');
  for (const directory of ['target', '.lexlean', '.lake', '.prism', '.tmp', 'node_modules', 'reports']) put(root, `${directory}/planted`);
  copySource(root, target); assert.deepEqual(files(target).map(row => row.path), ['src/authority.lex.tex']); cleanOutputs(target);
});
test('symlinks cannot import adjacent source or cached output', t => {
  const root = temporary(t), adjacent = temporary(t); put(adjacent, 'foreign');
  fs.symlinkSync(path.join(adjacent, 'foreign'), path.join(root, 'import'));
  assert.throws(() => copySource(root, temporary(t)), /URP001/);
});
test('prepopulated proof and application output is rejected', t => {
  for (const directory of ['target', '.lexlean', '.lake', '.prism']) {
    const root = temporary(t); put(root, `${directory}/planted`); assert.throws(() => cleanOutputs(root), /URP002/);
  }
});
test('artifact inventory includes every projection, generated and verified byte', t => {
  const f = fixture(t); assert.equal(inventory(f).files.length, 3); assert.deepEqual(inventory(f).normalization, []);
  put(f.root, `${f.build.artifacts[0]}/extra`, 'new'); assert.equal(inventory(f).files.length, 4);
});
test('missing projections cannot produce vacuous artifact identity', t => {
  const f = fixture(t); fs.unlinkSync(path.join(f.root, 'model/ids.toml')); assert.throws(() => inventory(f), /URP004/);
});
test('artifact and projection parent symlinks cannot import adjacent evidence', t => {
  for (const relative of ['model', '.lexlean', '.lexlean/build', `.lexlean/build/${'a'.repeat(64)}`, '.lexlean/verified', `.lexlean/verified/${'b'.repeat(64)}`]) {
    const f = fixture(t), adjacent = temporary(t), original = path.join(f.root, relative);
    const held = path.join(adjacent, 'held');
    fs.renameSync(original, held);
    fs.symlinkSync(held, original);
    assert.throws(() => inventory(f), /URP001/, relative);
    fs.unlinkSync(original);
    fs.renameSync(held, original);
    assert.equal(inventory(f).files.length, 3);
  }
});
test('projection inventories reject omitted, duplicate and escaping paths', t => {
  for (const paths of [[], ['model/ids.toml', 'model/ids.toml'], ['../outside'], ['/tmp/outside'], ['model/../model/ids.toml'], ['']]) {
    const f = fixture(t); f.projections = paths;
    assert.throws(() => inventory(f), /URP004/);
  }
});
test('verification must bind the current build and expected artifact path', t => {
  const f = fixture(t); f.verified.build_id = 'c'.repeat(64); assert.throws(() => inventory(f), /URP004/);
  f.verified.build_id = f.build.build_id; f.verified.artifacts = [f.build.artifacts[0]]; assert.throws(() => inventory(f), /URP004/);
});
test('old output trees and absolute checkout paths are never normalized away', t => {
  const f = fixture(t); put(f.root, '.lexlean/verified/old/evidence'); assert.throws(() => inventory(f), /URP003/);
  fs.rmSync(path.join(f.root, '.lexlean/verified/old'), { recursive: true });
  put(f.root, `${f.build.artifacts[0]}/absolute`, f.root); assert.throws(() => inventory(f), /URP003/);
});
test('missing setup fails explicitly before verification', t => {
  const root = temporary(t); assert.throws(() => checkSetup(root, temporary(t)), /URD001/);
});
test('stale setup cannot be recycled for another source or SDK', t => {
  const root = temporary(t), acquisition = temporary(t);
  put(root, 'prismpm.lock', JSON.stringify({ sdk_image: 'example@sha256:' + 'a'.repeat(64) }));
  put(acquisition, 'setup.json', JSON.stringify({ schema: 'uorc/dependency-setup/1', sdk_image: 'wrong', source_sha256: 'bad' }));
  assert.throws(() => checkSetup(root, acquisition), /URD002/);
});
test('normative gate retains reproduction and full formal verification', () => {
  const just = fs.readFileSync(new URL('../Justfile', import.meta.url), 'utf8');
  const vv = just.split('\n').find(line => line.startsWith('vv:'));
  for (const gate of ['lexlean-authority', 'template-check', 'fmt-check', 'model', 'lint', 'test', 'features', 'bdd', 'deny', 'lexlean-artifacts', 'reproducibility']) {
    assert.ok(vv.split(/\s+/).includes(gate), `${gate} omitted from just vv`);
  }
  assert.match(just, /lexlean-artifacts:\n    lexlean build --all\n    lexlean verify --all/);
  assert.match(just, /reproducibility:\n    node scripts\/offline-validation\.mjs reproduce/);
});

test('damaged cache cannot be hidden behind a correct setup identity', t => {
  const root = temporary(t), acquisition = temporary(t);
  const image = 'example@sha256:' + 'a'.repeat(64);
  put(root, 'prismpm.lock', JSON.stringify({ sdk_image: image }));
  put(acquisition, 'cache/package', 'original');
  put(acquisition, 'setup.json', JSON.stringify({ schema: 'uorc/dependency-setup/1', sdk_image: image,
    source_sha256: sha(canonical(files(root))), cache: files(path.join(acquisition, 'cache')) }));
  checkSetup(root, acquisition);
  put(acquisition, 'cache/package', 'changed');
  assert.throws(() => checkSetup(root, acquisition), /URD003/);
});
test('network interfaces and daemon escape hatches reject an offline claim', () => {
  offlineBoundary({ lo: [] }, false);
  assert.throws(() => offlineBoundary({ lo: [], eth0: [] }, false), /URD004/);
  assert.throws(() => offlineBoundary({ lo: [] }, true), /URD004/);
});

test('invalid operations and missing or extra arguments are typed', () => {
  for (const args of [[], ['unknown'], ['setup'], ['offline'], ['check-setup'], ['reproduce', 'ignored']]) {
    assert.throws(() => main(args), /URX002/);
  }
});
test('malformed filesystem data produces a typed public execution failure', t => {
  const acquisition = temporary(t); put(acquisition, 'setup.json', 'not json');
  const result = spawnSync(process.execPath, [new URL('./offline-validation.mjs', import.meta.url).pathname, 'check-setup', acquisition], { encoding: 'utf8' });
  assert.notEqual(result.status, 0); assert.match(result.stderr, /URX001/);
});
