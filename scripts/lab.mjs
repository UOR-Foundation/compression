// Developer and agent measurement commands for codec work: one command per
// measurement every lane otherwise rewrites by hand. Measurement only; no CI
// job runs this, and nothing here sets a target.
//
//   node scripts/lab.mjs corpus [--size 8M] [--corpus py,elf,text,canterbury,silesia]
//   node scripts/lab.mjs hosts  [--uorc BIN] [--threads 1,4] [--steps 0] [--size 8M] [--corpus py,elf] [--json F] [FILE...]
//   node scripts/lab.mjs ab     --ref <rev|path-to-uorc> [--uorc BIN] [--runs 3] [--threads 1,4] [--identical] [--size 8M] [--corpus py,elf] [--json F] [FILE...]
//   node scripts/lab.mjs cost   [--uorc BIN] [--bytes 512K] [--input py|FILE] [--op decompress|compress|both] [--top 25] [--json F]
//
// hosts: compress and decompress each input at each thread count and record
//   wall time, user+sys CPU, busy cores of the process and of the host (sampled
//   from /proc every --interval-ms, default 50: mean, peak and a ten-bucket
//   timeline) and peak RSS (wait4), and check the round trip.
// ab: the same measurements for the working tree's uorc (A) against a
//   reference (B: a git rev built once into target/lab/ref/bin/<sha>, or a
//   path to a uorc binary), runs interleaved A,B,B,A,... so drift on a noisy
//   host falls on both; reports medians and spread, round trips, and whether
//   every archive is byte-identical. --identical exits 1 when one differs.
// cost: cachegrind over one --threads 1 decode (and/or encode) of a
//   --bytes prefix of an input, from a release build carrying line tables
//   (CARGO_PROFILE_RELEASE_DEBUG=line-tables-only, target/lab/cost-target);
//   instructions per source line, function and file, top N.
//
// Inputs: --corpus picks generated, deterministic inputs (py: a tar of this
// host's Python stdlib .py files; elf: a concatenation of the ELF files in
// /usr/bin; text: the stdlib .py files concatenated, four times --size) and
// the Canterbury/Silesia corpora (fetched from the URLs release.yml uses),
// all cached under target/lab/corpus; FILE arguments add files or directories.
// Every input's sha256 is recorded, so two hosts' numbers say what they read.
// Without --uorc, A is `cargo build --locked --release -p uorc` of the working
// tree (CARGO_TARGET_DIR honoured). Every command prints Markdown tables and
// writes JSON (default target/lab/out/<command>.json).
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const lab = path.join(root, 'target', 'lab');
const fail = message => { console.error(`lab: ${message}`); process.exit(2); };

const flags = new Set(['identical', 'help']);
const [command, ...rest] = process.argv.slice(2);
const args = {_: []};
for (let i = 0; i < rest.length; i++) {
  const a = rest[i];
  if (!a.startsWith('--')) args._.push(a);
  else if (flags.has(a.slice(2))) args[a.slice(2)] = true;
  else if (i + 1 < rest.length) args[a.slice(2)] = rest[++i];
  else fail(`${a} needs a value`);
}
if (!command || command === 'help' || args.help) {
  process.stdout.write(fs.readFileSync(fileURLToPath(import.meta.url), 'utf8').split('\n').filter(l => l.startsWith('//')).map(l => l.slice(3)).join('\n') + '\n');
  process.exit(command ? 0 : 2);
}

const bytesOf = s => {
  const m = /^(\d+(?:\.\d+)?)\s*([kmg]?)i?b?$/i.exec(String(s).trim()) ?? fail(`not a size: ${s}`);
  return Math.round(Number(m[1]) * {'': 1, k: 1024, m: 1024 ** 2, g: 1024 ** 3}[m[2].toLowerCase()]);
};
const label = n => (n % 1024 ** 2 === 0 ? `${n / 1024 ** 2}M` : n % 1024 === 0 ? `${n / 1024}K` : `${n}`);
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const run = (cmd, argv, opts = {}) => {
  const r = spawnSync(cmd, argv, {encoding: 'utf8', maxBuffer: 2 ** 30, ...opts});
  if (r.error || r.status !== 0) fail(`${cmd} ${argv.join(' ')} failed: ${r.error ?? `exit ${r.status}`}\n${String(r.stderr ?? '').slice(-2000)}`);
  return r.stdout;
};
const median = xs => { const s = [...xs].sort((a, b) => a - b); const m = s.length >> 1; return s.length % 2 ? s[m] : (s[m - 1] + s[m]) / 2; };
const stats = xs => (xs.length ? {median: median(xs), min: Math.min(...xs), max: Math.max(...xs), spread: (Math.max(...xs) - Math.min(...xs)) / (median(xs) || 1), n: xs.length} : null);
// Markdown table; a column whose every cell reads as numbers aligns right.
const numeric = c => /^(n\/a|[-+]?[\d.,]+%?( \(±[\d.]+%\))?( \/ [-+]?[\d.,n/a]+)?)$/.test(String(c));
const table = (head, rows) => [`| ${head.join(' | ')} |`, `| ${head.map((_, i) => (rows.length && rows.every(r => numeric(r[i])) ? '---:' : '---')).join(' | ')} |`, ...rows.map(r => `| ${r.join(' | ')} |`)].join('\n');
const f = (x, d = 2) => (x == null || Number.isNaN(x) ? 'n/a' : x.toFixed(d));
const mib = kib => kib / 1024;
const writeJson = (name, report) => {
  const out = path.resolve(args.json ?? path.join(lab, 'out', `${name}.json`));
  fs.mkdirSync(path.dirname(out), {recursive: true});
  fs.writeFileSync(out, JSON.stringify(report, null, 1) + '\n');
  console.log(`\nwrote ${out.startsWith(process.cwd() + '/') ? path.relative(process.cwd(), out) : out}`);
};
const host = () => ({cores: os.availableParallelism(), cpu: os.cpus()[0]?.model ?? 'unknown', memory_bytes: os.totalmem(), kernel: os.release(), load_avg: os.loadavg()});

// ---------------------------------------------------------------- binaries

const cargoTarget = () => path.resolve(process.env.CARGO_TARGET_DIR ?? path.join(root, 'target'));
function binary(spec, env = {}, targetDir = cargoTarget()) {
  if (spec) return fs.existsSync(spec) ? path.resolve(spec) : fail(`no binary at ${spec}`);
  console.error(`lab: building the working tree's uorc into ${path.relative(root, targetDir) || targetDir}`);
  run('cargo', ['build', '--locked', '--release', '-p', 'uorc'], {cwd: root, stdio: ['ignore', 'inherit', 'inherit'], env: {...process.env, ...env, CARGO_TARGET_DIR: targetDir}});
  return path.join(targetDir, 'release', 'uorc');
}
// A reference: a path to a uorc binary, or a git rev built once in a worktree
// of its own (target/lab/ref/src, reused across revs so cargo rebuilds only
// what changed) and cached by commit.
function reference(spec) {
  if (fs.existsSync(spec) && fs.statSync(spec).isFile()) return {spec, bin: path.resolve(spec)};
  const r = spawnSync('git', ['rev-parse', '--verify', `${spec}^{commit}`], {cwd: root, encoding: 'utf8'});
  if (r.status !== 0) fail(`--ref ${spec}: neither a file nor a git rev`);
  const commit = r.stdout.trim();
  const bin = path.join(lab, 'ref', 'bin', commit, 'uorc');
  if (fs.existsSync(bin)) return {spec, commit, bin};
  const src = path.join(lab, 'ref', 'src');
  if (fs.existsSync(path.join(src, '.git'))) run('git', ['checkout', '--quiet', '--force', '--detach', commit], {cwd: src});
  else run('git', ['worktree', 'add', '--force', '--detach', src, commit], {cwd: root});
  console.error(`lab: building reference ${spec} (${commit.slice(0, 12)}) in ${path.relative(root, src)}`);
  run('cargo', ['build', '--locked', '--release', '-p', 'uorc'], {cwd: src, stdio: ['ignore', 'inherit', 'inherit'], env: {...process.env, CARGO_TARGET_DIR: path.join(lab, 'ref', 'target')}});
  fs.mkdirSync(path.dirname(bin), {recursive: true});
  fs.copyFileSync(path.join(lab, 'ref', 'target', 'release', 'uorc'), bin);
  fs.chmodSync(bin, 0o755);
  return {spec, commit, bin};
}
const describe = bin => ({path: bin, version: spawnSync(bin, ['--version'], {encoding: 'utf8'}).stdout?.trim() ?? null, sha256: sha(fs.readFileSync(bin))});

// ------------------------------------------------------------------ corpus

const corpusDir = path.join(lab, 'corpus');
const stdlib = () => run('python3', ['-I', '-c', 'import sysconfig; print(sysconfig.get_paths()["stdlib"])']).trim();
const stdlibPy = dir => run('find', ['.', '-name', '*.py', '-not', '-path', '*/site-packages/*', '-not', '-path', '*/__pycache__/*'], {cwd: dir})
  .split('\n').filter(Boolean).sort();
const fill = (chunks, size) => {
  const out = Buffer.alloc(size); let at = 0;
  if (!chunks.length) fail('no source files for a generated input');
  for (let i = 0; at < size; i = (i + 1) % chunks.length) at += chunks[i]().copy(out, at);
  return out;
};
const generators = {
  // A tar (GNU format, fixed owner, mode and time) of the stdlib .py files.
  py: size => {
    const dir = stdlib(), list = path.join(lab, 'py-files.txt');
    fs.mkdirSync(lab, {recursive: true});
    fs.writeFileSync(list, stdlibPy(dir).join('\n') + '\n');
    const tar = spawnSync('tar', ['-c', '-C', dir, '--format=gnu', '--owner=0', '--group=0', '--numeric-owner', '--mode=644', '--mtime=@0', '--no-recursion', '-T', list], {maxBuffer: 2 ** 31});
    if (tar.status !== 0) fail(`tar failed: ${tar.stderr}`);
    return fill([() => tar.stdout], size);
  },
  // ELF files from /usr/bin, by name, concatenated.
  elf: size => {
    const files = fs.readdirSync('/usr/bin').sort().map(n => path.join('/usr/bin', n)).filter(p => {
      try { const st = fs.statSync(p); if (!st.isFile()) return false; const fd = fs.openSync(p, 'r'), b = Buffer.alloc(4); fs.readSync(fd, b, 0, 4, 0); fs.closeSync(fd); return b.equals(Buffer.from([0x7f, 0x45, 0x4c, 0x46])); } catch { return false; }
    });
    return fill(files.map(p => () => fs.readFileSync(p)), size);
  },
  // The stdlib .py sources, concatenated: plain text, four times --size.
  text: size => { const dir = stdlib(); return fill(stdlibPy(dir).map(p => () => fs.readFileSync(path.join(dir, p))), 4 * size); },
};
const fetched = {
  canterbury: dir => run('sh', ['-c', 'curl -fsSL --retry 5 --retry-all-errors https://corpus.canterbury.ac.nz/resources/cantrbry.tar.gz | tar -xz -C "$1"', 'sh', dir]),
  silesia: dir => run('sh', ['-c', 'curl -fsSL --retry 5 --retry-all-errors -o "$1/s.zip" https://sun.aei.polsl.pl/~sdeor/corpus/silesia.zip && unzip -q -d "$1" "$1/s.zip" && rm "$1/s.zip" && find "$1" -mindepth 2 -type f -exec mv -t "$1" {} +', 'sh', dir]),
};
// The inputs of a run: [{name, file}], generated or fetched on first use.
function inputs(defaults = 'py,elf') {
  const size = bytesOf(args.size ?? '8M');
  const named = (args._.length && !args.corpus) ? [] : String(args.corpus ?? defaults).split(',').filter(Boolean);
  const out = [];
  for (const n of named) {
    if (generators[n]) {
      const file = path.join(corpusDir, `${n}-${label(n === 'text' ? 4 * size : size)}.bin`);
      if (!fs.existsSync(file)) { fs.mkdirSync(corpusDir, {recursive: true}); fs.writeFileSync(`${file}.tmp`, generators[n](size)); fs.renameSync(`${file}.tmp`, file); }
      out.push({name: path.basename(file, '.bin'), file});
    } else if (fetched[n]) {
      const dir = path.join(corpusDir, n);
      if (!fs.existsSync(dir)) { fs.mkdirSync(`${dir}.tmp`, {recursive: true}); fetched[n](`${dir}.tmp`); fs.renameSync(`${dir}.tmp`, dir); }
      out.push(...fs.readdirSync(dir).sort().map(x => ({name: `${n}/${x}`, file: path.join(dir, x)})));
    } else fail(`unknown corpus ${n} (py, elf, text, canterbury, silesia)`);
  }
  for (const a of args._) {
    const p = path.resolve(a);
    if (!fs.existsSync(p)) fail(`no input ${a}`);
    if (fs.statSync(p).isDirectory()) out.push(...fs.readdirSync(p).sort().filter(x => fs.statSync(path.join(p, x)).isFile()).map(x => ({name: `${path.basename(p)}/${x}`, file: path.join(p, x)})));
    else out.push({name: path.basename(p), file: p});
  }
  return out.map(i => ({...i, bytes: fs.statSync(i.file).size, sha256: sha(fs.readFileSync(i.file))}));
}

// ------------------------------------------------------------- measurement

// One process, measured by a Python helper: wait4 gives exact user/sys CPU
// and peak RSS; a sampler thread reads /proc every interval for the busy
// cores of the process (its utime+stime ticks) and of the host (/proc/stat).
const PROBE = String.raw`
import json, os, subprocess, sys, threading, time
spec = json.loads(sys.argv[1]); n = os.cpu_count(); tck = os.sysconf('SC_CLK_TCK'); dt = spec['interval']
def host():
    v = list(map(int, open('/proc/stat').readline().split()[1:]))
    return sum(v), v[3] + v[4]
def ticks(pid, top):
    s = open(f'/proc/{pid}/stat').read(); x = s[s.rindex(')') + 2:].split()
    t = int(x[11]) + int(x[12]) + (int(x[13]) + int(x[14]) if top else 0)
    for task in os.listdir(f'/proc/{pid}/task'):
        try: kids = open(f'/proc/{pid}/task/{task}/children').read().split()
        except Exception: kids = []
        for k in kids:
            try: t += ticks(k, False)
            except Exception: pass
    return t
def proc(pid):
    # The process, its reaped children (cutime) and its live descendants.
    try: return ticks(pid, True)
    except Exception: return None
out = open(spec['stdout'], 'wb') if spec.get('stdout') else subprocess.DEVNULL
samples = []; done = threading.Event()
w0 = time.monotonic()
p = subprocess.Popen(spec['cmd'], stdout=out, stderr=subprocess.PIPE)
def sampler():
    t0, i0 = host(); c0 = proc(p.pid) or 0; s0 = time.monotonic()
    while not done.wait(dt):
        t1, i1 = host(); c1 = proc(p.pid); s1 = time.monotonic()
        if c1 is None: break
        samples.append((round(s1 - w0, 3), (c1 - c0) / tck / (s1 - s0), (1 - (i1 - i0) / max(1, t1 - t0)) * n))
        t0, i0, c0, s0 = t1, i1, c1, s1
th = threading.Thread(target=sampler, daemon=True); th.start()
err = p.stderr.read()
_, status, ru = os.wait4(p.pid, 0)
wall = time.monotonic() - w0; done.set(); th.join()
def buckets(k):
    if not samples: return []
    m = max(1, -(-len(samples) // 10))
    return [round(sum(s[k] for s in samples[i:i + m]) / len(samples[i:i + m]), 2) for i in range(0, len(samples), m)]
print(json.dumps({'exit': os.waitstatus_to_exitcode(status), 'stderr': err.decode('utf8', 'replace')[-2000:],
  'wall_s': wall, 'user_s': ru.ru_utime, 'sys_s': ru.ru_stime, 'cpu_s': ru.ru_utime + ru.ru_stime,
  'peak_rss_kib': ru.ru_maxrss, 'busy_mean': (ru.ru_utime + ru.ru_stime) / wall if wall else 0,
  'busy_peak': max((s[1] for s in samples), default=None), 'host_busy_mean': sum(s[2] for s in samples) / len(samples) if samples else None,
  'host_busy_peak': max((s[2] for s in samples), default=None), 'samples': len(samples), 'interval_s': dt,
  'timeline': buckets(1), 'host_timeline': buckets(2)}))
`;
const interval = Number(args['interval-ms'] ?? 50) / 1000;
function probe(cmd, stdout) {
  const r = spawnSync('python3', ['-I', '-c', PROBE, JSON.stringify({cmd, stdout, interval})], {encoding: 'utf8', maxBuffer: 2 ** 26});
  if (r.status !== 0) fail(`probe failed: ${r.stderr}`);
  return JSON.parse(r.stdout);
}
// Compress and decompress `input` with `bin` at `threads`; the archive stays
// at `arc` for the caller to compare.
function cycle(bin, input, threads, arc) {
  const back = `${arc}.back`, steps = String(args.steps ?? 0);
  const c = probe([bin, 'compress', '--force', '--steps', steps, '--threads', String(threads), input.file, '-o', arc]);
  const result = {compress: c};
  if (c.exit !== 0) return {...result, error: `compress exit ${c.exit}: ${c.stderr.trim()}`};
  result.archive_bytes = fs.statSync(arc).size;
  result.archive_sha256 = sha(fs.readFileSync(arc));
  const d = probe([bin, 'decompress', '--force', '--threads', String(threads), arc, '-o', back]);
  result.decompress = d;
  result.round_trip = d.exit === 0 && sha(fs.readFileSync(back)) === input.sha256;
  if (d.exit !== 0) result.error = `decompress exit ${d.exit}: ${d.stderr.trim()}`;
  fs.rmSync(back, {force: true});
  return result;
}
const threadList = () => String(args.threads ?? `1,${os.availableParallelism()}`).split(',').map(Number);
const work = () => { fs.mkdirSync(path.join(lab, 'tmp'), {recursive: true}); return fs.mkdtempSync(path.join(lab, 'tmp', 'run-')); };

// ---------------------------------------------------------------- commands

function hosts() {
  const bin = binary(args.uorc), ins = inputs(), dir = work(), rows = [], results = [];
  for (const input of ins) for (const threads of threadList()) {
    const r = cycle(bin, input, threads, path.join(dir, 'a.uorc'));
    results.push({input: input.name, threads, ...r});
    for (const op of ['compress', 'decompress']) {
      const m = r[op]; if (!m) continue;
      rows.push([input.name, threads, op, f(m.wall_s, 3), f(m.cpu_s, 3), f(m.busy_mean), f(m.busy_peak, 1), f(m.host_busy_mean), f(mib(m.peak_rss_kib), 1),
        f(input.bytes / 1e6 / m.wall_s), r.archive_bytes ?? 'n/a', r.round_trip ? 'ok' : (r.error ?? 'FAIL'), m.timeline.join(' ')]);
    }
  }
  fs.rmSync(dir, {recursive: true, force: true});
  console.log(table(['input', 'threads', 'op', 'wall s', 'cpu s', 'busy mean', 'busy peak', 'host busy', 'peak RSS MiB', 'MB/s', 'archive', 'round trip', 'busy timeline (10 buckets)'], rows));
  console.log(`\nbusy: cores the uorc process kept busy (cpu/wall; peak of ${interval * 1000} ms samples); host busy: all of /proc/stat, other jobs included. ${host().cores} cores. Samples count CPU in 10 ms ticks, so a peak reads to about ±${f(0.01 / interval, 1)} cores.`);
  writeJson('hosts', {spec: 'uorc/lab-hosts/1', host: host(), uorc: describe(bin), steps: Number(args.steps ?? 0), inputs: ins.map(({file, ...i}) => i), results});
  if (results.some(r => !r.round_trip)) process.exit(1);
}

function ab() {
  if (!args.ref) fail('ab needs --ref <git rev | path to a uorc binary>');
  const A = binary(args.uorc), ref = reference(args.ref), B = ref.bin, ins = inputs(), runs = Number(args.runs ?? 3), dir = work();
  const sides = {A, B}, results = [], rows = [], sizeRows = [];
  let differ = 0, broken = 0;
  for (const input of ins) for (const threads of threadList()) {
    const per = {A: [], B: []};
    for (let i = 0; i < runs; i++) for (const s of i % 2 ? ['B', 'A'] : ['A', 'B']) per[s].push(cycle(sides[s], input, threads, path.join(dir, `${s}-${i}.uorc`)));
    const shas = s => [...new Set(per[s].map(r => r.archive_sha256))];
    let firstDiff = null;
    const [a0, b0] = [path.join(dir, 'A-0.uorc'), path.join(dir, 'B-0.uorc')];
    const identical = shas('A').length === 1 && shas('B').length === 1 && shas('A')[0] === shas('B')[0] && shas('A')[0] !== undefined;
    if (!identical && fs.existsSync(a0) && fs.existsSync(b0)) {
      const x = fs.readFileSync(a0), y = fs.readFileSync(b0);
      firstDiff = x.findIndex((v, k) => v !== y[k]);
      if (firstDiff < 0) firstDiff = Math.min(x.length, y.length);
    }
    const ok = ['A', 'B'].every(s => per[s].every(r => r.round_trip));
    differ += !identical; broken += !ok;
    const summary = {};
    for (const s of ['A', 'B']) for (const op of ['compress', 'decompress']) for (const k of ['wall_s', 'cpu_s', 'busy_mean', 'busy_peak', 'peak_rss_kib'])
      (summary[s] ??= {})[`${op}_${k}`] = stats(per[s].map(r => r[op]?.[k]).filter(v => v != null));
    results.push({input: input.name, threads, identical, first_differing_offset: firstDiff, round_trip: ok,
      archive_bytes: {A: per.A[0].archive_bytes, B: per.B[0].archive_bytes}, archive_sha256: {A: shas('A'), B: shas('B')}, summary, runs: per});
    for (const op of ['compress', 'decompress']) {
      const [a, b] = [summary.A[`${op}_wall_s`], summary.B[`${op}_wall_s`]];
      rows.push([input.name, threads, op, `${f(a?.median, 3)} (±${f(100 * (a?.spread ?? NaN), 0)}%)`, `${f(b?.median, 3)} (±${f(100 * (b?.spread ?? NaN), 0)}%)`,
        a && b ? `${f(100 * (a.median / b.median - 1), 1)}%` : 'n/a', `${f(summary.A[`${op}_cpu_s`]?.median, 3)} / ${f(summary.B[`${op}_cpu_s`]?.median, 3)}`,
        `${f(summary.A[`${op}_busy_mean`]?.median)} / ${f(summary.B[`${op}_busy_mean`]?.median)}`, `${f(mib(summary.A[`${op}_peak_rss_kib`]?.median ?? NaN), 1)} / ${f(mib(summary.B[`${op}_peak_rss_kib`]?.median ?? NaN), 1)}`]);
    }
    const [sa, sb] = [per.A[0].archive_bytes, per.B[0].archive_bytes];
    sizeRows.push([input.name, threads, input.bytes, sa ?? 'n/a', sb ?? 'n/a', sa != null && sb != null ? sa - sb : 'n/a', identical ? 'identical' : `differ at ${firstDiff ?? '?'}`, ok ? 'ok' : 'FAIL']);
  }
  fs.rmSync(dir, {recursive: true, force: true});
  console.log(`A: ${args.uorc ?? 'working tree'}  B: ${args.ref}${ref.commit ? ` (${ref.commit.slice(0, 12)})` : ''}  runs: ${runs} each, interleaved A,B,B,A,...\n`);
  console.log(table(['input', 'threads', 'op', 'A wall s (spread)', 'B wall s (spread)', 'A vs B', 'cpu s A / B', 'busy A / B', 'peak RSS MiB A / B'], rows));
  console.log('');
  console.log(table(['input', 'threads', 'input bytes', 'A archive', 'B archive', 'A − B', 'bytes', 'round trips'], sizeRows));
  console.log(`\nmedians over ${runs} runs; spread is (max − min) / median. ${differ ? `${differ} archive set(s) differ.` : 'Every archive is byte-identical.'}${broken ? ` ${broken} round-trip failure(s).` : ''}`);
  writeJson('ab', {spec: 'uorc/lab-ab/1', host: host(), A: {spec: args.uorc ?? 'working tree', ...describe(A)}, B: {...ref, ...describe(B)},
    steps: Number(args.steps ?? 0), runs, inputs: ins.map(({file, ...i}) => i), identical: differ === 0, results});
  if (broken || (args.identical && differ)) process.exit(1);
}

// Cachegrind's out file: fl= and fn= set the current file and function; each
// other numeric line is "line count...". Totals per line, function and file.
function cachegrind(file, top) {
  const lines = new Map(), lineFns = new Map(), fns = new Map(), files = new Map();
  let fl = '?', fn = '?', total = 0;
  const add = (m, k, v) => m.set(k, (m.get(k) ?? 0) + v);
  for (const l of fs.readFileSync(file, 'utf8').split('\n')) {
    if (l.startsWith('fl=')) fl = l.slice(3);
    else if (l.startsWith('fn=')) fn = l.slice(3);
    else if (/^\d/.test(l)) {
      const [line, ir] = l.split(' ').map(Number);
      if (!ir) continue;
      total += ir; add(lines, `${fl}\0${line}`, ir); add(lineFns, `${fl}\0${line}\0${fn}`, ir); add(fns, fn, ir); add(files, fl, ir);
    }
  }
  const rel = p => (p.startsWith(root + '/') ? p.slice(root.length + 1)
    : path.normalize(p).replace(/^\/rustc\/[0-9a-f]+\/library\//, 'std:').replace(/^.*\/registry\/src\/[^/]+\//, 'crate:'));
  const sysroot = spawnSync('rustc', ['--print', 'sysroot'], {cwd: root, encoding: 'utf8'}).stdout?.trim();
  const local = p => (sysroot && /^\/rustc\/[0-9a-f]+\/library\//.test(p) ? path.join(sysroot, 'lib/rustlib/src/rust/library', p.replace(/^\/rustc\/[0-9a-f]+\/library\//, '')) : p);
  const source = (p, n) => { try { return fs.readFileSync(local(p), 'utf8').split('\n')[n - 1]?.trim().slice(0, 80) ?? ''; } catch { return ''; } };
  const sorted = m => [...m].sort((a, b) => b[1] - a[1]).slice(0, top);
  return {total_ir: total,
    lines: sorted(lines).map(([k, ir]) => {
      const [p, n] = k.split('\0');
      const inlined = [...lineFns].filter(([x]) => x.startsWith(`${k}\0`)).sort((a, b) => b[1] - a[1]).map(([x, v]) => ({function: x.split('\0')[2], ir: v}));
      return {file: rel(p), line: Number(n), ir, share: ir / total, source: source(p, Number(n)), functions: inlined};
    }),
    functions: sorted(fns).map(([func, ir]) => ({function: func, ir, share: ir / total})),
    files: sorted(files).map(([p, ir]) => ({file: rel(p), ir, share: ir / total}))};
}

function cost() {
  if (spawnSync('valgrind', ['--version']).error) fail('valgrind is not installed: apt-get install -y valgrind');
  const bin = binary(args.uorc, {CARGO_PROFILE_RELEASE_DEBUG: 'line-tables-only'}, path.join(lab, 'cost-target'));
  const n = bytesOf(args.bytes ?? '512K'), top = Number(args.top ?? 25), op = args.op ?? 'decompress';
  if (!['decompress', 'compress', 'both'].includes(op)) fail('--op is decompress, compress or both');
  const name = args.input ?? 'py';
  let src = path.resolve(name);
  if (generators[name]) { args.size ??= '8M'; args.corpus = name; args._ = []; src = inputs()[0].file; }
  if (fs.statSync(src).size < n) fail(`${src} is shorter than --bytes ${label(n)}`);
  const dir = work(), input = path.join(dir, 'in'), arc = path.join(dir, 'in.uorc');
  fs.writeFileSync(input, fs.readFileSync(src).subarray(0, n));
  run(bin, ['compress', '--threads', '1', input, '-o', arc]);
  const report = {spec: 'uorc/lab-cost/1', host: host(), uorc: describe(bin), input: {source: path.relative(root, src), bytes: fs.statSync(input).size, sha256: sha(fs.readFileSync(input))}, archive_bytes: fs.statSync(arc).size, top, ops: {}};
  for (const o of op === 'both' ? ['compress', 'decompress'] : [op]) {
    const out = path.join(dir, `cachegrind.${o}.out`);
    const argv = o === 'compress' ? ['compress', '--threads', '1', input, '-o', path.join(dir, 'c.uorc')] : ['decompress', '--threads', '1', arc, '-o', path.join(dir, 'back')];
    console.error(`lab: cachegrind ${o} of ${label(n)}`);
    run('valgrind', ['--tool=cachegrind', '--cache-sim=no', `--cachegrind-out-file=${out}`, bin, ...argv]);
    const c = cachegrind(out, top);
    report.ops[o] = c;
    const keep = path.join(lab, 'out', `cachegrind.${o}.out`);
    fs.mkdirSync(path.dirname(keep), {recursive: true}); fs.copyFileSync(out, keep);
    console.log(`\n### ${o}: ${c.total_ir.toLocaleString('en')} instructions for ${label(n)} (${f(c.total_ir / n, 1)} per byte)\n`);
    const pct = x => `${f(100 * x, 1)}%`;
    console.log(table(['function', 'Ir', 'share'], c.functions.map(x => [`\`${x.function.slice(0, 90)}\``, x.ir.toLocaleString('en'), pct(x.share)])));
    console.log('');
    console.log(table(['file:line', 'Ir', 'share', 'mostly in', 'source'], c.lines.map(x => [`${x.file}:${x.line}`, x.ir.toLocaleString('en'), pct(x.share), `\`${(x.functions[0]?.function ?? '').slice(0, 50)}\``, `\`${x.source.replace(/\|/g, '\\|').replace(/`/g, "'")}\``])));
    console.log('');
    console.log(table(['file', 'Ir', 'share'], c.files.slice(0, 10).map(x => [x.file, x.ir.toLocaleString('en'), pct(x.share)])));
    console.log(`\nraw: ${path.relative(root, keep)} (cg_annotate reads it)`);
  }
  fs.rmSync(dir, {recursive: true, force: true});
  writeJson('cost', report);
}

function corpus() {
  const ins = inputs('py,elf,text');
  console.log(table(['input', 'bytes', 'sha256', 'path'], ins.map(i => [i.name, i.bytes, i.sha256.slice(0, 16), path.relative(root, i.file)])));
}

({hosts, ab, cost, corpus}[command] ?? (() => fail(`unknown command ${command} (hosts, ab, cost, corpus, help)`)))();
