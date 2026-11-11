// Measure the uorc release binary against standard compressors on public corpora.
// Measurement only: this records sizes, times and round-trip results; it sets
// no target and makes no claim beyond the files, versions and budgets it lists.
//
//   node scripts/bench.mjs --uorc target/release/uorc --corpus <dir>[,<dir>...] --steps N --out <dir>
//
// Writes bench.json (every measurement), bench.md (release notes) and
// bench.html (the Pages benchmarks page). Each report also carries the
// machine's ceiling: its core count and the rate at which it copies memory, on
// one core and on all of them, so each throughput reads against what the host
// can move at all.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {Worker} from 'node:worker_threads';

const args = Object.fromEntries(process.argv.slice(2).reduce((acc, a, i, all) => (a.startsWith('--') ? [...acc, [a.slice(2), all[i + 1]]] : acc), []));
const fail = message => { console.error(message); process.exit(2); };
const uorc = path.resolve(args.uorc ?? 'target/release/uorc');
const corpora = (args.corpus ?? fail('--corpus is required')).split(',').map(d => path.resolve(d));
const steps = String(args.steps ?? 0);
const out = path.resolve(args.out ?? 'bench-results');
// A per-run wall-clock limit, so one pathological file cannot eat the job.
const limitMs = Number(args['timeout-s'] ?? 1800) * 1000;

// Stream tools read stdin and write stdout. Archivers (zpaq, 7-Zip) work on
// files: their archive bytes include their own container headers.
const baselines = [
  {name: 'gzip -9', tool: 'gzip', compress: ['-9', '-c'], decompress: ['-d', '-c'], version: ['--version']},
  {name: 'bzip2 -9', tool: 'bzip2', compress: ['-9', '-c'], decompress: ['-d', '-c'], version: ['--help']},
  {name: 'xz -9e', tool: 'xz', compress: ['-9e', '-c'], decompress: ['-d', '-c'], version: ['--version']},
  {name: 'zstd -19 --long', tool: 'zstd', compress: ['-19', '--long=27', '-q', '-c'], decompress: ['-d', '--long=27', '-q', '-c'], version: ['--version']},
  {name: 'brotli -q 11', tool: 'brotli', compress: ['-q', '11', '-c'], decompress: ['-d', '-c'], version: ['--version']},
  // Context-mixing and PPM coders, the class uorc's ncm coder belongs to.
  {name: 'zpaq -m5', tool: 'zpaq', files: true, version: [],
    compress: (src, arc) => ['a', arc, src, '-m5'], decompress: (arc, dir) => ['x', arc, '-to', dir, '-f']},
  {name: '7z PPMd -mx9', tool: '7z', files: true, version: ['i'],
    compress: (src, arc) => ['a', '-t7z', '-m0=PPMd', '-mx=9', '-bd', '-bso0', arc, src], decompress: (arc, dir) => ['e', arc, `-o${dir}`, '-y', '-bd', '-bso0']},
];
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const timed = (cmd, argv, input) => {
  const started = process.hrtime.bigint();
  const r = spawnSync(cmd, argv, {input, cwd: work, maxBuffer: 2 ** 31, timeout: limitMs});
  return {r, seconds: Number(process.hrtime.bigint() - started) / 1e9};
};
const failure = r => (r.error ? String(r.error.code ?? r.error) : `exit ${r.status}: ${String(r.stderr ?? '').trim().slice(0, 200)}`);
const firstLine = (cmd, argv) => { const r = spawnSync(cmd, argv, {encoding: 'utf8'}); return r.error ? null : `${r.stdout}${r.stderr}`.trim().split('\n').find(l => l.trim()) ?? ''; };

// Memory copy rate: each of n threads copies its own 64 MiB buffer for about
// a second; the rate is bytes written per second, summed over the threads.
const copyWorker = `const {parentPort, workerData} = require('node:worker_threads');
const size = 64 << 20, a = Buffer.alloc(size, 1), b = Buffer.alloc(size);
for (let i = 0; i < 4; i++) a.copy(b);
const t0 = process.hrtime.bigint(); let copied = 0, ns = 0n;
do { a.copy(b); copied += size; ns = process.hrtime.bigint() - t0; } while (ns < BigInt(workerData.ms) * 1000000n);
parentPort.postMessage(copied / (Number(ns) / 1e9));`;
const copyRate = threads => Promise.all(Array.from({length: threads}, () => new Promise((resolve, reject) => {
  const w = new Worker(copyWorker, {eval: true, workerData: {ms: 1000}});
  w.once('message', resolve); w.once('error', reject);
}))).then(rates => rates.reduce((s, r) => s + r, 0));
const best = async (threads, runs = 3) => { let top = 0; for (let i = 0; i < runs; i++) top = Math.max(top, await copyRate(threads)); return top; };
const cores = os.availableParallelism();
const machine = {cores, cpu: os.cpus()[0]?.model ?? 'unknown', memory_bytes: os.totalmem(),
  copy_mb_s_one_core: await best(1) / 1e6, copy_mb_s_all_cores: await best(cores) / 1e6};

const tools = baselines.map(b => ({...b, available: firstLine(b.tool, b.version)})).filter(b => b.available !== null);
// uorc has one coder (ncm); --best is accepted and changes nothing.
const uorcName = `uorc (${steps} steps)`;
const names = [uorcName, ...tools.map(t => t.name)];
fs.mkdirSync(out, {recursive: true});
const work = fs.mkdtempSync(path.join(out, 'work-'));

function measure(file, input) {
  const row = {file, bytes: input.length, sha256: sha(input), results: []};
  const src = path.join(work, 'in'), arc = path.join(work, 'a.uorc'), back = path.join(work, 'back');
  fs.writeFileSync(src, input);
  const c = timed(uorc, ['compress', '--steps', steps, src, '-o', arc]);
  if (c.r.status === 0) {
    const archive = fs.statSync(arc).size;
    const d = timed(uorc, ['decompress', arc, '-o', back]);
    const ok = d.r.status === 0 && Buffer.compare(fs.readFileSync(back), input) === 0;
    row.results.push({compressor: uorcName, bytes: archive, compress_s: c.seconds, decompress_s: d.seconds, round_trip: ok, ...(d.r.status === 0 ? {} : {error: failure(d.r)})});
  } else row.results.push({compressor: uorcName, error: failure(c.r)});
  for (const f of [arc, back]) fs.rmSync(f, {force: true});
  for (const t of tools.filter(t => t.files)) {
    const dir = path.join(work, 'x'), archive = path.join(work, 'a.bin');
    const cz = timed(t.tool, t.compress('in', archive));
    if (cz.r.status !== 0) { row.results.push({compressor: t.name, error: failure(cz.r)}); fs.rmSync(archive, {force: true}); continue; }
    const dz = timed(t.tool, t.decompress(archive, dir));
    const restored = path.join(dir, 'in');
    const ok = dz.r.status === 0 && fs.existsSync(restored) && Buffer.compare(fs.readFileSync(restored), input) === 0;
    row.results.push({compressor: t.name, bytes: fs.statSync(archive).size, compress_s: cz.seconds, decompress_s: dz.seconds, round_trip: ok, ...(dz.r.status === 0 ? {} : {error: failure(dz.r)})});
    for (const f of [archive, dir]) fs.rmSync(f, {recursive: true, force: true});
  }
  fs.rmSync(src, {force: true});
  for (const t of tools.filter(t => !t.files)) {
    const cz = timed(t.tool, t.compress, input);
    if (cz.r.status !== 0) { row.results.push({compressor: t.name, error: failure(cz.r)}); continue; }
    const dz = timed(t.tool, t.decompress, cz.r.stdout);
    row.results.push({compressor: t.name, bytes: cz.r.stdout.length, compress_s: cz.seconds, decompress_s: dz.seconds, round_trip: dz.r.status === 0 && Buffer.compare(dz.r.stdout, input) === 0});
  }
  row.results.sort((a, b) => names.indexOf(a.compressor) - names.indexOf(b.compressor));
  console.log(file, row.results.map(r => `${r.compressor}=${r.bytes ?? 'error'}`).join(' '));
  return row;
}

const suites = corpora.map(dir => {
  const files = fs.readdirSync(dir).filter(f => fs.statSync(path.join(dir, f)).isFile()).sort();
  if (!files.length) fail(`no files in ${dir}`);
  return {corpus: path.basename(dir), files: files.map(f => measure(f, fs.readFileSync(path.join(dir, f))))};
});
fs.rmSync(work, {recursive: true, force: true});

// Per corpus and compressor: total archive bytes and throughput over the files
// every compressor round-tripped, so a failure never flatters a total.
const summarize = suite => names.map(n => {
  const done = suite.files.map(r => r.results.find(x => x.compressor === n)).filter(x => x?.round_trip);
  const input = suite.files.filter(r => r.results.find(x => x.compressor === n)?.round_trip).reduce((s, r) => s + r.bytes, 0);
  const sum = k => done.reduce((s, x) => s + x[k], 0);
  return {compressor: n, files: done.length, input_bytes: input, archive_bytes: sum('bytes'),
    ratio: input ? input / sum('bytes') : null, compress_mb_s: input / 1e6 / sum('compress_s') || null, decompress_mb_s: input / 1e6 / sum('decompress_s') || null};
}).map(x => ({...x, decompress_share_of_copy: x.decompress_mb_s && x.decompress_mb_s / machine.copy_mb_s_all_cores}));
for (const s of suites) s.summary = summarize(s);

const report = {
  spec: 'uorc/benchmark/3', kind: 'measurement', target: 'none',
  uorc: firstLine(uorc, ['--version']), steps: Number(steps), timeout_s: limitMs / 1000,
  host: {platform: process.platform, arch: process.arch, ...machine},
  tools: tools.map(t => ({name: t.name, version: t.available})),
  corpora: suites,
};
fs.writeFileSync(path.join(out, 'bench.json'), JSON.stringify(report, null, 1) + '\n');

const fixed = (x, d) => (x == null ? 'n/a' : x.toFixed(d));
const cell = r => (r.bytes === undefined ? 'error' : `${r.bytes}${r.round_trip ? '' : ' ✗'}`);
const esc = s => String(s).replace(/[&<>]/g, c => ({'&': '&amp;', '<': '&lt;', '>': '&gt;'}[c]));
const intro = `${report.uorc}, ${report.steps} search steps per file, ${report.host.platform}/${report.host.arch}. Measurement only; no target is set.`;
const gib = b => (b / 2 ** 30).toFixed(1);
const ceiling = `Machine ceiling: ${machine.cores} cores (${machine.cpu}), ${gib(machine.memory_bytes)} GiB memory; memory copy ${fixed(machine.copy_mb_s_one_core, 0)} MB/s on one core and ${fixed(machine.copy_mb_s_all_cores, 0)} MB/s on all ${machine.cores}. A decoder cannot write its output faster than the machine copies memory, so the last column is decompress throughput as a share of the all-core copy rate. uorc uses every core (8 MiB frames in parallel); the other tools run with their default thread settings.`;
const pct = x => (x == null ? 'n/a' : `${(100 * x).toPrecision(2)}%`);
const summaryHead = ['compressor', 'files', 'archive bytes', 'ratio', 'compress MB/s', 'decompress MB/s', 'decompress / copy'];
const summaryRow = s => [s.compressor, s.files, s.archive_bytes, fixed(s.ratio, 3), fixed(s.compress_mb_s, 2), fixed(s.decompress_mb_s, 2), pct(s.decompress_share_of_copy)];

const md = [`# uorc benchmarks`, '', intro, '', ceiling, ''];
for (const s of suites) {
  md.push(`## ${s.corpus}`, '', `| ${summaryHead.join(' | ')} |`, `| --- | ${summaryHead.slice(1).map(() => '---:').join(' | ')} |`,
    ...s.summary.map(x => `| ${summaryRow(x).join(' | ')} |`), '',
    '<details><summary>Per file (archive bytes)</summary>', '',
    `| file | bytes | ${names.join(' | ')} |`, `| --- | ---: | ${names.map(() => '---:').join(' | ')} |`,
    ...s.files.map(r => `| ${r.file} | ${r.bytes} | ${r.results.map(cell).join(' | ')} |`), '', '</details>', '');
}
md.push('Ratio is input bytes over archive bytes, and throughput is input MB per second, both over the files that compressor round-tripped. ✗ marks a failed round trip. zpaq and 7z archive bytes include their container headers. Tool versions:', '', ...report.tools.map(t => `- ${t.name}: ${t.version}`), '');
fs.writeFileSync(path.join(out, 'bench.md'), md.join('\n'));

const htmlTable = (head, rows) => `<div class="table-wrap"><table><thead><tr>${head.map(h => `<th scope="col">${esc(h)}</th>`).join('')}</tr></thead><tbody>${
  rows.map(r => `<tr>${r.map(c => `<td>${esc(c)}</td>`).join('')}</tr>`).join('')}</tbody></table></div>`;
const sections = suites.map(s => `<h2>${esc(s.corpus)}</h2>${htmlTable(summaryHead, s.summary.map(summaryRow))}<details><summary>Per file (archive bytes)</summary>${
  htmlTable(['file', 'bytes', ...names], s.files.map(r => [r.file, r.bytes, ...r.results.map(cell)]))}</details>`).join('');
const html = `<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>uorc benchmarks</title><link rel="stylesheet" href="app.css"><link rel="icon" href="icon.svg" type="image/svg+xml"></head><body><header><h1>Benchmarks</h1><p>${esc(intro)}</p><p>${esc(ceiling)}</p><p>Ratio is input over archive bytes and throughput is input MB per second, over the files each compressor round-tripped; ✗ marks a failed round trip; zpaq and 7z archive bytes include their container headers. Raw data: <a href="bench.json">bench.json</a>. <a href="./">Back to the client</a>.</p></header><main>${sections}<h2>Tools</h2><ul>${report.tools.map(t => `<li>${esc(t.name)}: ${esc(t.version)}</li>`).join('')}</ul></main></body></html>\n`;
fs.writeFileSync(path.join(out, 'bench.html'), html);
console.log(`wrote ${out}/bench.{json,md,html}`);
