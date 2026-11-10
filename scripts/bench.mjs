// Measure the uorc release binary against standard compressors on public corpora.
// Measurement only: this records sizes, times and round-trip results; it sets
// no target and makes no claim beyond the files, versions and budgets it lists.
//
//   node scripts/bench.mjs --uorc target/release/uorc --corpus <dir>[,<dir>...] --steps N --out <dir>
//
// Writes bench.json (every measurement), bench.md (release notes) and
// bench.html (the Pages benchmarks page).
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {spawnSync} from 'node:child_process';

const args = Object.fromEntries(process.argv.slice(2).reduce((acc, a, i, all) => (a.startsWith('--') ? [...acc, [a.slice(2), all[i + 1]]] : acc), []));
const fail = message => { console.error(message); process.exit(2); };
const uorc = path.resolve(args.uorc ?? 'target/release/uorc');
const corpora = (args.corpus ?? fail('--corpus is required')).split(',').map(d => path.resolve(d));
const steps = String(args.steps ?? 0);
const out = path.resolve(args.out ?? 'bench-results');
// A per-run wall-clock limit, so one pathological file cannot eat the job.
const limitMs = Number(args['timeout-s'] ?? 1800) * 1000;

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
  const r = spawnSync(cmd, argv, {input, maxBuffer: 2 ** 31, timeout: limitMs});
  return {r, seconds: Number(process.hrtime.bigint() - started) / 1e9};
};
const failure = r => (r.error ? String(r.error.code ?? r.error) : `exit ${r.status}: ${String(r.stderr ?? '').trim().slice(0, 200)}`);
const firstLine = (cmd, argv) => { const r = spawnSync(cmd, argv, {encoding: 'utf8'}); return r.error ? null : `${r.stdout}${r.stderr}`.trim().split('\n')[0]; };

const tools = baselines.map(b => ({...b, available: firstLine(b.tool, b.version)})).filter(b => b.available);
// uorc has one coder (ncm); --best is accepted and changes nothing.
const levels = [[`uorc (${steps} steps)`, []]];
const names = [...levels.map(([label]) => label), ...tools.map(t => t.name)];
fs.mkdirSync(out, {recursive: true});
const work = fs.mkdtempSync(path.join(out, 'work-'));

function measure(file, input) {
  const row = {file, bytes: input.length, sha256: sha(input), results: []};
  const src = path.join(work, 'in'), arc = path.join(work, 'a.uorc'), back = path.join(work, 'back');
  fs.writeFileSync(src, input);
  for (const [label, extra] of levels) {
    const c = timed(uorc, ['compress', ...extra, '--steps', steps, src, '-o', arc]);
    if (c.r.status === 0) {
      const archive = fs.statSync(arc).size;
      const d = timed(uorc, ['decompress', arc, '-o', back]);
      const ok = d.r.status === 0 && Buffer.compare(fs.readFileSync(back), input) === 0;
      row.results.push({compressor: label, bytes: archive, compress_s: c.seconds, decompress_s: d.seconds, round_trip: ok, ...(d.r.status === 0 ? {} : {error: failure(d.r)})});
    } else row.results.push({compressor: label, error: failure(c.r)});
    for (const f of [arc, back]) fs.rmSync(f, {force: true});
  }
  fs.rmSync(src, {force: true});
  for (const t of tools) {
    const cz = timed(t.tool, t.compress, input);
    if (cz.r.status !== 0) { row.results.push({compressor: t.name, error: failure(cz.r)}); continue; }
    const dz = timed(t.tool, t.decompress, cz.r.stdout);
    row.results.push({compressor: t.name, bytes: cz.r.stdout.length, compress_s: cz.seconds, decompress_s: dz.seconds, round_trip: dz.r.status === 0 && Buffer.compare(dz.r.stdout, input) === 0});
  }
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
});
for (const s of suites) s.summary = summarize(s);

const report = {
  spec: 'uorc/benchmark/2', kind: 'measurement', target: 'none',
  uorc: firstLine(uorc, ['--version']), steps: Number(steps), timeout_s: limitMs / 1000,
  host: {platform: process.platform, arch: process.arch},
  tools: tools.map(t => ({name: t.name, version: t.available})),
  corpora: suites,
};
fs.writeFileSync(path.join(out, 'bench.json'), JSON.stringify(report, null, 1) + '\n');

const fixed = (x, d) => (x == null ? 'n/a' : x.toFixed(d));
const cell = r => (r.bytes === undefined ? 'error' : `${r.bytes}${r.round_trip ? '' : ' ✗'}`);
const esc = s => String(s).replace(/[&<>]/g, c => ({'&': '&amp;', '<': '&lt;', '>': '&gt;'}[c]));
const intro = `${report.uorc}, ${report.steps} search steps per file, ${report.host.platform}/${report.host.arch}. Measurement only; no target is set.`;
const summaryHead = ['compressor', 'files', 'archive bytes', 'ratio', 'compress MB/s', 'decompress MB/s'];
const summaryRow = s => [s.compressor, s.files, s.archive_bytes, fixed(s.ratio, 3), fixed(s.compress_mb_s, 2), fixed(s.decompress_mb_s, 2)];

const md = [`# uorc benchmarks`, '', intro, ''];
for (const s of suites) {
  md.push(`## ${s.corpus}`, '', `| ${summaryHead.join(' | ')} |`, `| --- | ${summaryHead.slice(1).map(() => '---:').join(' | ')} |`,
    ...s.summary.map(x => `| ${summaryRow(x).join(' | ')} |`), '',
    '<details><summary>Per file (archive bytes)</summary>', '',
    `| file | bytes | ${names.join(' | ')} |`, `| --- | ---: | ${names.map(() => '---:').join(' | ')} |`,
    ...s.files.map(r => `| ${r.file} | ${r.bytes} | ${r.results.map(cell).join(' | ')} |`), '', '</details>', '');
}
md.push('Ratio is input bytes over archive bytes, and throughput is input MB per second, both over the files that compressor round-tripped. ✗ marks a failed round trip. Tool versions:', '', ...report.tools.map(t => `- ${t.name}: ${t.version}`), '');
fs.writeFileSync(path.join(out, 'bench.md'), md.join('\n'));

const htmlTable = (head, rows) => `<div class="table-wrap"><table><thead><tr>${head.map(h => `<th scope="col">${esc(h)}</th>`).join('')}</tr></thead><tbody>${
  rows.map(r => `<tr>${r.map(c => `<td>${esc(c)}</td>`).join('')}</tr>`).join('')}</tbody></table></div>`;
const sections = suites.map(s => `<h2>${esc(s.corpus)}</h2>${htmlTable(summaryHead, s.summary.map(summaryRow))}<details><summary>Per file (archive bytes)</summary>${
  htmlTable(['file', 'bytes', ...names], s.files.map(r => [r.file, r.bytes, ...r.results.map(cell)]))}</details>`).join('');
const html = `<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>uorc benchmarks</title><link rel="stylesheet" href="style.css"></head><body><header><h1>Benchmarks</h1><p>${esc(intro)} Ratio is input over archive bytes and throughput is input MB per second, over the files each compressor round-tripped; ✗ marks a failed round trip. Raw data: <a href="bench.json">bench.json</a>. <a href="./">Back to the client</a>.</p></header><main>${sections}<h2>Tools</h2><ul>${report.tools.map(t => `<li>${esc(t.name)}: ${esc(t.version)}</li>`).join('')}</ul></main></body></html>\n`;
fs.writeFileSync(path.join(out, 'bench.html'), html);
console.log(`wrote ${out}/bench.{json,md,html}`);
