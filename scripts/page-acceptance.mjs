#!/usr/bin/env node
// UC-WEB-01: the browser acceptance of the Pages client against its Prism
// product model (src/Uorc/Page.lex.tex, carried as JSON by the built page).
//
// It serves a built site, drives headless Chromium over the DevTools protocol
// (no dependency beyond Node), walks every journey of the model in a fresh
// page, runs each feature's acceptance cases, records every state transition
// and checks every control's enabled state in every state reached, and holds
// every gated quality measure to its threshold. Measured-only measures are
// reported. Any failure exits 1.
//
//   node scripts/page-acceptance.mjs <site> [--chrome <path>] [--report <file>]
import { spawn } from 'node:child_process';
import crypto from 'node:crypto';
import fs from 'node:fs';
import http from 'node:http';
import os from 'node:os';
import path from 'node:path';

const [site, ...rest] = process.argv.slice(2);
const opt = (key) => { const i = rest.indexOf(key); return i < 0 ? undefined : rest[i + 1]; };
if (!site) { console.error('usage: node scripts/page-acceptance.mjs <site> [--chrome <path>] [--report <file>]'); process.exit(2); }
const chrome = opt('--chrome') ?? process.env.CHROME ?? ['/usr/bin/google-chrome', '/usr/bin/chromium', '/usr/bin/chromium-browser', '/opt/pw-browsers/chromium'].find((p) => fs.existsSync(p));
if (!chrome) { console.error('page acceptance: no Chrome or Chromium found; pass --chrome'); process.exit(2); }

const html = fs.readFileSync(path.join(site, 'index.html'), 'utf8');
const model = JSON.parse(html.match(/<script type="application\/json" id="page-model">(.*?)<\/script>/s)[1].replace(/<\\\//g, '</'));
const sha = (bytes) => crypto.createHash('sha256').update(bytes).digest('hex');
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// Inputs.
const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'uorc-page-'));
const words = 'archive frame object reference integrity window decode encode model stream byte seal header context mixer weight'.split(' ');
let seed = 1;
const prose = Array.from({ length: 40000 }, () => { seed = (seed * 1103515245 + 12345) >>> 0; return words[seed % words.length]; }).join(' ');
// Sentences of words, `n` bytes.
function passage(n) {
  const vocabulary = ['frame', 'archive', 'object', 'label', 'run', 'unit', 'basis', 'window', 'model', 'page'];
  let x = 12345, s = '';
  while (s.length < n) { x = (x * 1103515245 + 12345) >>> 0; s += vocabulary[x % vocabulary.length] + ((x >> 8) % 7 ? ' ' : '.\n'); }
  return s.slice(0, n);
}
const file = (name, bytes) => { const p = path.join(dir, name); fs.mkdirSync(path.dirname(p), { recursive: true }); fs.writeFileSync(p, bytes); return p; };
const inputs = {
  text: file('text.txt', Buffer.from(prose)),
  other: file('other.bin', crypto.randomBytes(3000)),
  // Three 8 MiB frames of one repeated passage: the first is coded, the
  // repeats after it are reference frames.
  frames: file('frames.bin', Buffer.alloc(3 * 8 * 1048576, passage(1048576))),
  folder: path.join(dir, 'folder'),
};
file('folder/a.txt', Buffer.from('alpha\n'));
file('folder/sub/b.bin', crypto.randomBytes(700));

// The site, and the same site without its WebAssembly module.
const types = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.wasm': 'application/wasm', '.json': 'application/json', '.svg': 'image/svg+xml', '.png': 'image/png', '.woff2': 'font/woff2', '.webmanifest': 'application/manifest+json' };
const server = http.createServer((req, res) => {
  let url = decodeURIComponent(req.url.split('?')[0]);
  const broken = url.startsWith('/no-wasm/');
  if (broken) url = url.slice('/no-wasm'.length);
  const name = url === '/' ? 'index.html' : url.slice(1);
  const p = path.join(site, name);
  if (name.includes('/') || !fs.existsSync(p) || (broken && name.endsWith('.wasm'))) { res.writeHead(404); res.end(); return; }
  res.writeHead(200, { 'content-type': types[path.extname(name)] ?? 'application/octet-stream' });
  res.end(fs.readFileSync(p));
});
await new Promise((r) => server.listen(0, '127.0.0.1', r));
const origin = `http://127.0.0.1:${server.address().port}`;

// Chromium over the DevTools protocol.
const profile = fs.mkdtempSync(path.join(os.tmpdir(), 'uorc-chrome-'));
const browser = spawn(chrome, ['--headless=new', '--no-sandbox', '--disable-gpu', '--no-first-run', '--remote-debugging-port=0', `--user-data-dir=${profile}`, 'about:blank'], { stdio: ['ignore', 'ignore', 'pipe'] });
const wsUrl = await new Promise((resolve, reject) => {
  let err = '';
  browser.stderr.on('data', (d) => { err += d; const m = err.match(/DevTools listening on (ws:\/\/\S+)/); if (m) resolve(m[1]); });
  browser.on('exit', () => reject(new Error(`chrome exited: ${err}`)));
});
const ws = new WebSocket(wsUrl);
await new Promise((r) => ws.addEventListener('open', r));
let nextId = 0;
const pending = new Map();
const listeners = [];
ws.addEventListener('message', ({ data }) => {
  const msg = JSON.parse(data);
  if (msg.id !== undefined && pending.has(msg.id)) {
    const { resolve, reject } = pending.get(msg.id);
    pending.delete(msg.id);
    if (msg.error) reject(new Error(msg.error.message)); else resolve(msg.result);
  } else for (const l of listeners) l(msg);
});
const send = (method, params = {}, sessionId) => new Promise((resolve, reject) => {
  const id = ++nextId;
  pending.set(id, { resolve, reject });
  ws.send(JSON.stringify({ id, method, params, sessionId }));
});

let page;
let pageTarget;
let workerSession;
listeners.push((msg) => {
  if (msg.method === 'Target.attachedToTarget' && msg.params.targetInfo.type === 'worker') workerSession = msg.params.sessionId;
});
const evaluate = async (expression, session = page) => {
  const r = await send('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true }, session);
  if (r.exceptionDetails) throw new Error(`${expression.slice(0, 80)}: ${r.exceptionDetails.exception?.description ?? r.exceptionDetails.text}`);
  return r.result.value;
};
const waitFor = async (expression, timeout = 120000) => {
  const end = Date.now() + timeout;
  for (;;) {
    if (await evaluate(expression)) return;
    if (Date.now() > end) throw new Error(`timed out waiting for ${expression}`);
    await sleep(50);
  }
};
const state = () => evaluate('document.body.dataset.state');

// What the page records for the acceptance: every state change with its time
// and whether the enabled controls match the model, and every long task.
const RECORDER = `(() => {
  const model = JSON.parse(document.getElementById('page-model')?.textContent ?? 'null');
  window.__acceptance = { changes: [], violations: [], longTasks: [] };
  try { new PerformanceObserver((l) => { for (const e of l.getEntries()) __acceptance.longTasks.push([e.startTime, e.duration]); }).observe({ type: 'longtask', buffered: true }); } catch {}
  const check = (s) => {
    const m = JSON.parse(document.getElementById('page-model').textContent);
    const on = (id) => m.Control.find((c) => c.id === id).enabledIn.includes(s);
    const expect = (what, el, enabled) => { if (el && el.disabled === enabled) __acceptance.violations.push(s + ': ' + what + ' is ' + (el.disabled ? 'disabled' : 'enabled')); };
    expect('cancel', document.getElementById('cancel'), on('cancel'));
    if (!on('run')) expect('run', document.getElementById('run'), false);
    for (const t of document.querySelectorAll('[role=tab]')) expect('tab ' + t.dataset.operation, t, on('operation'));
    for (const i of document.querySelectorAll('.field input')) expect('field ' + i.id, i, on('field'));
    for (const b of document.querySelectorAll('[data-example]')) expect('example ' + b.dataset.example, b, on('example'));
  };
  const start = () => {
    let last = document.body.dataset.state;
    __acceptance.changes.push([performance.now(), null, last]);
    new MutationObserver(() => {
      const s = document.body.dataset.state;
      if (s === last) return;
      __acceptance.changes.push([performance.now(), last, s]);
      last = s;
      check(s);
    }).observe(document.body, { attributes: true, attributeFilter: ['data-state'] });
  };
  if (document.body) start(); else document.addEventListener('DOMContentLoaded', start);
})();`;

const transitions = new Set();
const violations = [];
const gates = {};
const measured = {};
let longTasksSeen = 0;
const raise = (key, value) => { gates[key] = Math.max(gates[key] ?? 0, value); };

async function harvest() {
  if (!page) return;
  const a = await evaluate('window.__acceptance ?? null').catch(() => null);
  if (!a) return;
  for (const [, from, to] of a.changes) if (from) transitions.add(`${from}->${to}`);
  violations.push(...a.violations);
  // Long tasks that started while an operation ran.
  const running = [];
  a.changes.forEach(([t, , to], i) => { if (to === 'running') running.push([t, a.changes[i + 1]?.[0] ?? Infinity]); });
  longTasksSeen += a.longTasks.length;
  for (const [start, duration] of a.longTasks) if (running.some(([s, e]) => start >= s && start < e)) raise('longestPageTask', Math.round(duration));
}

async function open(url = `${origin}/`) {
  await harvest();
  if (pageTarget) await send('Target.closeTarget', { targetId: pageTarget }).catch(() => {});
  const { targetId } = await send('Target.createTarget', { url: 'about:blank' });
  const { sessionId } = await send('Target.attachToTarget', { targetId, flatten: true });
  page = sessionId;
  pageTarget = targetId;
  await send('Page.enable', {}, page);
  await send('Runtime.enable', {}, page);
  await send('Target.setAutoAttach', { autoAttach: true, waitForDebuggerOnStart: false, flatten: true }, page);
  await send('Page.addScriptToEvaluateOnNewDocument', { source: RECORDER }, page);
  const started = Date.now();
  await send('Page.navigate', { url }, page);
  await waitFor("document.body && document.body.dataset.state && document.body.dataset.state !== 'loading'", 30000);
  return Date.now() - started;
}

const setFiles = async (id, files) => {
  const { result } = await send('Runtime.evaluate', { expression: `document.getElementById(${JSON.stringify(id)})` }, page);
  await send('DOM.setFileInputFiles', { files, objectId: result.objectId }, page);
  await evaluate(`document.getElementById(${JSON.stringify(id)}).files.length`);
};
const click = (selector) => evaluate(`document.querySelector(${JSON.stringify(selector)}).click()`);
const tab = (op) => click(`#tab-${op}`);
const text = (id) => evaluate(`document.getElementById(${JSON.stringify(id)}).textContent`);
async function runAndWait() {
  const t = Date.now();
  await click('#run');
  await waitFor("document.body.dataset.state !== 'running'", 600000);
  return { state: await state(), ms: Date.now() - t };
}
// The files the result offers: name and bytes.
async function downloads() {
  const list = await evaluate(`Promise.all([...document.querySelectorAll('#result a[download]')].map(async (a) => [a.download, a.textContent, Array.from(new Uint8Array(await (await fetch(a.href)).arrayBuffer()))]))`);
  return list.map(([name, label, bytes]) => ({ name, label, bytes: Buffer.from(bytes) }));
}
const expect = (ok, what) => { if (!ok) throw new Error(what); };
async function expectHint(code) {
  const hint = model.ErrorHint.find((h) => h.code === code);
  expect(await evaluate("!document.getElementById('error').hidden"), `${code}: the error box is hidden`);
  expect((await text('error-title')) === hint.title, `${code}: title differs from the model`);
  expect((await text('error-explanation')).startsWith(hint.explanation), `${code}: explanation differs from the model`);
  expect((await text('error-recovery')) === hint.recovery, `${code}: recovery differs from the model`);
}
// The library's archive for a chosen file, computed in the page by uorc.js.
const libraryArchive = (inputId) => evaluate(`(async () => {
  const { load } = await import('./uorc.js');
  const u = await load('./uorc_bg.wasm');
  const f = document.getElementById(${JSON.stringify(inputId)}).files[0];
  return Array.from(u.compress(new Uint8Array(await f.arrayBuffer())));
})()`).then((a) => Buffer.from(a));

const saved = {};
async function compressAndCompare(input, name) {
  await tab('compress');
  await setFiles('field-files', [input]);
  const r = await runAndWait();
  expect(r.state === 'done', `compress ${name}: ${await text('status')}`);
  const [out] = await downloads();
  expect(out.name === `${path.basename(input)}.uorc`, `compress ${name}: offered ${out.name}`);
  saved[name] = file(`out/${out.name}`, out.bytes);
  const lib = await libraryArchive('field-files');
  raise('differingArchives', sha(lib) === sha(out.bytes) ? 0 : 1);
  return r;
}

const cases = {
  async 'compress-file'() {
    const r = await compressAndCompare(inputs.text, 'text');
    measured.compressThroughput = Math.round(fs.statSync(inputs.text).size / r.ms);
  },
  async 'compress-matches-library'() {
    await compressAndCompare(inputs.frames, 'frames');
    expect(gates.differingArchives === 0, 'a page archive differs from the library archive');
  },
  async 'decompress-file'() {
    await need('compress-file');
    await tab('decompress');
    await setFiles('field-archive', [saved.text]);
    const r = await runAndWait();
    expect(r.state === 'done', `decompress: ${await text('status')}`);
    const [out] = await downloads();
    expect(out.name === 'text.txt' && sha(out.bytes) === sha(fs.readFileSync(inputs.text)), `decompress restored ${out.name} with other bytes`);
    measured.decompressThroughput = Math.round(fs.statSync(inputs.text).size / r.ms);
  },
  async 'decompress-reference-frames'() {
    await need('compress-matches-library');
    await tab('decompress');
    await setFiles('field-archive', [saved.frames]);
    const r = await runAndWait();
    expect(r.state === 'done', `decompress frames: ${await text('status')}`);
    const [out] = await downloads();
    expect(sha(out.bytes) === sha(fs.readFileSync(inputs.frames)), 'reference frames restored other bytes');
    const mib = Number((await evaluate("document.querySelector('#result .memory').textContent")).match(/([\d,]+) MiB/)[1].replace(/,/g, ''));
    raise('workerMemory', mib);
  },
  async 'compress-files'() {
    await tab('compress');
    await setFiles('field-files', [inputs.text, inputs.other]);
    const r = await runAndWait();
    expect(r.state === 'done', `compress files: ${await text('status')}`);
    const [out] = await downloads();
    expect(out.name === 'files.uorc', `several files offered ${out.name}`);
    saved.files = file('out/files.uorc', out.bytes);
  },
  async 'compress-folder'() {
    await tab('compress');
    await setFiles('field-folder', [inputs.folder]);
    expect((await evaluate("document.getElementById('field-files').files.length")) === 0, 'choosing a folder did not clear the files');
    const r = await runAndWait();
    expect(r.state === 'done', `compress folder: ${await text('status')}`);
    const [out] = await downloads();
    expect(out.name === 'folder.uorc', `folder offered ${out.name}`);
    saved.folder = file('out/folder.uorc', out.bytes);
  },
  async 'decompress-folder'() {
    await need('compress-folder');
    await tab('decompress');
    await setFiles('field-archive', [saved.folder]);
    const r = await runAndWait();
    expect(r.state === 'done', `decompress folder: ${await text('status')}`);
    const got = Object.fromEntries((await downloads()).map((d) => [d.label.split(' ')[1], sha(d.bytes)]));
    const want = { 'folder/a.txt': sha(fs.readFileSync(path.join(inputs.folder, 'a.txt'))), 'folder/sub/b.bin': sha(fs.readFileSync(path.join(inputs.folder, 'sub/b.bin'))) };
    expect(JSON.stringify(got) === JSON.stringify(want), `folder restored ${JSON.stringify(Object.keys(got))}`);
  },
  async 'inspect-archive'() {
    await need('compress-matches-library');
    await tab('inspect');
    await setFiles('field-archive', [saved.frames]);
    const r = await runAndWait();
    expect(r.state === 'done', `inspect: ${await text('status')}`);
    const rows = await evaluate("[...document.querySelectorAll('#result tbody tr')].map((tr) => [...tr.cells].map((c) => c.textContent))");
    const head = await evaluate("[...document.querySelectorAll('#result thead th')].map((th) => th.textContent)");
    const columns = model.Column.filter((c) => c.table === 'inspect').sort((a, b) => a.order - b.order).map((c) => c.label);
    expect(JSON.stringify(head) === JSON.stringify(columns), 'inspect columns differ from the model');
    expect(rows.length === 3 && rows[2][1] === 'reference' && rows.every((row) => row[5] === 'intact'), `inspect rows ${JSON.stringify(rows)}`);
  },
  async 'verify-archive'() {
    await need('compress-file');
    await tab('verify');
    await setFiles('field-archive', [saved.text]);
    const r = await runAndWait();
    expect(r.state === 'done' && (await text('status')).startsWith('Verified'), `verify: ${await text('status')}`);
  },
  async 'verify-original'() {
    await tab('verify');
    await setFiles('field-archive', [saved.text]);
    await setFiles('field-original', [inputs.text]);
    const r = await runAndWait();
    expect(r.state === 'done' && (await text('status')).includes('reproduces text.txt exactly'), `verify original: ${await text('status')}`);
  },
  async 'verify-original-differs'() {
    await tab('verify');
    await setFiles('field-archive', [saved.text]);
    await setFiles('field-original', [inputs.other]);
    const r = await runAndWait();
    expect(r.state === 'failed', `verify against another file: ${await text('status')}`);
  },
  async 'cancel-running'() {
    await tab('compress');
    await setFiles('field-files', [inputs.frames]);
    await click('#run');
    await waitFor("document.body.dataset.state === 'running'");
    await sleep(500);
    const t = Date.now();
    await click('#cancel');
    await waitFor("document.body.dataset.state === 'ready'", 30000);
    raise('cancelLatency', Date.now() - t);
    expect((await text('status')) === model.Labels[0].cancelled, `after cancel: ${await text('status')}`);
    expect(await evaluate("document.getElementById('result').hidden"), 'a cancelled operation left a result');
  },
  async 'examples-round-trip'() {
    for (const example of model.Example) {
      await click(`[data-example=${example.id}]`);
      await waitFor("document.body.dataset.state !== 'running'");
      expect((await state()) === 'done', `example ${example.id}: ${await text('status')}`);
    }
    const rows = await evaluate("[...document.querySelectorAll('#example-results tbody tr')].map((tr) => [...tr.cells].map((c) => c.textContent))");
    expect(rows.length === model.Example.length && rows.every((r) => r[3] === 'identical'), `example rows ${JSON.stringify(rows)}`);
  },
  async 'error-decode'() {
    await tab('decompress');
    await setFiles('field-archive', [inputs.text]);
    const r = await runAndWait();
    expect(r.state === 'failed', `a non-archive decompressed: ${await text('status')}`);
    await expectHint('decode');
    await tab('compress');
    expect((await state()) === 'ready' && (await evaluate("document.getElementById('error').hidden")), 'editing after a failure did not return to ready');
  },
  async 'error-mismatch'() {
    await need('compress-file');
    await tab('verify');
    await setFiles('field-archive', [saved.text]);
    await setFiles('field-original', [inputs.other]);
    expect((await runAndWait()).state === 'failed', 'a differing original verified');
    await expectHint('mismatch');
  },
  async 'error-stopped'() {
    await tab('compress');
    await setFiles('field-files', [inputs.other]);
    await evaluate("self.addEventListener('message', () => { throw new Error('stopped by the acceptance'); })", workerSession);
    expect((await runAndWait()).state === 'failed', 'a stopped worker did not fail the operation');
    await expectHint('stopped');
    // The page replaced its worker: the next operation runs.
    await sleep(200);
    expect((await runAndWait()).state === 'done', `after a stop: ${await text('status')}`);
  },
  async 'error-load'() {
    await open(`${origin}/no-wasm/`);
    expect((await state()) === 'unavailable', `without the module the page is ${await state()}`);
    await expectHint('load');
    expect(await evaluate("[...document.querySelectorAll('button, input')].every((e) => e.disabled)"), 'a control is enabled without the client');
  },
};
const done = new Set();
async function need(name) {
  if (done.has(name)) return;
  done.add(name);
  const t = Date.now();
  await cases[name]();
  console.log(`  ok ${name} (${Date.now() - t} ms)`);
}

const failures = [];
const featureOf = (ref) => model.Feature.find((f) => `Uorc.Page::${f.name}` === ref);
for (const journey of model.Journey) {
  console.log(`journey ${journey.name} (${journey.actor.split('::')[1]})`);
  try {
    raise('readyTime', await open());
    for (const step of journey.steps) for (const name of featureOf(step).acceptance) await need(name);
  } catch (error) {
    failures.push(`${journey.name}: ${error.message}`);
    console.log(`  FAILED ${error.message}`);
  }
}
await harvest();

// Every case exists and ran, every transition was taken, every control
// matched the model, every gated measure held.
const declared = model.Feature.flatMap((f) => f.acceptance);
for (const name of declared) if (!cases[name]) failures.push(`acceptance case ${name} is not implemented`);
for (const name of Object.keys(cases)) if (!declared.includes(name)) failures.push(`acceptance case ${name} is not in the model`);
for (const name of declared) if (!done.has(name)) failures.push(`acceptance case ${name} did not run`);
for (const t of model.UmlTransition) if (!transitions.has(`${t.source}->${t.target}`)) failures.push(`transition ${t.source} -${t.trigger}-> ${t.target} was not taken`);
for (const t of transitions) if (!model.UmlTransition.some((m) => `${m.source}->${m.target}` === t)) failures.push(`unmodeled transition ${t}`);
failures.push(...new Set(violations));
const unnamed = await evaluate("[...document.querySelectorAll('button, input, a, [role=tab]')].filter((e) => !(e.labels?.[0]?.textContent.trim() || e.getAttribute('aria-label') || e.textContent.trim())).length").catch(() => 1);
await open();
raise('unnamedControls', await evaluate("[...document.querySelectorAll('button, input, a, [role=tab]')].filter((e) => !(e.labels?.[0]?.textContent.trim() || e.getAttribute('aria-label') || e.textContent.trim())).length"));
raise('unnamedControls', unnamed);
await harvest();
gates.longestPageTask ??= 0;
const report = { measures: [] };
for (const m of model.QualityMeasure) {
  const value = m.disposition === 'gate' ? gates[m.name] : measured[m.name];
  if (value === undefined) failures.push(`measure ${m.name} was not taken`);
  else if (m.disposition === 'gate' && value > m.threshold) failures.push(`measure ${m.name}: ${value} ${m.unit} exceeds ${m.threshold}`);
  report.measures.push({ measure: m.name, quantity: m.quantity, value, unit: m.unit, threshold: m.disposition === 'gate' ? m.threshold : null });
  console.log(`measure ${m.name}: ${value} ${m.unit}${m.disposition === 'gate' ? ` (at most ${m.threshold})` : ' (measured)'}`);
}
report.longTasksObserved = longTasksSeen;
report.transitions = [...transitions].sort();
report.cases = [...done];
report.failures = failures;
if (opt('--report')) fs.writeFileSync(opt('--report'), `${JSON.stringify(report, null, 2)}\n`);

ws.close();
browser.kill();
await new Promise((r) => (browser.exitCode !== null ? r() : browser.on('exit', r)));
server.close();
fs.rmSync(dir, { recursive: true, force: true });
fs.rmSync(profile, { recursive: true, force: true });
if (failures.length) {
  console.error(`page acceptance FAILED:\n  ${failures.join('\n  ')}`);
  process.exit(1);
}
console.log(`page acceptance: ${done.size} cases, ${transitions.size} transitions, ${model.QualityMeasure.length} measures`);
