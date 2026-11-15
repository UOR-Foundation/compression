// node --test scripts/factory.test.mjs
import test from 'node:test';
import assert from 'node:assert/strict';
import {covered, route, reweigh, schedule, parseStore, releaseStep, RELEASE_STAGES, trainQueue, trainStep, mergeTrain, REQUIRED_CHECKS, TRAIN_LABEL, unrecorded} from './factory.mjs';

const graph = {obligations: [
  {name: 'leg', kind: 'hermetic', needs: [], inputs: []},
  {name: 'net', kind: 'fetch', needs: [], inputs: []},
  {name: 'build', kind: 'fetch', needs: [], inputs: []},
  {name: 'bench', kind: 'fetch', needs: ['build'], inputs: []},
  {name: 'gate', kind: 'gate', needs: ['leg', 'net'], inputs: []},
]};
const ids = {leg: 'a', net: 'b', build: 'c', bench: 'd', gate: 'e'};
const receipt = (identity, name, result, run, expires = 'never') => `receipt ${identity} ${name} ${result} ${run} 1 ${'0'.repeat(40)} ${expires}`;

test('coverage: the longest inclusion must beat the longest exclusion', () => {
  const inputs = ['', '!client/', 'client/src/Keep.lex.tex', '@mode'];
  assert.equal(covered(inputs, 'src/A.lex.tex', new Set()), true);
  assert.equal(covered(inputs, 'client/gen/x.py', new Set()), false);
  assert.equal(covered(inputs, 'client/src/Keep.lex.tex', new Set()), true);
  assert.equal(covered(['crates/uorc/'], 'Cargo.toml', new Set()), false);
  assert.equal(covered(['!x/'], 'x/y', new Set(['x/y'])), true);
});

test('a pass proves, a hermetic failure refutes, a fetch failure is observed again', () => {
  const store = parseStore([receipt('a', 'leg', 'fail', 1), receipt('b', 'net', 'fail', 2), receipt('c', 'build', 'pass', 3)].join('\n'));
  const r = route(graph, ids, store);
  assert.equal(r.leg.route, 'refuted');
  assert.equal(r.leg.run, '1');
  assert.equal(r.net.route, 'observe');
  assert.equal(r.build.route, 'proved');
  assert.equal(r.build.run, '3');
  assert.equal(r.bench.route, 'observe');
  assert.equal(r.gate.route, 'blocked');
});

test('a manual re-run observes a refuted obligation; a later pass outweighs a failure', () => {
  const failed = parseStore(receipt('a', 'leg', 'fail', 1));
  assert.equal(route(graph, ids, failed, {attempt: 2}).leg.route, 'observe');
  const both = parseStore([receipt('a', 'leg', 'fail', 1), receipt('a', 'leg', 'pass', 4), receipt('b', 'net', 'pass', 5)].join('\n'));
  const r = route(graph, ids, both);
  assert.equal(r.leg.route, 'proved');
  assert.equal(r.gate.route, 'proved');
});

test('a receipt for other inputs or past its evidence proves nothing', () => {
  const store = parseStore([receipt('zz', 'build', 'pass', 1), receipt('c', 'build', 'pass', 2, '2000-01-01T00:00:00Z')].join('\n'));
  assert.equal(route(graph, ids, store).build.route, 'observe');
});

test('outside a run, a need that is not yet proved blocks', () => {
  assert.equal(route(graph, ids, parseStore(''), {inRun: false}).bench.route, 'blocked');
  assert.equal(route(graph, ids, parseStore(''), {inRun: true}).bench.route, 'observe');
});

test('weights stay in 1..16: a failure raises priority, a pass lowers it, and schedule orders by it', () => {
  let w = {};
  for (let i = 0; i < 20; i++) w = reweigh(w, [{name: 'leg', result: 'fail'}, {name: 'net', result: 'pass'}]);
  assert.deepEqual(w, {leg: 16, net: 1});
  const r = route(graph, ids, parseStore(''));
  assert.deepEqual(schedule(graph, r, w).map(s => s.name), ['leg', 'build', 'bench', 'net']);
});

test('a manifest input covers only that manifest, even under an exclusion', () => {
  const inputs = ['', '!crates/uorc/', 'manifest:crates/uorc/Cargo.toml'];
  assert.equal(covered(inputs, 'crates/uorc/Cargo.toml', new Set()), true);
  assert.equal(covered(inputs, 'crates/uorc/src/lib.rs', new Set()), false);
  assert.equal(covered(inputs, 'crates/model/src/lib.rs', new Set()), true);
});

test('a version bump on the default branch is tagged once its gate and release stages are proved', () => {
  const routes = (gate, stage) => Object.fromEntries([['gate', {route: gate}], ...RELEASE_STAGES.map(n => [n, {route: stage}])]);
  const step = (tagged, gate, stage, trigger = 'bootstrap') => releaseStep({version: '0.5.0', tagged, routes: routes(gate, stage), trigger}).step;
  assert.equal(step(true, 'proved', 'proved'), 'done');
  assert.equal(step(false, 'blocked', 'proved'), 'wait');
  assert.equal(step(false, 'proved', 'observe'), 'rehearse');
  assert.equal(step(false, 'proved', 'observe', 'release'), 'wait');
  assert.equal(step(false, 'proved', 'proved', 'release'), 'tag');
  assert.equal(releaseStep({version: '0.5.0', tagged: false, routes: routes('proved', 'proved')}).tag, 'v0.5.0');
});

const rider = (number, extra = {}) => ({number, title: `PR ${number}`, body: '', draft: false, base: 'main', headRef: `b${number}`, headSha: `s${number}`, headRepo: 'o/r', labels: [TRAIN_LABEL], ...extra});

test('the merge train takes labelled, non-draft requests into the default branch in label order', () => {
  const {riders, skipped} = trainQueue([
    rider(5, {labeledAt: '2026-11-02T00:00:00Z'}),
    rider(3, {labeledAt: '2026-11-03T00:00:00Z'}),
    rider(4, {labeledAt: '2026-11-01T00:00:00Z'}),
    rider(9), rider(8),
    rider(6, {draft: true}),
    rider(7, {base: 'feature'}),
    rider(2, {labels: []}),
  ], {base: 'main'});
  assert.deepEqual(riders.map(p => p.number), [4, 5, 3, 8, 9]);
  assert.deepEqual(skipped.map(s => s.number), [6, 7]);
  assert.match(skipped[1].why, /stacked on feature; retarget it to main/);
});

test('the head of the train waits for its branch to take in main, then merges when green; a failure, conflict or fork drops it', () => {
  const pass = Object.fromEntries(REQUIRED_CHECKS.map(n => [n, {status: 'completed', conclusion: 'success'}]));
  const done = [{status: 'completed'}];
  const step = state => trainStep({pr: rider(1), base: 'main', repository: 'o/r', behind: 0, runs: done, checks: pass, ...state}).action;
  assert.equal(step({}), 'merge');
  assert.equal(step({behind: 2}), 'behind');
  assert.equal(step({behind: 2, pr: rider(1, {mergeable: false})}), 'drop');
  assert.equal(step({runs: [...done, {status: 'in_progress'}]}), 'wait');
  assert.equal(step({checks: {[REQUIRED_CHECKS[0]]: {status: 'in_progress'}}}), 'wait');
  assert.equal(step({checks: {[REQUIRED_CHECKS[0]]: pass[REQUIRED_CHECKS[0]]}}), 'wait');
  assert.equal(step({checks: {...pass, [REQUIRED_CHECKS[1]]: {status: 'completed', conclusion: 'failure'}}}), 'drop');
  assert.equal(step({checks: {...pass, [REQUIRED_CHECKS[1]]: {status: 'completed', conclusion: 'cancelled'}}}), 'drop');
  assert.equal(step({checks: {}, runs: []}), 'wait');
  assert.equal(step({checks: {}, runs: done}), 'drop');
  assert.equal(step({pr: rider(1, {headRepo: 'fork/r'})}), 'drop');
  assert.equal(step({pr: rider(1, {headRepo: null}), behind: 3}), 'drop');
});

// A fake GitHub API holding two riders, labelled in number order.
function fakeGitHub({conclusion = 'success', behind = {b1: 1}, runs = () => [{status: 'completed'}], label = true} = {}) {
  const calls = [];
  const pull = (n, ref, sha, body) => ({number: n, title: `PR ${n}`, body, draft: false, state: 'open', base: {ref: 'main'}, head: {ref, sha, repo: {full_name: 'o/r'}}, labels: [{name: TRAIN_LABEL}], mergeable: true});
  const pulls = {1: pull(1, 'one', 'a1', 'one'), 2: pull(2, 'two', 'b1', null)};
  const comments = {};
  const reply = (status, data) => ({status, ok: status < 300, text: async () => (data === undefined ? '' : JSON.stringify(data))});
  globalThis.factoryFetch = async (url, {method, body}) => {
    const [path, query = ''] = url.replace('https://api.github.com/repos/o/r/', '').split('?');
    calls.push(`${method} ${path}${body ? ' ' + body : ''}`);
    const sha = path.split('/')[1];
    if (method === 'GET' && path === `labels/${TRAIN_LABEL}`) return label ? reply(200, {name: TRAIN_LABEL}) : reply(404, {message: 'Not Found'});
    if (method === 'POST' && path === 'labels') { label = true; return reply(201, {name: TRAIN_LABEL}); }
    if (method === 'GET' && path === 'pulls') return reply(200, Object.values(pulls).filter(p => p.state === 'open'));
    if (method === 'GET' && /^issues\/\d+\/events$/.test(path)) return reply(200, [{event: 'labeled', label: {name: TRAIN_LABEL}, created_at: `2026-11-0${path.split('/')[1]}T00:00:00Z`}]);
    if (method === 'GET' && /^pulls\/\d+$/.test(path)) return reply(200, pulls[path.split('/')[1]]);
    if (method === 'GET' && path.startsWith('compare/main...')) return reply(200, {behind_by: behind[path.split('...')[1]] ?? 0});
    if (method === 'GET' && path === 'actions/workflows/bootstrap.yml/runs') return reply(200, {workflow_runs: runs(query.match(/head_sha=(\w+)/)[1])});
    if (method === 'GET' && path.endsWith('/check-runs')) return reply(200, {check_runs: runs(sha).length ? [{id: 1, status: 'completed', conclusion: sha === 'a1' ? conclusion : 'success'}] : []});
    if (method === 'PUT' && path === 'pulls/1/merge') { pulls[1].state = 'closed'; return reply(200, {sha: 'm1', merged: true}); }
    if (method === 'GET' && /^issues\/\d+\/comments$/.test(path)) return reply(200, comments[path.split('/')[1]] ?? []);
    if (method === 'POST' && path === 'issues/2/comments') { (comments[2] ??= []).push(JSON.parse(body)); return reply(201, {}); }
    if (method === 'DELETE' && path === `issues/1/labels/${TRAIN_LABEL}`) { pulls[1].labels = []; return reply(200, []); }
    if (method === 'POST' && path === 'issues/1/comments') return reply(201, {});
    if (method === 'POST' && path === 'actions/workflows/bootstrap.yml/dispatches') return reply(204);
    return reply(404, {message: 'Not Found'});
  };
  return calls;
}

async function ride(options) {
  const calls = fakeGitHub(options);
  try { return {result: await mergeTrain({repository: 'o/r', base: 'main', token: 't', sleep: async () => {}, log: () => {}}), writes: calls.filter(c => !c.startsWith('GET'))}; }
  finally { delete globalThis.factoryFetch; }
}

test('a merge train run merges a green head, dispatches main, then asks the next rider once to take in main', async () => {
  const {result, writes} = await ride({behind: {b1: 1}});
  assert.equal(result.action, 'behind');
  assert.equal(writes.length, 3);
  assert.equal(writes[0], `PUT pulls/1/merge ${JSON.stringify({merge_method: 'squash', sha: 'a1', commit_title: 'PR 1 (#1)', commit_message: 'one'})}`);
  assert.equal(writes[1], 'POST actions/workflows/bootstrap.yml/dispatches {"ref":"main"}');
  assert.match(writes[2], /^POST issues\/2\/comments .*merge-train behind b1.*merge main into its branch and push/);
  assert.ok(!writes.some(w => w.includes('update-branch')), 'the train never pushes to a rider');
});

test('a merge train run drops a failed head with one comment and tests the next rider', async () => {
  const {result, writes} = await ride({conclusion: 'failure', behind: {}, runs: sha => (sha === 'a1' ? [{status: 'completed'}] : [])});
  assert.equal(result.action, 'wait');
  assert.equal(writes.length, 2);
  assert.equal(writes[0], `DELETE issues/1/labels/${TRAIN_LABEL}`);
  assert.match(writes[1], /^POST issues\/1\/comments .*Left the merge train: acceptance \/ ubuntu-24.04 is failure/);
});

test('a merge train run creates its label when the repository has none', async () => {
  const {writes} = await ride({label: false});
  assert.match(writes[0], new RegExp(`^POST labels .*"name":"${TRAIN_LABEL}"`));
  assert.equal(writes.filter(w => w.startsWith('POST labels')).length, 1);
});

test('dispatched runs the store has not seen are collected, oldest first, from this repository only', async () => {
  const g = {obligations: [{name: 'leg', workflow: 'bootstrap.yml'}, {name: 'build', workflow: 'release.yml'}, {name: 'gate'}]};
  const store = parseStore(`${receipt('a', 'leg', 'pass', 7)}\nseen 7 1`);
  assert.deepEqual([...store.seen], ['7 1']);
  const asked = [];
  const runs = {
    'bootstrap.yml': [{id: 7, run_attempt: 1, updated_at: '2026-01-01T00:00:00Z', head_repository: {full_name: 'o/r'}},
                      {id: 9, run_attempt: 1, updated_at: '2026-01-03T00:00:00Z', head_repository: {full_name: 'o/r'}},
                      {id: 10, run_attempt: 1, updated_at: '2026-01-04T00:00:00Z', head_repository: {full_name: 'fork/r'}}],
    'release.yml': [{id: 7, run_attempt: 2, updated_at: '2026-01-02T00:00:00Z', head_repository: {full_name: 'o/r'}}],
  };
  const call = async url => { asked.push(url); const w = url.match(/workflows\/([^/]+)\/runs/)[1]; return {ok: true, json: async () => ({workflow_runs: runs[w]})}; };
  assert.deepEqual(await unrecorded(g, store, {repository: 'o/r', token: 't', call}), ['7 2', '9 1']);
  assert.equal(asked.length, 2);
  assert.ok(asked.every(u => u.includes('event=workflow_dispatch') && u.includes('status=completed') && u.includes('actor=github-actions%5Bbot%5D')));
});
