#!/usr/bin/env node
// The uorc release factory: the PrismPM factory route (PrismPM #566, #579-#581,
// #586) applied to this repository's CI and release.
//
// Each obligation in factory/graph.json declares its own inputs. Its identity is
// the SHA-256 of those inputs' tracked content, its kind, environment and run,
// and the identities of what it needs, in PrismPM's `prism-factory/obligation/1`
// text, so a change re-runs only the obligations that read it. Outcomes become
// receipts on the `factory-receipts` branch, written only by factory.yml from
// the default branch after a same-repository run completes. Routing:
//   proved   a passing receipt for this identity exists; the work is skipped and
//            its evidence (if any) comes from the run that proved it
//   refuted  a hermetic obligation failed on exactly these inputs; it is not
//            run again (a manual re-run, attempt > 1, observes it again)
//   blocked  something it needs is neither proved nor runnable
//   observe  run it
// A failed fetch obligation is observed again, because the network is not an
// input. Priority is a 16-level weight per obligation: a failure raises it by
// one, a pass lowers it by one, and `plan` orders the remaining work by it.
//
//   node scripts/factory.mjs plan [--rev REV] [--context KEY=VALUE]...
//   node scripts/factory.mjs identity NAME [--rev REV] [--context KEY=VALUE]...
//   node scripts/factory.mjs route NAME... [--context KEY=VALUE]... [--expect-run TEXT]
//   node scripts/factory.mjs materialize NAME     (CI: delete what NAME does not declare)
//   node scripts/factory.mjs outcome NAME IDENTITY success|failure DIR
//   node scripts/factory.mjs publish DIR          (factory.yml only: needs GH_TOKEN)
//   node scripts/factory.mjs release-step --context trigger=WORKFLOW   (factory.yml: tag a proved version)
//   node scripts/factory.mjs merge-train          (factory.yml: update, test and merge the head of the merge train)
//   node scripts/factory.mjs export-check | export-record
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

export const BRANCH = process.env.FACTORY_RECEIPTS_BRANCH || 'factory-receipts';
const LEVELS = 16, DEFAULT_LEVEL = 4;
const sha = data => crypto.createHash('sha256').update(data).digest('hex');
const fail = message => { throw new Error(message); };

export function git(args, {cwd = process.cwd(), input, allowFail = false, env} = {}) {
  const r = spawnSync('git', args, {cwd, input, env: env ? {...process.env, ...env} : process.env, maxBuffer: 1 << 30});
  if (r.status !== 0 && !allowFail) fail(`git ${args.join(' ')}: ${r.stderr}`);
  return r.status === 0 ? r.stdout : null;
}
const root = () => git(['rev-parse', '--show-toplevel']).toString().trim();

/// Every tracked entry at REV, in Git tree order: its path and `mode:object`.
export function tree(rev, cwd) {
  const entries = [];
  for (const row of git(['ls-tree', '-r', '-z', '--full-tree', rev], {cwd}).toString().split('\0')) {
    if (!row) continue;
    const tab = row.indexOf('\t'); const [mode, , object] = row.slice(0, tab).split(' ');
    const p = row.slice(tab + 1); if (p.includes('\n')) fail(`unsupported path ${JSON.stringify(p)}`);
    entries.push({path: p, object: `${mode}:${object}`});
  }
  return entries;
}

export function loadGraph(dir) {
  const graph = JSON.parse(fs.readFileSync(path.join(dir, 'factory/graph.json'), 'utf8'));
  if (graph.schema !== 'uorc/factory-graph/1') fail('factory/graph.json: unknown schema');
  const seen = new Set();
  for (const o of graph.obligations) {
    if (seen.has(o.name)) fail(`obligation ${o.name} is declared twice`);
    for (const need of o.needs) if (!seen.has(need)) fail(`${o.name} needs ${need}, which is not declared before it`);
    if (!['hermetic', 'fetch', 'gate'].includes(o.kind)) fail(`${o.name}: unknown kind ${o.kind}`);
    seen.add(o.name);
    o.inputs = o.inputs.flatMap(i => i.startsWith('&') ? (graph.sets[i.slice(1)] ?? fail(`${o.name}: unknown set ${i}`)) : [i]);
  }
  return graph;
}

// `closure:ROOTS@DIRS` is the import closure of ROOTS (comma-separated module
// names; `Uorc.Integrity.*` names every module under that directory), each
// module read from the first of DIRS (default `src`) that has it, as the
// export stages client/src over src.
function closurePaths(spec, entries) {
  const [rootList, dirList = 'src'] = spec.split('@');
  const dirs = dirList.split(',');
  const paths = new Set(entries.map(e => e.path)); const found = []; const seen = new Set();
  const stack = rootList.split(',').flatMap(r => {
    if (!r.endsWith('.*')) return [r];
    const prefix = r.slice(0, -2).split('.').join('/') + '/';
    return [...paths].flatMap(p => dirs.map(d => `${d}/`).filter(d => p.startsWith(d + prefix) && p.endsWith('.lex.tex')).map(d => p.slice(d.length, -8).split('/').join('.')));
  });
  while (stack.length) {
    const m = stack.pop(); if (seen.has(m)) continue; seen.add(m);
    const rel = m.split('.').join('/') + '.lex.tex';
    const p = dirs.map(d => `${d}/${rel}`).find(x => paths.has(x)) ?? fail(`closure ${spec}: module ${m} is not tracked`);
    found.push(p);
    const object = entries.find(e => e.path === p).object.split(':')[1];
    for (const i of git(['cat-file', 'blob', object]).toString().matchAll(/\\importmodule\{([^}]+)\}/g)) stack.push(i[1]);
  }
  return new Set(found);
}

// `manifest:PATH` is a Cargo manifest read for workspace resolution only: the
// manifest is an input, and a closure copy gives each target it declares an
// empty file when the target's own sources are not inputs.
const manifestPath = i => i.slice('manifest:'.length);

/// An entry is covered when the longest inclusion it starts with is longer
/// than the longest exclusion (`!prefix`) it starts with.
export function covered(inputs, p, exact) {
  let include = -1, exclude = -1;
  for (const i of inputs) {
    if (i.startsWith('@') || i.startsWith('closure:')) continue;
    if (i.startsWith('manifest:')) { if (p === manifestPath(i)) include = Math.max(include, p.length); continue; }
    if (i.startsWith('!')) { if (p.startsWith(i.slice(1))) exclude = Math.max(exclude, i.length - 1); }
    else if (p.startsWith(i)) include = Math.max(include, i.length);
  }
  if (exact.has(p)) include = Math.max(include, p.length);
  return include > exclude;
}

function exactPaths(o, entries) {
  return new Set(o.inputs.filter(i => i.startsWith('closure:')).flatMap(i => [...closurePaths(i.slice(8), entries)]));
}

/// Identities of every obligation at REV: PrismPM's `prism-factory/obligation/1`.
export function identities(graph, entries, contexts) {
  const ids = {};
  for (const o of graph.obligations) {
    // An obligation that reads a context nobody supplied, or needs one that
    // does, has no identity here (a pull request run never plans `publish`).
    if (o.inputs.some(i => i.startsWith('@') && contexts[i.slice(1)] === undefined) || o.needs.some(n => ids[n] === null)) { ids[o.name] = null; continue; }
    const exact = exactPaths(o, entries);
    let closure = '';
    for (const e of entries) if (covered(o.inputs, e.path, exact)) closure += `file ${e.object} ${e.path}\n`;
    for (const i of o.inputs) if (i.startsWith('@')) {
      closure += `file context:${contexts[i.slice(1)]} ${i}\n`;
    }
    let text = `prism-factory/obligation/1\nname ${o.name}\nkind ${o.kind}\nenvironment ${o.environment}\nrun ${o.run}\n`;
    for (const i of o.inputs) text += `input ${i}\n`;
    for (const n of o.needs) text += `needs ${n} ${ids[n]}\n`;
    ids[o.name] = sha(text + `closure ${sha(closure)}\n`);
  }
  return ids;
}

// Receipts: the index of the receipts branch, one line per receipt:
//   receipt IDENTITY NAME pass|fail RUN ATTEMPT COMMIT EXPIRES
// and its weights: `weight NAME LEVEL`.
export function parseStore(text) {
  const receipts = [], weights = {};
  for (const line of (text ?? '').split('\n')) {
    const f = line.split(' ');
    if (f[0] === 'receipt' && f.length === 8) receipts.push({identity: f[1], name: f[2], result: f[3], run: f[4], attempt: f[5], commit: f[6], expires: f[7]});
    if (f[0] === 'weight' && f.length === 3) weights[f[1]] = Math.min(LEVELS, Math.max(1, Number(f[2]) || DEFAULT_LEVEL));
  }
  return {receipts, weights};
}

export function fetchStore(cwd) {
  if (process.env.FACTORY_STORE) return fs.readFileSync(process.env.FACTORY_STORE, 'utf8');
  if (git(['fetch', '--quiet', '--depth=1', 'origin', `+refs/heads/${BRANCH}:refs/factory/receipts`], {cwd, allowFail: true}) === null) {
    console.error(`factory: no ${BRANCH} branch could be read; every obligation is observed`);
    return '';
  }
  return (git(['cat-file', 'blob', 'refs/factory/receipts:index'], {cwd, allowFail: true}) ?? '').toString();
}

/// Routes in graph order. `inRun` treats an `observe` need as passing later in
/// the same run (a release run builds native before bench); without it a need
/// must already be proved.
export function route(graph, ids, store, {now = new Date(), attempt = 1, inRun = true} = {}) {
  const routes = {};
  for (const o of graph.obligations) {
    if (ids[o.name] === null) { routes[o.name] = {route: 'unplanned', identity: null}; continue; }
    const mine = store.receipts.filter(r => r.identity === ids[o.name] && r.name === o.name && (r.expires === 'never' || new Date(r.expires) > now));
    const pass = mine.filter(r => r.result === 'pass').at(-1), failed = mine.filter(r => r.result === 'fail').at(-1);
    const needs = o.needs.map(n => routes[n].route);
    const ready = needs.every(r => r === 'proved' || (inRun && r === 'observe'));
    let r;
    if (o.kind === 'gate') r = needs.every(x => x === 'proved') ? {route: 'proved'} : {route: 'blocked'};
    else if (!ready) r = {route: 'blocked'};
    else if (pass) r = {route: 'proved', run: pass.run};
    else if (o.kind === 'hermetic' && failed && attempt < 2) r = {route: 'refuted', run: failed.run};
    else r = {route: 'observe'};
    routes[o.name] = {...r, identity: ids[o.name]};
  }
  return routes;
}

export function schedule(graph, routes, weights) {
  return graph.obligations.filter(o => routes[o.name].route === 'observe')
    .map((o, index) => ({name: o.name, level: weights[o.name] ?? DEFAULT_LEVEL, index}))
    .sort((a, b) => b.level - a.level || a.index - b.index);
}

export function reweigh(weights, outcomes) {
  const next = {...weights};
  for (const {name, result} of outcomes) {
    const level = next[name] ?? DEFAULT_LEVEL;
    next[name] = result === 'pass' ? Math.max(1, level - 1) : Math.min(LEVELS, level + 1);
  }
  return next;
}

// The release stages a tag publishes from: built and decided on the exact tree
// by a rehearsal (or an earlier release run) before the tag exists.
export const RELEASE_STAGES = ['native-x86_64-unknown-linux-gnu', 'native-aarch64-unknown-linux-gnu', 'wasm', 'bench', 'frontier'];

/// What the default branch's head needs next to ship its workspace version.
/// Merging a version bump is the release: once the bootstrap gate and every
/// release stage are proved on that tree, the tag is made and published. Only
/// a run that is not itself a release starts a rehearsal, so a rehearsal that
/// fails does not start another.
export function releaseStep({version, tagged, routes, trigger}) {
  const tag = `v${version}`;
  if (tagged) return {step: 'done', tag, why: `${tag} is already tagged`};
  if (routes.gate.route !== 'proved') return {step: 'wait', tag, why: `the bootstrap gate is not proved on this tree yet`};
  const open = RELEASE_STAGES.filter(n => routes[n].route !== 'proved');
  if (!open.length) return {step: 'tag', tag, why: `the gate and every release stage are proved on this tree`};
  if (trigger === 'release') return {step: 'wait', tag, why: `a release run finished without proving ${open.join(', ')}; fix it and dispatch a rehearsal`};
  return {step: 'rehearse', tag, why: `${open.join(', ')} not proved on this tree`};
}

// The merge train. A non-draft pull request into the default branch that
// carries the `merge-train` label rides it, in the order the label was applied
// (then by number). Only the head of the train moves. Branch protection is
// strict: a request merges only when its head contains the default branch and
// the required checks passed on that head, so each merge makes the next rider
// stale; the train updates it, tests it, and merges it, with no one waiting.
export const TRAIN_LABEL = 'merge-train';
export const REQUIRED_CHECKS = ['acceptance / ubuntu-24.04', 'acceptance / ubuntu-24.04-arm'];
const PASSING = ['success', 'neutral', 'skipped'];

/// The riders in order, and the labelled requests that wait off the train:
/// drafts, and stacked requests until they are retargeted.
export function trainQueue(prs, {base}) {
  const riders = [], skipped = [];
  for (const pr of prs) {
    if (!pr.labels.includes(TRAIN_LABEL)) continue;
    if (pr.draft) skipped.push({number: pr.number, why: 'it is a draft'});
    else if (pr.base !== base) skipped.push({number: pr.number, why: `it is stacked on ${pr.base}; retarget it to ${base}`});
    else riders.push(pr);
  }
  const at = pr => { const t = Date.parse(pr.labeledAt ?? ''); return Number.isNaN(t) ? Infinity : t; };
  riders.sort((a, b) => at(a) - at(b) || a.number - b.number);
  return {riders, skipped};
}

/// The next move for the head of the train, from how many commits of the
/// default branch its head lacks, the bootstrap runs on its head commit, and
/// the newest run of each required check there:
///   update    merge the default branch into it, then test the new head
///   dispatch  no bootstrap run has tested the head: start one
///   wait      a bootstrap run on the head is still going
///   merge     it contains the default branch and every required check passed
///   drop      it cannot merge as it is: take the label off and say why
/// A workflow token's push starts no workflow, so the train dispatches every
/// bootstrap run it needs; it dispatches only when none ran on the head, so a
/// run that never reports cannot loop.
export function trainStep({pr, base, repository, behind, runs, checks}) {
  if (pr.headRepo !== repository) return {action: 'drop', why: `its branch is in ${pr.headRepo ?? 'a deleted repository'}; the train updates and tests only branches of ${repository}`};
  if (behind > 0) {
    if (pr.mergeable === false) return {action: 'drop', why: `it conflicts with ${base}; merge ${base} into it by hand`};
    return {action: 'update', why: `it is ${behind} commit(s) behind ${base}`};
  }
  if (runs.some(r => r.status !== 'completed')) return {action: 'wait', why: `bootstrap is still running on ${pr.headSha}`};
  const failed = REQUIRED_CHECKS.filter(n => checks[n]?.status === 'completed' && !PASSING.includes(checks[n].conclusion));
  if (failed.length) return {action: 'drop', why: `${failed.map(n => `${n} is ${checks[n].conclusion}`).join(', ')} on ${pr.headSha}`};
  if (REQUIRED_CHECKS.every(n => checks[n]?.status === 'completed')) return {action: 'merge', why: `it contains ${base} and every required check passed on ${pr.headSha}`};
  if (REQUIRED_CHECKS.some(n => checks[n])) return {action: 'wait', why: `the required checks are still running on ${pr.headSha}`};
  if (!runs.length) return {action: 'dispatch', why: `no bootstrap run has tested ${pr.headSha}`};
  return {action: 'drop', why: `bootstrap ran on ${pr.headSha} without reporting ${REQUIRED_CHECKS.join(' and ')}`};
}

function exportRoute(graph, ids, dir) {
  const o = graph.obligations.find(x => x.name === 'export');
  const file = path.join(dir, o.record);
  const recorded = fs.existsSync(file) ? fs.readFileSync(file, 'utf8').trim() : null;
  return {route: recorded === `export ${ids.export}` ? 'proved' : 'observe', identity: ids.export, recorded, file};
}

// The graph names each obligation's job; a renamed job or a changed leg recipe
// must change the graph too, or the route refuses to decide.
function checkWorkflows(graph, dir) {
  for (const o of graph.obligations) {
    if (!o.workflow) continue;
    const text = fs.readFileSync(path.join(dir, '.github/workflows', o.workflow), 'utf8');
    if (!text.includes(`\n  ${o.job}:\n`)) fail(`${o.name}: job ${o.job} is not in ${o.workflow}`);
  }
}

function options(argv) {
  const out = {names: [], contexts: {}, rev: 'HEAD'};
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === '--context') { const [k, ...v] = argv[++i].split('='); if (!k || !v.length) fail('--context needs KEY=VALUE'); out.contexts[k] = v.join('='); }
    else if (a === '--rev') out.rev = argv[++i];
    else if (a === '--expect-run') out.expectRun = argv[++i];
    else if (a.startsWith('--')) fail(`unknown option ${a}`);
    else out.names.push(a);
  }
  return out;
}

const outputKey = name => name.replace(/[^A-Za-z0-9]/g, '_');
const server = () => `${process.env.GITHUB_SERVER_URL ?? 'https://github.com'}/${process.env.GITHUB_REPOSITORY ?? 'UOR-Foundation/compression'}`;

function main(argv) {
  const [command, ...rest] = argv;
  const dir = root();
  const opts = options(rest);
  const graph = loadGraph(dir);
  const byName = name => graph.obligations.find(o => o.name === name) ?? fail(`unknown obligation ${name}`);
  const plannedContexts = {mode: 'rehearsal', run: 'local', ...opts.contexts};
  switch (command) {
    case 'identity': {
      const ids = identities(graph, tree(opts.rev, dir), plannedContexts);
      for (const n of opts.names) console.log(ids[byName(n).name]);
      return;
    }
    case 'plan': {
      const ids = identities(graph, tree(opts.rev, dir), plannedContexts);
      const store = parseStore(fetchStore(dir));
      const routes = route(graph, ids, store, {inRun: false});
      if (opts.rev === 'HEAD') routes.export = exportRoute(graph, ids, dir);
      for (const o of graph.obligations) {
        const r = routes[o.name];
        console.log(`obligation ${o.name} ${r.route} ${r.identity}${r.run ? ` run ${server()}/actions/runs/${r.run}` : ''}`);
      }
      for (const s of schedule(graph, routes, store.weights)) console.log(`schedule ${s.name} ${s.level}`);
      console.log(`releasable ${routes.gate.route === 'proved' ? 'yes' : 'no'}`);
      return;
    }
    case 'route': {
      // GitHub Actions: write each named obligation's route to $GITHUB_OUTPUT.
      checkWorkflows(graph, dir);
      if (!opts.names.length) fail('route needs at least one obligation');
      if (opts.expectRun !== undefined && (opts.names.length !== 1 || byName(opts.names[0]).run !== opts.expectRun))
        fail(`the workflow runs "${opts.expectRun}" but factory/graph.json declares "${byName(opts.names[0]).run}" for ${opts.names[0]}`);
      const ids = identities(graph, tree('HEAD', dir), opts.contexts);
      const routes = route(graph, ids, parseStore(fetchStore(dir)), {attempt: Number(process.env.GITHUB_RUN_ATTEMPT ?? 1)});
      const lines = [];
      for (const n of opts.names) {
        const r = routes[byName(n).name];
        if (r.identity === null) fail(`${n} reads a context; pass it with --context`);
        const key = opts.names.length === 1 ? '' : `${outputKey(n)}_`;
        lines.push(`${key}route=${r.route}`, `${key}identity=${r.identity}`, `${key}run=${r.run ?? process.env.GITHUB_RUN_ID ?? ''}`);
        const where = r.run ? ` in ${server()}/actions/runs/${r.run}` : '';
        if (r.route === 'proved') console.error(`::notice title=${n} proved by receipt::identity ${r.identity} passed${where}; its inputs are unchanged, so it is not run again.`);
        if (r.route === 'refuted') console.error(`::error title=${n} refuted::identity ${r.identity} failed${where} on exactly these inputs. Change an input, or re-run the job to observe it again.`);
        if (r.route === 'blocked') console.error(`::error title=${n} blocked::${byName(n).needs.filter(x => routes[x].route !== 'proved' && routes[x].route !== 'observe').join(', ') || byName(n).needs.join(', ')} not proved for this tree.`);
      }
      process.stdout.write(lines.join('\n') + '\n');
      return;
    }
    case 'outcome': {
      // A job's own record of what it established, for factory.yml to publish.
      const [name, identity, result, out] = opts.names;
      byName(name);
      if (!/^[0-9a-f]{64}$/.test(identity ?? '')) fail('outcome needs the identity the route printed');
      if (!['success', 'failure'].includes(result)) fail('outcome result is success or failure');
      fs.mkdirSync(out, {recursive: true});
      fs.writeFileSync(path.join(out, `${name}.outcome`), `outcome ${name} ${identity} ${result === 'success' ? 'pass' : 'fail'} ${git(['rev-parse', 'HEAD']).toString().trim()}\n`);
      return;
    }
    case 'materialize': {
      // A closure copy in place (PrismPM runs each obligation in a copy of its
      // closure): delete every tracked file the obligation does not declare,
      // so an undeclared input fails the run instead of proving a false receipt.
      const o = byName(opts.names[0]);
      if (o.inputs.includes('') && !o.inputs.some(i => i.startsWith('!'))) { console.log(`${o.name} reads the whole tree`); return; }
      const entries = tree('HEAD', dir), exact = exactPaths(o, entries);
      let removed = 0, kept = 0;
      for (const e of entries) {
        if (covered(o.inputs, e.path, exact)) { kept++; continue; }
        if (e.path.startsWith('factory/') || e.path.startsWith('scripts/factory.')) continue; // the host itself, for the outcome step
        fs.rmSync(path.join(dir, e.path), {force: true}); removed++;
      }
      const stubs = [];
      for (const i of o.inputs.filter(x => x.startsWith('manifest:'))) {
        const manifest = manifestPath(i), crate = path.dirname(manifest);
        const text = fs.readFileSync(path.join(dir, manifest), 'utf8');
        const targets = [...text.matchAll(/^path\s*=\s*"([^"]+)"/gm)].map(m => m[1]);
        if (!targets.length) targets.push('src/lib.rs');
        for (const t of targets) {
          const file = path.join(dir, crate, t);
          if (fs.existsSync(file)) continue;
          fs.mkdirSync(path.dirname(file), {recursive: true}); fs.writeFileSync(file, '');
          stubs.push(path.join(crate, t));
        }
      }
      console.log(`${o.name}: kept ${kept} declared files, removed ${removed}, stubbed ${stubs.length} target(s) of manifest-only crates`);
      return;
    }
    case 'publish': return publish(graph, dir, opts.names[0]);
    case 'release-step': {
      // factory.yml, on the default branch's head: tag, rehearse, wait or done.
      checkWorkflows(graph, dir);
      const version = fs.readFileSync(path.join(dir, 'Cargo.toml'), 'utf8').match(/^\[workspace\.package\][^[]*?^version = "([^"]+)"/m)?.[1] ?? fail('Cargo.toml: no [workspace.package] version');
      const tagged = git(['rev-parse', '--verify', '--quiet', `refs/tags/v${version}`], {cwd: dir, allowFail: true}) !== null;
      const routes = route(graph, identities(graph, tree('HEAD', dir), {}), parseStore(fetchStore(dir)), {inRun: false});
      const r = releaseStep({version, tagged, routes, trigger: opts.contexts.trigger});
      console.error(`::notice title=release ${r.tag}: ${r.step}::${r.why}`);
      process.stdout.write(`step=${r.step}\ntag=${r.tag}\nsha=${git(['rev-parse', 'HEAD'], {cwd: dir}).toString().trim()}\n`);
      return;
    }
    case 'merge-train':
      return mergeTrain({
        repository: process.env.GITHUB_REPOSITORY ?? fail('GITHUB_REPOSITORY is required'),
        base: process.env.DEFAULT_BRANCH ?? fail('DEFAULT_BRANCH is required'),
        token: process.env.GH_TOKEN ?? fail('GH_TOKEN is required'),
        run: process.env.GITHUB_RUN_ID ?? '',
      });
    case 'export-check': {
      const ids = identities(graph, tree('HEAD', dir), plannedContexts);
      const r = exportRoute(graph, ids, dir);
      if (r.route === 'proved') console.log(`export proved: generated/rust was exported from these inputs (${r.identity})`);
      else console.log(`export observe: the client closure changed since generated/rust was exported (${r.recorded ?? 'no record'} -> export ${r.identity}); run node scripts/client-export.mjs`);
      process.exitCode = r.route === 'proved' ? 0 : 3;
      return;
    }
    case 'export-record': {
      // Identity of the working tree's export inputs (client-export.mjs runs on
      // the working tree, so it records what it actually read).
      const ids = identities(graph, workingTree(dir), plannedContexts);
      const o = byName('export');
      fs.writeFileSync(path.join(dir, o.record), `export ${ids.export}\n`);
      console.log(`recorded export ${ids.export}`);
      return;
    }
    default:
      process.stderr.write(fs.readFileSync(fileURLToPath(import.meta.url), 'utf8').split('\n').filter(l => l.startsWith('//   node')).join('\n') + '\n');
      process.exitCode = 64;
  }
}

/// The working tree as a tree listing: tracked and untracked (not ignored) files
/// with their blob ids as `git add` would write them.
export function workingTree(dir) {
  const files = git(['ls-files', '-z', '--cached', '--others', '--exclude-standard'], {cwd: dir}).toString().split('\0').filter(Boolean)
    .filter(p => fs.existsSync(path.join(dir, p)));
  const unique = [...new Set(files)].sort((a, b) => Buffer.compare(Buffer.from(a), Buffer.from(b)));
  const objects = git(['hash-object', '-w', '--stdin-paths'], {cwd: dir, input: unique.join('\n') + '\n'}).toString().trim().split('\n');
  return unique.map((p, i) => {
    const st = fs.lstatSync(path.join(dir, p));
    const mode = st.isSymbolicLink() ? '120000' : (st.mode & 0o111 ? '100755' : '100644');
    return {path: p, object: `${mode}:${objects[i]}`};
  });
}

// factory.yml: turn a completed run's outcome files into receipts on the
// receipts branch, reweigh the schedule, and push; another run may push first,
// so rebuild on the newest head and retry.
export async function publish(graph, dir, outcomes) {
  const run = process.env.FACTORY_RUN_ID ?? fail('FACTORY_RUN_ID is required');
  const attempt = process.env.FACTORY_RUN_ATTEMPT ?? fail('FACTORY_RUN_ATTEMPT is required');
  const token = process.env.GH_TOKEN ?? fail('GH_TOKEN is required');
  const repo = process.env.GITHUB_REPOSITORY ?? fail('GITHUB_REPOSITORY is required');
  const files = fs.existsSync(outcomes) ? fs.readdirSync(outcomes).filter(f => f.endsWith('.outcome')).sort() : [];
  const fresh = files.flatMap(f => {
    const [tag, name, identity, result, commit] = fs.readFileSync(path.join(outcomes, f), 'utf8').trim().split(' ');
    if (tag !== 'outcome' || !/^[a-z0-9._-]+$/.test(name ?? '') || !/^[0-9a-f]{64}$/.test(identity ?? '') || !['pass', 'fail'].includes(result) || !/^[0-9a-f]{40}$/.test(commit ?? '')) fail(`${f}: malformed outcome`);
    // A pull request may declare an obligation the default branch does not know yet.
    if (!graph.obligations.some(o => o.name === name)) { console.error(`::warning::${name} is not in the default branch's factory/graph.json; no receipt`); return []; }
    return [{name, identity, result, commit}];
  });
  if (!fresh.length) { console.log('factory: this run established nothing new'); return; }
  // A receipt whose obligation carries evidence expires a day before its evidence does.
  const listing = await (globalThis.factoryFetch ?? fetch)(`https://api.github.com/repos/${repo}/actions/runs/${run}/artifacts?per_page=100`, {headers: {authorization: `Bearer ${token}`, accept: 'application/vnd.github+json'}});
  const artifacts = listing.ok ? (await listing.json()).artifacts : fail(`listing the artifacts of run ${run}: ${listing.status}`);
  const expiry = name => {
    const o = graph.obligations.find(x => x.name === name);
    if (!o.evidence) return 'never';
    const a = artifacts.find(x => x.name === o.evidence && !x.expired);
    return a ? new Date(Date.parse(a.expires_at) - 86400000).toISOString() : null;
  };
  const header = `AUTHORIZATION: basic ${Buffer.from(`x-access-token:${token}`).toString('base64')}`;
  const auth = ['-c', `http.https://github.com/.extraheader=${header}`];
  const work = fs.mkdtempSync(path.join(os.tmpdir(), 'factory-receipts-'));
  for (let tries = 1; tries <= 5; tries++) {
    const index = path.join(work, `index-${tries}`); const env = {GIT_INDEX_FILE: index};
    let parent = [];
    if (git([...auth, 'fetch', '--quiet', '--depth=1', 'origin', `+refs/heads/${BRANCH}:refs/factory/receipts`], {cwd: dir, allowFail: true}) !== null) {
      git(['read-tree', 'refs/factory/receipts'], {cwd: dir, env});
      parent = ['-p', git(['rev-parse', 'refs/factory/receipts'], {cwd: dir}).toString().trim()];
    }
    const old = parentIndex(dir, env);
    const store = parseStore(old);
    const lines = [];
    for (const o of fresh) {
      const expires = o.result === 'pass' ? expiry(o.name) : 'never';
      if (expires === null) { console.error(`::warning::${o.name} passed but its evidence ${graph.obligations.find(x => x.name === o.name).evidence} is not in run ${run}; no receipt`); continue; }
      const line = `receipt ${o.identity} ${o.name} ${o.result} ${run} ${attempt} ${o.commit} ${expires}`;
      if (store.receipts.some(r => r.identity === o.identity && r.name === o.name && r.run === run && r.attempt === attempt)) continue;
      lines.push(line);
      const blob = git(['hash-object', '-w', '--stdin'], {cwd: dir, input: line + '\n'}).toString().trim();
      git(['update-index', '--add', '--cacheinfo', `100644,${blob},receipts/${o.name}/${o.identity}-${run}-${attempt}.receipt`], {cwd: dir, env});
    }
    const weights = reweigh(store.weights, fresh);
    const now = new Date();
    const kept = store.receipts.filter(r => r.expires === 'never' || new Date(r.expires) > now)
      .map(r => `receipt ${r.identity} ${r.name} ${r.result} ${r.run} ${r.attempt} ${r.commit} ${r.expires}`);
    const text = [...kept, ...lines, ...Object.keys(weights).sort().map(n => `weight ${n} ${weights[n]}`)].join('\n') + '\n';
    const blob = git(['hash-object', '-w', '--stdin'], {cwd: dir, input: text}).toString().trim();
    git(['update-index', '--add', '--cacheinfo', `100644,${blob},index`], {cwd: dir, env});
    const treeId = git(['write-tree'], {cwd: dir, env}).toString().trim();
    const commit = git(['-c', 'user.name=uorc-factory', '-c', 'user.email=uorc-factory@users.noreply.github.com', 'commit-tree', treeId, ...parent, '-m', `Receipts of run ${run} attempt ${attempt}`], {cwd: dir}).toString().trim();
    if (git([...auth, 'push', '--quiet', 'origin', `${commit}:refs/heads/${BRANCH}`], {cwd: dir, allowFail: true}) !== null) {
      console.log(`factory: recorded ${lines.length} receipt(s) of run ${run}:\n${lines.join('\n')}`);
      return;
    }
    await new Promise(r => setTimeout(r, tries * 5000));
  }
  fail(`could not push receipts to ${BRANCH}`);
}

/// Moves the merge train as far as it can without waiting: a merge or a drop
/// goes on to the next rider; an update, a dispatch or a wait ends the run, and
/// the bootstrap run it waits for moves the train again when it completes.
export async function mergeTrain({repository, base, token, run = '', sleep = ms => new Promise(r => setTimeout(r, ms)), log = console.log}) {
  const call = async (method, route, body) => {
    const response = await (globalThis.factoryFetch ?? fetch)(`https://api.github.com/repos/${repository}/${route}`, {method, headers: {authorization: `Bearer ${token}`, accept: 'application/vnd.github+json', 'x-github-api-version': '2022-11-28'}, ...(body === undefined ? {} : {body: JSON.stringify(body)})});
    const text = await response.text();
    let data = null; try { data = text ? JSON.parse(text) : null; } catch { data = {message: text}; }
    return {status: response.status, ok: response.ok, data};
  };
  const get = async route => { const r = await call('GET', route); return r.ok ? r.data : fail(`GET ${route}: ${r.status} ${r.data?.message ?? ''}`); };
  const all = async route => {
    const out = [];
    for (let page = 1; page <= 10; page++) {
      const items = await get(`${route}${route.includes('?') ? '&' : '?'}per_page=100&page=${page}`);
      out.push(...items); if (items.length < 100) break;
    }
    return out;
  };
  const dispatch = async ref => {
    const r = await call('POST', 'actions/workflows/bootstrap.yml/dispatches', {ref});
    if (!r.ok) fail(`dispatching bootstrap on ${ref}: ${r.status} ${r.data?.message ?? ''}`);
    log(`merge-train: dispatched bootstrap on ${ref}`);
  };
  const where = run ? ` ([run](https://github.com/${repository}/actions/runs/${run}))` : '';
  const drop = async (pr, why) => {
    log(`merge-train: #${pr.number} leaves the train: ${why}`);
    const unlabeled = await call('DELETE', `issues/${pr.number}/labels/${encodeURIComponent(TRAIN_LABEL)}`);
    if (!unlabeled.ok && unlabeled.status !== 404) fail(`removing ${TRAIN_LABEL} from #${pr.number}: ${unlabeled.status}`);
    const r = await call('POST', `issues/${pr.number}/comments`, {body: `Left the merge train${where}: ${why}. Add the \`${TRAIN_LABEL}\` label again to rejoin at the back.`});
    if (!r.ok) fail(`commenting on #${pr.number}: ${r.status}`);
  };
  // The label is part of the route: the first train run creates it.
  const label = await call('GET', `labels/${encodeURIComponent(TRAIN_LABEL)}`);
  if (label.status === 404) {
    const made = await call('POST', 'labels', {name: TRAIN_LABEL, color: '0e8a16', description: 'Ride the factory merge train: kept current with main, merged in order when green'});
    if (!made.ok && made.status !== 422) fail(`creating the ${TRAIN_LABEL} label: ${made.status} ${made.data?.message ?? ''}`);
    log(`merge-train: created the ${TRAIN_LABEL} label`);
  } else if (!label.ok) fail(`reading the ${TRAIN_LABEL} label: ${label.status}`);
  for (let moves = 0; moves < 50; moves++) {
    const open = (await all('pulls?state=open')).filter(p => p.labels.some(l => l.name === TRAIN_LABEL));
    const prs = [];
    for (const p of open) {
      const labeled = (await all(`issues/${p.number}/events`)).filter(e => e.event === 'labeled' && e.label?.name === TRAIN_LABEL).at(-1);
      prs.push({number: p.number, title: p.title, body: p.body ?? '', draft: p.draft, base: p.base.ref, headRef: p.head.ref, headSha: p.head.sha, headRepo: p.head.repo?.full_name ?? null, labels: p.labels.map(l => l.name), labeledAt: labeled?.created_at});
    }
    const {riders, skipped} = trainQueue(prs, {base});
    for (const s of skipped) log(`merge-train: #${s.number} waits off the train: ${s.why}`);
    if (!riders.length) { log('merge-train: no rider'); return {action: 'idle'}; }
    log(`merge-train: ${riders.map(p => `#${p.number}`).join(' ')}`);
    const head = riders[0];
    const detail = await get(`pulls/${head.number}`);
    const pr = {...head, headSha: detail.head.sha, mergeable: detail.mergeable};
    const behind = (await get(`compare/${encodeURIComponent(base)}...${pr.headSha}`)).behind_by;
    const runs = (await get(`actions/workflows/bootstrap.yml/runs?head_sha=${pr.headSha}&per_page=100`)).workflow_runs;
    const checks = {};
    for (const name of REQUIRED_CHECKS) {
      const found = (await get(`commits/${pr.headSha}/check-runs?check_name=${encodeURIComponent(name)}&per_page=100`)).check_runs;
      if (found.length) checks[name] = found.reduce((a, b) => (b.id > a.id ? b : a));
    }
    const step = trainStep({pr, base, repository, behind, runs, checks});
    log(`merge-train: #${pr.number} ${step.action}: ${step.why}`);
    if (step.action === 'wait') return step;
    if (step.action === 'dispatch') { await dispatch(pr.headRef); return step; }
    if (step.action === 'drop') { await drop(pr, step.why); continue; }
    if (step.action === 'update') {
      const r = await call('PUT', `pulls/${pr.number}/update-branch`, {expected_head_sha: pr.headSha});
      if (!r.ok) {
        const message = r.data?.message ?? `status ${r.status}`;
        if (/expected head sha/i.test(message)) { log(`merge-train: #${pr.number} moved while updating; its push starts its own run`); return {action: 'wait', why: message}; }
        await drop(pr, /conflict/i.test(message) ? `it conflicts with ${base}; merge ${base} into it by hand` : `updating it from ${base} failed: ${message}`);
        continue;
      }
      for (let tries = 0; tries < 24; tries++) {
        await sleep(5000);
        if ((await get(`pulls/${pr.number}`)).head.sha !== pr.headSha) { await dispatch(pr.headRef); return step; }
      }
      log(`merge-train: #${pr.number}'s updated head has not appeared yet; the next train run dispatches its bootstrap`);
      return step;
    }
    // merge: the workflow token's merge starts no push run, and the default
    // branch's receipts and release step follow from that run, so dispatch it.
    const r = await call('PUT', `pulls/${pr.number}/merge`, {merge_method: 'squash', sha: pr.headSha, commit_title: `${pr.title} (#${pr.number})`, commit_message: pr.body});
    if (!r.ok) {
      const message = r.data?.message ?? `status ${r.status}`;
      if (r.status === 409) { log(`merge-train: #${pr.number} moved while merging: ${message}`); return {action: 'wait', why: message}; }
      await drop(pr, `merging it failed: ${message}`);
      continue;
    }
    log(`merge-train: merged #${pr.number} as ${r.data?.sha}`);
    await dispatch(base);
  }
  return {action: 'wait', why: 'move limit reached'};
}

function parentIndex(dir, env) {
  return (git(['cat-file', 'blob', ':index'], {cwd: dir, env, allowFail: true}) ?? '').toString();
}

if (process.argv[1] && fs.realpathSync(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { await main(process.argv.slice(2)); } catch (error) { console.error(`factory: ${error.message}`); process.exitCode = 1; }
}
