// Fetches the public benchmark corpora that frontier.toml registers
// (Canterbury, Silesia) into <out>/<name>/, every member checked against the
// sha256 and byte count the register pins. A corpus host that refuses a runner
// cannot fail a release: sources are tried in order and the first one whose
// files all match the pins wins.
//
//   node scripts/corpora.mjs <out> [--corpus canterbury,silesia] [--publish]
//
// Sources, in order:
//   1. this repository's `corpora` release asset <name>.tar.gz (storage the
//      repository controls; the files are the pinned ones or nothing is used),
//   2. the corpus's own source URL (frontier.toml `source`),
//   3. the Internet Archive's copy of that URL,
//   4. per-file copies of single members on GitHub (MIRRORS below).
// Members still missing after a source are taken from the next one, so a
// partial source is never wasted. --publish (with GH_TOKEN and contents:write)
// uploads <name>.tar.gz to the `corpora` release when it was assembled from
// sources 2-4, so the next run reads source 1. A failed upload only warns.

import {execFileSync} from 'node:child_process';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const RELEASE_TAG = 'corpora';
const AGENT = 'Mozilla/5.0 (X11; Linux x86_64) uorc-corpora';

// Single members published elsewhere byte for byte; a copy is used only when
// its sha256 matches the register.
const MIRRORS = {
  canterbury: Object.fromEntries(['alice29.txt', 'asyoulik.txt', 'lcet10.txt', 'plrabn12.txt']
    .map(f => [f, {url: `https://raw.githubusercontent.com/google/brotli/master/tests/testdata/${f}`}])),
  silesia: Object.fromEntries(['dickens', 'mozilla', 'mr', 'nci', 'ooffice', 'osdb', 'reymont', 'samba', 'sao', 'webster', 'x-ray', 'xml']
    .map(f => [f, {url: `https://raw.githubusercontent.com/MiloszKrajewski/SilesiaCorpus/master/${f}.zip`, zip: true}])),
};

// The [[corpus]] entries of frontier.toml with their pinned members.
export function register(text) {
  const corpora = [];
  let corpus = null, member = null;
  for (const raw of text.split('\n')) {
    const line = raw.trim();
    if (line === '[[corpus]]') { corpus = {members: []}; member = null; corpora.push(corpus); continue; }
    if (line === '[[corpus.members]]') { member = {}; corpus.members.push(member); continue; }
    if (line.startsWith('[')) { corpus = member = null; continue; }
    const m = /^(\w+)\s*=\s*(?:"([^"]*)"|(\d+)|(true|false))$/.exec(line);
    if (!m || !corpus) continue;
    const value = m[2] ?? (m[3] !== undefined ? Number(m[3]) : m[4] === 'true');
    (member ?? corpus)[m[1]] = value;
  }
  return corpora;
}

const sha256 = file => crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');

// Members of `corpus` present in `dir` with the pinned digest and size.
export function matching(corpus, dir) {
  return corpus.members.filter(m => {
    const file = path.join(dir, m.name);
    return fs.existsSync(file) && fs.statSync(file).size === m.bytes && sha256(file) === m.sha256;
  }).map(m => m.name);
}

const run = (cmd, args, opts = {}) => execFileSync(cmd, args, {stdio: ['ignore', 'pipe', 'pipe'], ...opts});
const tryRun = (cmd, args, opts) => { try { run(cmd, args, opts); return true; } catch { return false; } };
const download = (url, file) => tryRun('curl', ['-fsSL', '--retry', '3', '--retry-all-errors', '--max-time', '900', '-A', AGENT, '-o', file, url]);

// Unpacks an archive and moves every regular file named like a missing member
// into `dir`, whatever directory the archive kept it in.
function adopt(archive, corpus, dir, have) {
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), 'corpora-'));
  try {
    const ok = archive.endsWith('.zip') ? tryRun('unzip', ['-q', '-o', archive, '-d', scratch])
      : tryRun('tar', ['-xzf', archive, '-C', scratch]);
    if (!ok) return;
    const wanted = new Set(corpus.members.map(m => m.name).filter(n => !have.has(n)));
    const walk = d => {
      for (const e of fs.readdirSync(d, {withFileTypes: true})) {
        const p = path.join(d, e.name);
        if (e.isDirectory()) walk(p);
        else if (e.isFile() && wanted.has(e.name)) fs.renameSync(p, path.join(dir, e.name));
      }
    };
    walk(scratch);
  } finally {
    fs.rmSync(scratch, {recursive: true, force: true});
  }
}

export function fetchCorpus(corpus, out, {repository, publish, log = console.error, dir = path.join(out, corpus.name)} = {}) {
  fs.mkdirSync(dir, {recursive: true});
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'corpora-dl-'));
  const have = () => new Set(matching(corpus, dir));
  const complete = () => have().size === corpus.members.length;
  const report = label => log(`${corpus.name}: ${label}: ${have().size}/${corpus.members.length} members match the register`);
  try {
    if (complete()) { report('cached'); return {source: 'cached'}; }
    const asset = `https://github.com/${repository}/releases/download/${RELEASE_TAG}/${corpus.name}.tar.gz`;
    const archived = path.join(tmp, 'asset.tar.gz');
    if (repository && download(asset, archived)) adopt(archived, corpus, dir, have());
    report('repository release asset');
    if (complete()) return {source: 'release'};
    const sources = [corpus.source, `https://web.archive.org/web/2024id_/${corpus.source}`];
    for (const url of sources) {
      const file = path.join(tmp, url.endsWith('.zip') ? 'source.zip' : 'source.tar.gz');
      if (download(url, file)) adopt(file, corpus, dir, have());
      fs.rmSync(file, {force: true});
      report(url);
      if (complete()) break;
    }
    if (!complete()) {
      for (const [name, mirror] of Object.entries(MIRRORS[corpus.name] ?? {})) {
        if (have().has(name)) continue;
        const file = path.join(tmp, name + (mirror.zip ? '.zip' : ''));
        if (!download(mirror.url, file)) continue;
        if (mirror.zip) adopt(file, corpus, dir, have());
        else fs.renameSync(file, path.join(dir, name));
      }
      report('per-file mirrors');
    }
    // Anything left that does not match is removed, so a caller never reads it.
    const good = have();
    for (const f of fs.readdirSync(dir)) if (!good.has(f)) fs.rmSync(path.join(dir, f), {recursive: true, force: true});
    const missing = corpus.members.filter(m => !good.has(m.name)).map(m => m.name);
    if (missing.length) throw new Error(`${corpus.name}: no source supplied ${missing.join(', ')} with the pinned sha256`);
    if (publish && repository) publishAsset(corpus, dir, tmp, repository, log);
    return {source: 'assembled'};
  } finally {
    fs.rmSync(tmp, {recursive: true, force: true});
  }
}

function publishAsset(corpus, dir, tmp, repository, log) {
  const archive = path.join(tmp, `${corpus.name}.tar.gz`);
  const names = corpus.members.map(m => m.name).sort();
  run('tar', ['-czf', archive, '-C', dir, ...names]);
  const gh = args => tryRun('gh', ['--repo', repository, ...args]);
  const notes = 'Benchmark corpora pinned by frontier.toml (sha256 per member), mirrored by scripts/corpora.mjs so a corpus host refusing a runner cannot fail a release. Not a uorc release.';
  if (!gh(['release', 'view', RELEASE_TAG]) &&
      !gh(['release', 'create', RELEASE_TAG, '--prerelease', '--latest=false', '--title', 'Benchmark corpora', '--notes', notes])) {
    log(`::warning::${corpus.name}: could not create the ${RELEASE_TAG} release; the next run fetches from the sources again`);
    return;
  }
  if (gh(['release', 'upload', RELEASE_TAG, archive, '--clobber'])) log(`${corpus.name}: published ${RELEASE_TAG}/${corpus.name}.tar.gz`);
  else log(`::warning::${corpus.name}: upload to the ${RELEASE_TAG} release failed`);
}

// The registered corpora that have a public source URL.
export const publicCorpora = () => register(fs.readFileSync(path.join(ROOT, 'frontier.toml'), 'utf8')).filter(c => /^https?:/.test(c.source ?? ''));

function main(argv) {
  const out = argv.find(a => !a.startsWith('--'));
  if (!out) throw new Error('usage: node scripts/corpora.mjs <out> [--corpus canterbury,silesia] [--publish]');
  const i = argv.indexOf('--corpus');
  const all = publicCorpora();
  const names = i >= 0 ? argv[i + 1].split(',') : all.map(c => c.name);
  const repository = process.env.GITHUB_REPOSITORY ?? 'UOR-Foundation/compression';
  for (const name of names) {
    const corpus = all.find(c => c.name === name);
    if (!corpus) throw new Error(`frontier.toml registers no public corpus ${name}`);
    fetchCorpus(corpus, out, {repository, publish: argv.includes('--publish')});
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { main(process.argv.slice(2)); } catch (e) { console.error(e.message); process.exit(1); }
}
