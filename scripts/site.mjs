#!/usr/bin/env node
// Assembles the Pages site as a Prism browser closure v2 (PrismPM SPEC §1.2.1):
// the six-file page (index.html, app.css, app.js, uorc.js, uorc_bg.wasm,
// provenance.json) and the ten Hologram brand files beside it (five web fonts,
// their two OFL licences, icon.svg, icon-512.png and manifest.webmanifest), all
// flat and same-origin, with no remote font, image or script. provenance.json
// names the SHA-256 of every other closure file. The brand files are the
// vendored `hologram-brand` dependency (vendor/hologram-brand, byte-identical to
// PrismPM's at 898dcdf, held by its MANIFEST.sha256). The release's benchmark
// page (bench.html, bench.json, bench.md) is published beside the closure, and
// so is model.json: the release's performance theorems (the UC-PERF rows of the
// claim ledger), its closed claims (frontier.toml) and its engine state
// (model/engine.toml), which the page's model view shows.
//
//   node scripts/site.mjs <out> --wasm <uorc_wasm.wasm> --version <tag> [--bench <dir>] [--frontier <decided frontier.toml>] [--model-notes <file>]
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const BRAND = path.join(ROOT, 'vendor/hologram-brand');
export const NAME = 'uorc';
export const STEM = 'uorc';
export const BRAND_FILES = {
  'Archivo-SemiBold.woff2': 'fonts/Archivo-SemiBold.woff2',
  'Geist-Medium.woff2': 'fonts/Geist-Medium.woff2',
  'Geist-Regular.woff2': 'fonts/Geist-Regular.woff2',
  'Geist-SemiBold.woff2': 'fonts/Geist-SemiBold.woff2',
  'GeistMono-Regular.woff2': 'fonts/GeistMono-Regular.woff2',
  'OFL-Archivo.txt': 'fonts/OFL-Archivo.txt',
  'OFL-Geist.txt': 'fonts/OFL-Geist.txt',
  'icon-512.png': 'icon-512.png',
  'icon.svg': 'icon.svg',
};
export const PAGE_FILES = ['index.html', 'app.css', 'app.js', `${STEM}.js`];
export const BENCH_FILES = ['bench.html', 'bench.json', 'bench.md'];
/** Published beside the closure from the repository's own model files. */
export const MODEL_FILE = 'model.json';
const BESIDE = [...BENCH_FILES, MODEL_FILE];

/**
 * The TOML the model files use: tables, arrays of tables, and keys holding
 * strings, integers, Booleans or one-line arrays of them. Anything else is
 * refused, so a file the page cannot read faithfully fails the build.
 */
export function readToml(text, name = 'toml') {
  const root = {};
  let at = root;
  const scalar = (raw, line) => {
    const v = raw.trim();
    if (v.startsWith('"')) {
      const m = v.match(/^"((?:[^"\\]|\\.)*)"$/);
      if (!m) throw new Error(`${name}:${line}: unreadable string ${v}`);
      return JSON.parse(`"${m[1]}"`);
    }
    if (v === 'true' || v === 'false') return v === 'true';
    if (/^[+-]?(0x[0-9a-fA-F_]+|[0-9][0-9_]*)$/.test(v)) {
      const n = BigInt(v.replace(/_/g, ''));
      return n <= BigInt(Number.MAX_SAFE_INTEGER) && n >= -BigInt(Number.MAX_SAFE_INTEGER) ? Number(n) : n.toString();
    }
    if (v.startsWith('[') && v.endsWith(']')) {
      const inner = v.slice(1, -1).trim();
      if (!inner) return [];
      return (inner.match(/"(?:[^"\\]|\\.)*"|[^,]+/g) ?? []).map((x) => x.trim()).filter((x) => x).map((x) => scalar(x, line));
    }
    throw new Error(`${name}:${line}: unsupported value ${v}`);
  };
  const strip = (l) => { let q = false; for (let i = 0; i < l.length; i += 1) { if (l[i] === '"' && l[i - 1] !== '\\') q = !q; if (l[i] === '#' && !q) return l.slice(0, i); } return l; };
  text.split('\n').forEach((rawLine, i) => {
    const l = strip(rawLine).trim();
    if (!l) return;
    let m;
    if ((m = l.match(/^\[\[([A-Za-z0-9_.-]+)\]\]$/))) {
      const keys = m[1].split('.');
      let t = root;
      // [[a.b]] appends to b of the last [[a]] table.
      for (const k of keys.slice(0, -1)) { t = t[k] ??= {}; if (Array.isArray(t)) t = t.at(-1); }
      const last = keys.at(-1);
      (t[last] ??= []).push((at = {}));
    } else if ((m = l.match(/^\[([A-Za-z0-9_.-]+)\]$/))) {
      at = root;
      for (const k of m[1].split('.')) { at = at[k] ??= {}; if (Array.isArray(at)) at = at.at(-1); }
    } else if ((m = l.match(/^([A-Za-z0-9_-]+|"[^"]+")\s*=\s*(.+)$/))) {
      at[m[1].replace(/^"|"$/g, '')] = scalar(m[2], i + 1);
    } else throw new Error(`${name}:${i + 1}: unsupported line ${l}`);
  });
  return root;
}

/**
 * model.json: the theorems (the ledger's UC-PERF rows), the claims of
 * frontier.toml and the engine state of model/engine.toml, as the page shows
 * them. A file that does not exist yet contributes nothing.
 */
export function modelState(root = ROOT, { frontier } = {}) {
  const read = (rel) => (fs.existsSync(path.join(root, rel)) ? readToml(fs.readFileSync(path.join(root, rel), 'utf8'), rel) : {});
  const ledger = read('model/ledger.toml').claim ?? [];
  const theorems = ledger.filter((c) => c.id.startsWith('UC-PERF-'))
    .map((c) => ({ id: c.id, statement: c.statement, evidence: c.evidence_kind }));
  const register = frontier ? readToml(fs.readFileSync(frontier, 'utf8'), frontier) : read('frontier.toml');
  const claims = (register.claim ?? []).map((c) => ({
    id: c.id, form: c.form, codec: c.codec ?? c.candidate ?? '', candidate: c.candidate ?? null, incumbent: c.incumbent ?? null,
    family: c.family ?? null, corpus: c.corpus ?? c.family ?? '', bytes: c.bytes ?? null, steps: c.steps ?? null, status: c.status,
  }));
  // model/engine.toml (uorc/engine/1): [[weight]] names a [[strategy]] by
  // index (its point profile word, bit 31 the x86 filter); [[generator]]
  // weights order the engine's search.
  const engine = read('model/engine.toml');
  const strategies = engine.strategy ?? [];
  const weights = [
    ...(engine.weight ?? []).map((w) => ({ strategy: strategies[w.strategy]?.word ?? String(w.strategy), family: w.family, weight: w.value })),
    ...(engine.generator ?? []).map((g) => ({ strategy: `generator ${g.generator}`, family: g.family, weight: g.value })),
  ];
  return {
    schema: 'uorc/model-state/1',
    theorems,
    claims,
    engine: { tick: engine.tick ?? null, rule30: engine.rule30_state ?? null, strategies: strategies.length, weights },
  };
}

/** The closure v2 file names: the six-file page and the ten brand files. */
export function closureV2() {
  return [...PAGE_FILES, `${STEM}_bg.wasm`, 'provenance.json', ...Object.keys(BRAND_FILES), 'manifest.webmanifest'].sort();
}

const sha256 = (bytes) => crypto.createHash('sha256').update(bytes).digest('hex');

/** The web app manifest: the bytes PrismPM's brand::web_manifest writes for `name`. */
export function webManifest(name) {
  const n = JSON.stringify(name);
  return Buffer.from(`{"name":${n},"short_name":${n},"start_url":"./","scope":"./","display":"standalone","background_color":"#151312","theme_color":"#151312","icons":[{"src":"icon.svg","sizes":"any","type":"image/svg+xml","purpose":"any"},{"src":"icon-512.png","sizes":"512x512","type":"image/png","purpose":"any maskable"}]}\n`);
}

/** The vendored brand files, each checked against the vendor manifest. */
function brandFiles() {
  const manifest = new Map(
    fs.readFileSync(path.join(BRAND, 'MANIFEST.sha256'), 'utf8').trim().split('\n')
      .map((line) => line.split(/\s+/)).map(([digest, file]) => [file, digest]),
  );
  return Object.entries(BRAND_FILES).map(([name, source]) => {
    const bytes = fs.readFileSync(path.join(BRAND, source));
    if (manifest.get(source) !== sha256(bytes)) throw new Error(`vendor/hologram-brand/${source} differs from its MANIFEST.sha256`);
    return [name, bytes];
  });
}

/** Every same-origin reference a page file makes, and every remote one. */
export function references(text) {
  const refs = [...text.matchAll(/(?:href|src)="([^"]+)"|url\(([^)]+)\)|(?:load|fetch|import)\(\s*"([^"]+)"|from "([^"]+)"/g)]
    .map((m) => (m[1] ?? m[2] ?? m[3] ?? m[4]).replace(/^["']|["']$/g, ''));
  return {
    local: refs.filter((r) => !/^[a-z]+:/i.test(r) && !r.startsWith('#')).map((r) => r.replace(/^\.\//, '')).filter((r) => r !== ''),
    remote: refs.filter((r) => /^[a-z]+:/i.test(r)),
  };
}

/** Builds the site into `out` and returns its provenance. */
export function build(out, { wasm, version, bench, frontier }) {
  fs.rmSync(out, { recursive: true, force: true });
  fs.mkdirSync(out, { recursive: true });
  const files = new Map();
  for (const name of PAGE_FILES) files.set(name, fs.readFileSync(path.join(ROOT, 'web', name)));
  files.set(`${STEM}_bg.wasm`, fs.readFileSync(wasm));
  for (const [name, bytes] of brandFiles()) files.set(name, bytes);
  files.set('manifest.webmanifest', webManifest(NAME));
  const provenance = {
    schema: 'uorc/browser-provenance/2',
    closure: 'prism/browser-closure/2',
    application: NAME,
    version,
    wasm_sha256: sha256(files.get(`${STEM}_bg.wasm`)),
    files: Object.fromEntries([...files].sort(([a], [b]) => (a < b ? -1 : 1)).map(([name, bytes]) => [name, sha256(bytes)])),
  };
  files.set('provenance.json', Buffer.from(`${JSON.stringify(provenance, null, 2)}\n`));
  const site = new Set([...files.keys()]);
  if (bench) for (const name of BENCH_FILES) { files.set(name, fs.readFileSync(path.join(bench, name))); site.add(name); }
  files.set(MODEL_FILE, Buffer.from(`${JSON.stringify(modelState(ROOT, { frontier }), null, 2)}\n`));
  site.add(MODEL_FILE);
  check(files, site);
  for (const [name, bytes] of files) fs.writeFileSync(path.join(out, name), bytes);
  return provenance;
}

/** Refuses a site that is not closure v2, or whose pages load anything remote or outside it. */
export function check(files, site = new Set(files.keys())) {
  const closure = [...files.keys()].filter((name) => !BESIDE.includes(name)).sort();
  if (JSON.stringify(closure) !== JSON.stringify(closureV2())) throw new Error(`not closure v2: ${closure.join(', ')}`);
  for (const name of files.keys()) if (name.includes('/')) throw new Error(`not flat: ${name}`);
  for (const name of [...PAGE_FILES, ...BENCH_FILES.filter((n) => n.endsWith('.html'))]) {
    if (!files.has(name)) continue;
    const { local, remote } = references(files.get(name).toString('utf8'));
    const loads = remote.filter((r) => !/^https:\/\/github\.com\//.test(r));
    if (loads.length) throw new Error(`${name} references ${loads.join(', ')}`);
    for (const ref of local) if (!site.has(ref)) throw new Error(`${name} references ${ref}, which the site does not hold`);
  }
  const provenance = JSON.parse(files.get('provenance.json'));
  for (const name of closure.filter((n) => n !== 'provenance.json')) {
    if (provenance.files[name] !== sha256(files.get(name))) throw new Error(`provenance.json misnames ${name}`);
  }
  if (Object.keys(provenance.files).length !== closure.length - 1) throw new Error('provenance.json names a file outside the closure');
}

/** The release notes' account of model.json, in Markdown. */
export function modelNotes(m) {
  const count = (status) => m.claims.filter((c) => c.status === status).length;
  const lines = ['## The model on the page', ''];
  lines.push(m.theorems.length ? `Performance rows: ${m.theorems.map((t) => `${t.id} (${t.evidence})`).join(', ')}.` : 'Performance rows: none recorded.');
  lines.push('', `Claims: ${m.claims.length} recorded, ${count('proved')} proved, ${count('refuted')} refuted (listed above).`);
  lines.push('', m.engine.tick == null ? 'Engine: no tick recorded.' : `Engine: tick ${m.engine.tick}, rule30 register ${m.engine.rule30}, ${m.engine.strategies} strategies in the portfolio, ${m.engine.weights.length} weights.`);
  lines.push('', 'The full register is model.json, on the Pages site and attached to this release.', '');
  return lines.join('\n');
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const [out, ...rest] = process.argv.slice(2);
  const opt = (key) => { const i = rest.indexOf(key); return i < 0 ? undefined : rest[i + 1]; };
  if (!out || !opt('--wasm') || !opt('--version')) {
    console.error('usage: node scripts/site.mjs <out> --wasm <file> --version <tag> [--bench <dir>] [--frontier <decided frontier.toml>] [--model-notes <file>]');
    process.exit(2);
  }
  const p = build(out, { wasm: opt('--wasm'), version: opt('--version'), bench: opt('--bench'), frontier: opt('--frontier') });
  if (opt('--model-notes')) fs.writeFileSync(opt('--model-notes'), modelNotes(JSON.parse(fs.readFileSync(path.join(out, MODEL_FILE), 'utf8'))));
  console.log(`site: closure v2, ${Object.keys(p.files).length + 1} files, ${p.application} ${p.version}`);
}
