// The export's literal-first projection and literal-second check, on data-form
// fragments shaped as `xtask data-form` writes them.
import assert from 'node:assert/strict';
import {test} from 'node:test';
import {literalFirst, literalSecondHazards, SECOND_LITERAL_BOUND} from './client-export.mjs';

const nat = value => ({kind: 'nat', value: String(value)});
const v = name => ({kind: 'var', name});
const prim = (operation, ...args) => ({arguments: args, kind: 'primitive', operation, result: {kind: 'nat'}});
const add = (left, right) => ({kind: 'add', left, right});
const definition = (name, body) => ({axioms: [], body, kind: 'definition', name, parameters: [{name: 'mb', type: {kind: 'nat'}}], result: {kind: 'nat'}});
const module = (...declarations) => ({declarations});
const clone = x => structuredClone(x);

test('addition and multiplication with a literal second operand are written literal first', () => {
  const data = new Map([['Repro.Main', module(
    definition('f', prim('multiply', v('mb'), nat(67108864))),
    definition('g', {arguments: [add(v('mb'), nat(1))], function: {name: 'f'}, kind: 'call'}),
  )]]);
  const {touched, swaps} = literalFirst(data);
  assert.equal(swaps, 2);
  assert.deepEqual([...touched], ['Repro.Main']);
  const [f, g] = data.get('Repro.Main').declarations;
  assert.deepEqual(f.body, prim('multiply', nat(67108864), v('mb')));
  assert.deepEqual(g.body.arguments[0], add(nat(1), v('mb')));
});

test('the projection leaves everything else as it was', () => {
  const untouched = module(
    definition('literalFirst', add(nat(1), prim('multiply', nat(4), v('mb')))),
    definition('bothLiteral', add(nat(2), nat(3))),
    definition('neitherLiteral', prim('multiply', v('mb'), v('mb'))),
    definition('subtract', prim('subtract', v('mb'), nat(1))),
    definition('quotient', prim('quotient', v('mb'), nat(256), nat(0))),
    // A nat type node has no value: it is not a literal.
    definition('typeNotLiteral', add(v('mb'), {kind: 'nat'})),
    {kind: 'theorem', name: 'statement', parameters: [], proof: {kind: 'reflexivity'}, statement: {kind: 'eq', left: add(v('mb'), nat(1)), right: add(v('mb'), nat(1))}},
  );
  const data = new Map([['M', clone(untouched)]]);
  assert.deepEqual(literalFirst(data), {touched: new Set(), swaps: 0});
  assert.deepEqual(data.get('M'), untouched);
});

test('definitions a theorem with parameters can unfold keep their form', () => {
  const call = (name, ...args) => ({arguments: args, function: {name}, kind: 'call'});
  const data = new Map([
    ['Lib', module(definition('step', add(v('mb'), nat(1))))],
    ['M', module(
      definition('g', call('h', v('mb'))),
      definition('h', {arguments: [add(v('mb'), nat(1))], function: {module: 'Lib', name: 'step'}, kind: 'call'}),
      definition('free', add(v('mb'), nat(1))),
      {kind: 'theorem', name: 'g_step', parameters: [{name: 'n', type: {kind: 'nat'}}], proof: {kind: 'reflexivity'}, statement: {kind: 'eq', left: call('g', v('n')), right: add(v('n'), nat(2))}},
      // A closed theorem evaluates; it pins nothing.
      {kind: 'theorem', name: 'free_zero', parameters: [], proof: {kind: 'reflexivity'}, statement: {kind: 'eq', left: call('free', nat(0)), right: nat(1)}},
    )],
  ]);
  assert.equal(literalFirst(data).swaps, 1);
  assert.deepEqual(data.get('Lib').declarations[0].body, add(v('mb'), nat(1)));
  assert.deepEqual(data.get('M').declarations[1].body.arguments[0], add(v('mb'), nat(1)));
  assert.deepEqual(data.get('M').declarations[2].body, add(nat(1), v('mb')));
});

test('a subtraction with a literal second operand above the bound is named by module and definition', () => {
  const above = (SECOND_LITERAL_BOUND + 1n).toString();
  const data = new Map([
    ['Uorc.Codec.Planted', module(
      definition('peel', {binder: {name: 'x', type: {kind: 'nat'}}, body: prim('subtract', v('x'), nat(above)), kind: 'let', value: v('mb')}),
      definition('atBound', prim('subtract', v('mb'), nat(SECOND_LITERAL_BOUND))),
      definition('literalFirst', prim('subtract', nat(above), v('mb'))),
      definition('closed', prim('subtract', nat(above), nat(above))),
    )],
  ]);
  assert.deepEqual(literalSecondHazards(data), [{module: 'Uorc.Codec.Planted', definition: 'peel', operation: 'subtract', literal: above}]);
});

test('a certified literal second passes only in its own module and operation', () => {
  const big = (2n ** 46n).toString();
  const data = new Map([
    ['Uorc.Codec.NcmEncode', module(definition('renamedByTheGenerator', prim('subtract', v('mb'), nat(big))))],
    ['Uorc.Codec.NcmDecode', module(definition('elsewhere', prim('subtract', v('mb'), nat(big))))],
  ]);
  assert.deepEqual(literalSecondHazards(data).map(h => h.module), ['Uorc.Codec.NcmDecode']);
  assert.equal(literalSecondHazards(data, SECOND_LITERAL_BOUND, []).length, 2);
});

test('the check runs after the projection, so only what cannot be reordered remains', () => {
  const big = (SECOND_LITERAL_BOUND * 2n).toString();
  const data = new Map([['M', module(definition('f', prim('multiply', v('mb'), nat(big))))]]);
  assert.deepEqual(literalSecondHazards(data).map(h => h.operation), ['multiply']);
  literalFirst(data);
  assert.deepEqual(literalSecondHazards(data), []);
});
