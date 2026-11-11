// The Pages site is a browser closure v2: built from web/ and the vendored
// brand files, it holds exactly the sixteen closure files (plus the bench
// page), flat, with provenance.json naming every other file's SHA-256 and no
// page loading anything remote or outside the site.
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { test } from 'node:test';
import { build, check, closureV2, references, webManifest } from './site.mjs';

const scratch = () => fs.mkdtempSync(path.join(os.tmpdir(), 'uorc-site-'));
const sha256 = (bytes) => crypto.createHash('sha256').update(bytes).digest('hex');

function fixture() {
  const dir = scratch();
  const wasm = path.join(dir, 'uorc_wasm.wasm');
  fs.writeFileSync(wasm, Buffer.from([0, 0x61, 0x73, 0x6d, 1, 0, 0, 0]));
  const bench = path.join(dir, 'bench');
  fs.mkdirSync(bench);
  fs.writeFileSync(path.join(bench, 'bench.html'), '<!doctype html><link rel="stylesheet" href="app.css"><a href="bench.json">raw</a> <a href="./">client</a>');
  fs.writeFileSync(path.join(bench, 'bench.json'), '{}\n');
  fs.writeFileSync(path.join(bench, 'bench.md'), '# bench\n');
  return { dir, wasm, bench };
}

test('the site is exactly closure v2 plus the bench page, flat', () => {
  const { dir, wasm, bench } = fixture();
  const out = path.join(dir, 'site');
  const provenance = build(out, { wasm, version: 'v0.0.0-test', bench });
  const names = fs.readdirSync(out).sort();
  assert.equal(closureV2().length, 16);
  assert.deepEqual(names, [...closureV2(), 'bench.html', 'bench.json', 'bench.md'].sort());
  for (const name of Object.keys(provenance.files)) {
    assert.equal(provenance.files[name], sha256(fs.readFileSync(path.join(out, name))), name);
  }
  assert.equal(JSON.parse(fs.readFileSync(path.join(out, 'provenance.json'))).version, 'v0.0.0-test');
  const manifest = JSON.parse(fs.readFileSync(path.join(out, 'manifest.webmanifest')));
  assert.equal(manifest.name, 'uorc');
  assert.equal(manifest.display, 'standalone');
  for (const icon of manifest.icons) assert.ok(names.includes(icon.src), icon.src);
});

test('the page loads only same-origin closure files', () => {
  const { local, remote } = references(fs.readFileSync(new URL('../web/index.html', import.meta.url), 'utf8')
    + fs.readFileSync(new URL('../web/app.css', import.meta.url), 'utf8')
    + fs.readFileSync(new URL('../web/app.js', import.meta.url), 'utf8')
    + fs.readFileSync(new URL('../web/uorc.js', import.meta.url), 'utf8'));
  for (const name of ['manifest.webmanifest', 'icon.svg', 'icon-512.png', 'Geist-Regular.woff2', 'uorc_bg.wasm', 'provenance.json']) {
    assert.ok(local.includes(name), name);
  }
  assert.ok(remote.every((r) => r.startsWith('https://github.com/')), remote.join(', '));
});

test('a site that is not closure v2 is refused', () => {
  const { dir, wasm, bench } = fixture();
  const out = path.join(dir, 'site');
  build(out, { wasm, version: 'v', bench });
  const files = new Map(fs.readdirSync(out).map((n) => [n, fs.readFileSync(path.join(out, n))]));
  const without = new Map(files); without.delete('icon.svg');
  assert.throws(() => check(without), /not closure v2/);
  const extra = new Map(files); extra.set('extra.js', Buffer.from(''));
  assert.throws(() => check(extra), /not closure v2/);
  const tampered = new Map(files); tampered.set('app.js', Buffer.concat([files.get('app.js'), Buffer.from('\n')]));
  assert.throws(() => check(tampered), /misnames app.js/);
  const remote = new Map(files); remote.set('index.html', Buffer.from('<script src="https://cdn.example/x.js"></script>'));
  assert.throws(() => check(remote), /references https:\/\/cdn.example/);
  const missing = new Map(files); missing.set('index.html', Buffer.from('<img src="logo.png">'));
  assert.throws(() => check(missing), /logo.png/);
});

test('the web app manifest is the bytes PrismPM writes', () => {
  assert.equal(webManifest('A').toString(), '{"name":"A","short_name":"A","start_url":"./","scope":"./","display":"standalone","background_color":"#151312","theme_color":"#151312","icons":[{"src":"icon.svg","sizes":"any","type":"image/svg+xml","purpose":"any"},{"src":"icon-512.png","sizes":"512x512","type":"image/png","purpose":"any maskable"}]}\n');
});
