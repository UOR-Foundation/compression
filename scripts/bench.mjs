// Measure the uorc release binary against standard compressors on a corpus.
// Measurement only: this records sizes, times and round-trip results; it sets
// no target and makes no claim beyond the files, versions and budgets it lists.
//
//   node scripts/bench.mjs --uorc target/release/uorc --corpus <dir> --steps N --out <dir>
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {spawnSync} from 'node:child_process';

const args = Object.fromEntries(process.argv.slice(2).reduce((acc, a, i, all) => (a.startsWith('--') ? [...acc, [a.slice(2), all[i + 1]]] : acc), []));
const uorc = path.resolve(args.uorc ?? 'target/release/uorc');
const corpus = path.resolve(args.corpus ?? fail('--corpus is required'));
const steps = String(args.steps ?? 10_000_000);
const out = path.resolve(args.out ?? 'bench-results');
function fail(message) { console.error(message); process.exit(2); }

const baselines = [
  {name: 'gzip -9', tool: 'gzip', compress: ['-9', '-c'], decompress: ['-d', '-c'], version: ['--version']},
  {name: 'bzip2 -9', tool: 'bzip2', compress: ['-9', '-c'], decompress: ['-d', '-c'], version: ['--help']},
  {name: 'xz -9e', tool: 'xz', compress: ['-9e', '-c'], decompress: ['-d', '-c'], version: ['--version']},
  {name: 'zstd -19 --long', tool: 'zstd', compress: ['-19', '--long=27', '-q', '-c'], decompress: ['-d', '--long=27', '-q', '-c'], version: ['--version']},
  {name: 'brotli -q 11', tool: 'brotli', compress: ['-q', '11', '-c'], decompress: ['-d', '-c'], version: ['--version']},
];
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const timed = (cmd, argv, input) => {
  const started = process.hrtime.bigint();
  const r = spawnSync(cmd, argv, {input, maxBuffer: 1 << 30});
  return {r, seconds: Number(process.hrtime.bigint() - started) / 1e9};
};
const firstLine = (cmd, argv) => { const r = spawnSync(cmd, argv, {encoding: 'utf8'}); return r.error ? null : `${r.stdout}${r.stderr}`.trim().split('\n')[0]; };

const files = fs.readdirSync(corpus).filter(f => fs.statSync(path.join(corpus, f)).isFile()).sort();
if (!files.length) fail(`no files in ${corpus}`);
const tools = baselines.map(b => ({...b, available: firstLine(b.tool, b.version)})).filter(b => b.available);
const work = fs.mkdtempSync(path.join(out + '-work'));
const rows = [];
for (const file of files) {
  const input = fs.readFileSync(path.join(corpus, file));
  const row = {file, bytes: input.length, sha256: sha(input), results: []};
  const src = path.join(work, 'in'); fs.writeFileSync(src, input);
  const c = timed(uorc, ['compress', '--steps', steps, src, '-o', path.join(work, 'a.uorc')]);
  if (c.r.status === 0) {
    const archive = fs.readFileSync(path.join(work, 'a.uorc'));
    const d = timed(uorc, ['decompress', path.join(work, 'a.uorc'), '-o', path.join(work, 'back')]);
    const ok = d.r.status === 0 && Buffer.compare(fs.readFileSync(path.join(work, 'back')), input) === 0;
    row.results.push({compressor: `uorc (${steps} steps)`, bytes: archive.length, compress_s: c.seconds, decompress_s: d.seconds, round_trip: ok});
  } else row.results.push({compressor: `uorc (${steps} steps)`, error: c.r.stderr.toString().trim()});
  for (const t of tools) {
    const cz = timed(t.tool, t.compress, input);
    const dz = timed(t.tool, t.decompress, cz.r.stdout);
    row.results.push({compressor: t.name, bytes: cz.r.stdout.length, compress_s: cz.seconds, decompress_s: dz.seconds, round_trip: dz.r.status === 0 && Buffer.compare(dz.r.stdout, input) === 0});
  }
  rows.push(row);
  console.log(file, row.results.map(r => `${r.compressor}=${r.bytes ?? 'error'}`).join(' '));
}
fs.rmSync(work, {recursive: true, force: true});

const report = {
  spec: 'uorc/benchmark/1', kind: 'measurement', target: 'none',
  uorc: firstLine(uorc, ['--version']), steps: Number(steps),
  host: {platform: process.platform, arch: process.arch},
  tools: tools.map(t => ({name: t.name, version: t.available})),
  corpus: path.basename(corpus), files: rows,
};
fs.mkdirSync(out, {recursive: true});
fs.writeFileSync(path.join(out, 'bench.json'), JSON.stringify(report, null, 1) + '\n');

const names = rows[0].results.map(r => r.compressor);
const total = n => rows.reduce((s, r) => s + (r.results.find(x => x.compressor === n)?.bytes ?? 0), 0);
const input = rows.reduce((s, r) => s + r.bytes, 0);
const esc = s => String(s).replace(/[&<>]/g, c => ({'&': '&amp;', '<': '&lt;', '>': '&gt;'}[c]));
const cell = r => r.bytes === undefined ? 'error' : `${r.bytes}${r.round_trip ? '' : ' ✗'}`;
const md = [`# uorc benchmark: ${report.corpus}`, '', `${report.uorc}, ${report.steps} search steps per file, ${report.host.platform}/${report.host.arch}. Measurement only; no target is set.`, '',
  `| file | bytes | ${names.join(' | ')} |`, `| --- | ---: | ${names.map(() => '---:').join(' | ')} |`,
  ...rows.map(r => `| ${r.file} | ${r.bytes} | ${r.results.map(cell).join(' | ')} |`),
  `| **total** | **${input}** | ${names.map(n => `**${total(n)}**`).join(' | ')} |`, '',
  'Archive sizes in bytes; ✗ marks a failed round trip. Tool versions:', '', ...report.tools.map(t => `- ${t.name}: ${t.version}`), ''].join('\n');
fs.writeFileSync(path.join(out, 'bench.md'), md);
const table = `<table><thead><tr><th scope="col">File</th><th scope="col">Bytes</th>${names.map(n => `<th scope="col">${esc(n)}</th>`).join('')}</tr></thead><tbody>${
  rows.map(r => `<tr><td>${esc(r.file)}</td><td>${r.bytes}</td>${r.results.map(x => `<td>${esc(cell(x))}</td>`).join('')}</tr>`).join('')
}<tr><th scope="row">Total</th><td>${input}</td>${names.map(n => `<td>${total(n)}</td>`).join('')}</tr></tbody></table>`;
const html = `<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>uorc benchmarks</title><link rel="stylesheet" href="style.css"></head><body><header><h1>Benchmarks</h1><p>${esc(report.uorc)}, ${report.steps} search steps per file on ${esc(report.host.platform)}/${esc(report.host.arch)}. Measurement only: no target is set. Archive sizes are in bytes; ✗ marks a failed round trip. Raw data: <a href="bench.json">bench.json</a>. <a href="./">Back to the client</a>.</p></header><main><h2>${esc(report.corpus)}</h2><div class="table-wrap">${table}</div><h2>Tools</h2><ul>${report.tools.map(t => `<li>${esc(t.name)}: ${esc(t.version)}</li>`).join('')}</ul></main></body></html>\n`;
fs.writeFileSync(path.join(out, 'bench.html'), html);
console.log(`wrote ${out}/bench.{json,md,html}`);
