import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {test} from 'node:test';
import {fileURLToPath} from 'node:url';
import {matching, register} from './corpora.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

test('reads the public corpora and their pinned members from frontier.toml', () => {
  const corpora = register(fs.readFileSync(path.join(ROOT, 'frontier.toml'), 'utf8'));
  const byName = Object.fromEntries(corpora.map(c => [c.name, c]));
  assert.equal(byName.canterbury.members.length, 11);
  assert.equal(byName.silesia.members.length, 12);
  assert.match(byName.canterbury.source, /^https:/);
  for (const m of [...byName.canterbury.members, ...byName.silesia.members]) {
    assert.match(m.sha256, /^[0-9a-f]{64}$/);
    assert.ok(Number.isInteger(m.bytes) && m.bytes > 0);
  }
});

test('a member counts only with its pinned digest and size', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'corpora-test-'));
  const good = Buffer.from('pinned bytes');
  fs.writeFileSync(path.join(dir, 'a'), good);
  fs.writeFileSync(path.join(dir, 'b'), 'other bytes');
  const corpus = {members: [
    {name: 'a', bytes: good.length, sha256: crypto.createHash('sha256').update(good).digest('hex')},
    {name: 'b', bytes: 11, sha256: crypto.createHash('sha256').update(good).digest('hex')},
    {name: 'c', bytes: 1, sha256: '0'.repeat(64)},
  ]};
  assert.deepEqual(matching(corpus, dir), ['a']);
  fs.rmSync(dir, {recursive: true, force: true});
});
