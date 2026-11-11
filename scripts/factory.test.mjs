// node --test scripts/factory.test.mjs
import test from 'node:test';
import assert from 'node:assert/strict';
import {covered, route, reweigh, schedule, parseStore} from './factory.mjs';

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
