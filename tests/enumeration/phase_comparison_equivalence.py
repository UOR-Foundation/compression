#!/usr/bin/env python3
"""Focused exact-source Phase repair diagnostics, never compiler acceptance.

Run without arguments or optimized Python. The frozen source checkpoint and the
current OwnedEnumeration are compared as complete canonical modules. Only three
scalar equality predicates may become exhaustive source-owned Phase matches.
The source evaluator supplies finite diagnostic evidence, not SDK elaboration,
native execution, kernel proof, resource bounds, or controller qualification.
"""
import ast
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
BASELINE = '8f39fcdd0cbe5c4aef057321e53e8fbf304f628f'
MODULE = 'Uorc.Search.OwnedEnumeration'
SOURCE = 'src/Uorc/Search/OwnedEnumeration.lex.tex'
PHASES = (
    'allocateControl', 'offer', 'waiting', 'scanStart', 'scanRead',
    'buildBegin', 'buildNext', 'buildRead', 'appendBegin', 'builderStep',
    'finishBegin', 'release', 'drain', 'install', 'cancelSelect',
    'cancelBuilder', 'cancelBuilderStep', 'cancelled',
)
SITES = (
    ('ticketMatches', ('body', 'left'), 'waiting',
     'retained-compiler-Phase-scalar-equal-rejection'),
    ('cancel', ('body', 'else_value', 'condition'), 'cancelled',
     'static-source-finding-not-an-additional-compiler-result'),
    ('cancel', ('body', 'else_value', 'else_value', 'fields', 0, 'value',
                'condition'), 'drain',
     'static-source-finding-not-an-additional-compiler-result'),
)


def require(condition, message):
    if not condition:
        raise ValueError(message)


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':')).encode()


def sha(value):
    return hashlib.sha256(value).hexdigest()


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, 'duplicate JSON key: ' + key)
        result[key] = value
    return result


def parse(data):
    text = data.decode('utf-8')
    marker = r'\semanticdata{'
    require(text.count(marker) == 1, 'expected exactly one semanticdata form')
    require(text.startswith(r'\begin{lexlean}{' + MODULE + '}\n'),
            'wrong module header')
    start = text.index(marker) + len(marker)
    node, length = json.JSONDecoder(object_pairs_hook=unique_object).raw_decode(text[start:])
    require(text[start:start + length].encode() == canonical(node),
            'noncanonical semantic JSON')
    require(node.get('spec') == 'lexlean/semantic-module/1', 'wrong semantic schema')
    names = [d.get('name') for d in node['declarations']]
    require(all(isinstance(n, str) for n in names) and len(set(names)) == len(names),
            'missing or duplicate declaration')
    return {'raw': data, 'ast': node, 'prefix': text[:start].encode(),
            'suffix': text[start + length:].encode()}


def declarations(module):
    return {d['name']: d for d in module['declarations']}


def at(node, path):
    for key in path:
        node = node[key]
    return node


def replace(module, name, path, value):
    node = declarations(module)[name]
    at(node, path[:-1])[path[-1]] = copy.deepcopy(value)


def phase_project():
    return {'kind': 'project', 'field': 'phase',
            'value': {'kind': 'var', 'name': 'cursor'}}


def primitive(target):
    return {'kind': 'primitive', 'operation': 'equal', 'result': {'kind': 'bool'},
            'arguments': [phase_project(), {'kind': 'constructor',
                          'constructor': {'name': 'Phase.' + target}, 'arguments': []}]}


def predicate(target):
    return {'kind': 'match', 'scrutinee': phase_project(), 'branches': [
        {'constructor': {'name': 'Phase.' + name}, 'binders': [],
         'body': {'kind': 'bool', 'value': name == target}} for name in PHASES]}


def walk(node):
    if isinstance(node, dict):
        yield node
        for child in node.values():
            yield from walk(child)
    elif isinstance(node, list):
        for child in node:
            yield from walk(child)


def check_shapes(module):
    ds = declarations(module)
    require(ds['Phase'] == {'kind': 'inductive', 'name': 'Phase',
            'constructors': [{'name': n, 'fields': []} for n in PHASES],
            'parameters': [], 'type_parameters': []}, 'Phase domain/payload shape changed')
    require(next(f['type'] for f in ds['Cursor']['fields'] if f['name'] == 'phase') ==
            {'kind': 'named', 'member': {'name': 'Phase'}, 'arguments': []},
            'cursor.phase is not the exact source Phase type')
    for name, path, target, _ in SITES:
        node = at(ds[name], path)
        require(node.get('kind') == 'match', name + '/' + target + ': expected Phase match')
        require(set(node) == {'kind', 'scrutinee', 'branches'} and
                node['scrutinee'] == phase_project(), 'wrong Phase match scrutinee')
        branches = node['branches']
        require(len(branches) == len(PHASES) and
                [b.get('constructor') for b in branches] ==
                [{'name': 'Phase.' + n} for n in PHASES],
                'missing, duplicate, reordered or foreign Phase match branch')
        for branch in branches:
            require(set(branch) == {'constructor', 'binders', 'body'} and
                    branch['binders'] == [] and set(branch['body']) == {'kind', 'value'} and
                    branch['body']['kind'] == 'bool' and
                    isinstance(branch['body']['value'], bool),
                    'wrong Phase match branch type or binder')
    bad = [n for n in walk(module) if n.get('kind') == 'primitive' and
           n.get('operation') == 'equal' and any(
               x.get('kind') == 'constructor' and
               x.get('constructor', {}).get('name', '').startswith('Phase.')
               for x in n.get('arguments', []))]
    require(not bad, 'source still uses scalar equal on Phase')


def check_exact(before, after):
    expected = copy.deepcopy(before['ast'])
    old = declarations(expected)
    for name, path, target, _ in SITES:
        require(at(old[name], path) == primitive(target), 'unexpected frozen comparison site')
        replace(expected, name, path, predicate(target))
    require(after['prefix'] == before['prefix'] and after['suffix'] == before['suffix'],
            'non-JSON source bytes changed')
    require(after['ast'] == expected, 'module differs beyond the exact three Phase predicates')
    require(after['raw'] == before['prefix'] + canonical(expected) + before['suffix'],
            'source bytes differ beyond exact Phase predicate substitutions')
    check_shapes(after['ast'])


def evaluate(evaluator, module, name, *arguments):
    d = declarations(module)[name]
    return evaluator.ev(d['body'], MODULE,
                        dict(zip([p['name'] for p in d['parameters']], arguments)))


def check_semantics(evaluator, old, new):
    T = evaluator.tag
    cursor = evaluator.call(MODULE, 'start')
    # Distinct non-null handles and nonzero fields make custody/preservation
    # checks meaningful without manufacturing heap transitions or extra roots.
    cursor.update(control=dict(cursor['control'], id=17),
                  current=dict(cursor['current'], id=23),
                  next=dict(cursor['next'], id=29), length=31, nextLength=37,
                  position=41, pivot=43, index=47, byteValue=53, paid=59)
    predicate_cases = ticket_cases = cancellation_cases = 0
    old_ds, new_ds = declarations(old), declarations(new)
    for phase in PHASES:
        phase_value = T(MODULE + '.Phase.' + phase)
        for name, path, target, _ in SITES:
            value = evaluator.ev(at(new_ds[name], path), MODULE,
                                 {'cursor': dict(cursor, phase=phase_value)})
            previous = evaluator.ev(at(old_ds[name], path), MODULE,
                                    {'cursor': dict(cursor, phase=phase_value)})
            require(value is (phase == target) and value == previous,
                    'Phase predicate semantic mismatch: ' + name + '/' + target + '/' + phase)
            predicate_cases += 1
        for owned in (False, True):
            for cancelling in (False, True):
                for active in (False, True):
                    for held in (False, True):
                        c = dict(cursor, phase=phase_value, ticketOwned=owned,
                                 cancelling=cancelling, builderActive=active, builderHeld=held)
                        got = evaluate(evaluator, new, 'cancel', c)
                        want = evaluate(evaluator, old, 'cancel', c)
                        require(got == want, 'cancel semantic mismatch: ' + phase)
                        preserve = cancelling or phase == 'cancelled'
                        expected_phase = phase if preserve else ('drain' if phase == 'drain'
                                         else 'cancelBuilder' if active else 'cancelSelect')
                        expected = c if preserve else dict(c,
                            phase=T(MODULE + '.Phase.' + expected_phase), cancelling=True, paid=0)
                        require(got == expected, 'cancel ownership/state preservation mismatch')
                        cancellation_cases += 1
            c = dict(cursor, phase=phase_value, ticketOwned=owned)
            ticket = {'type': MODULE + '.Ticket', 'owner': c['control'],
                      'sequence': c['current'], 'length': c['length']}
            tickets = [('exact', ticket)]
            for field in ('owner', 'sequence'):
                for part in ('id',):
                    bad = dict(ticket)
                    bad[field] = dict(ticket[field], **{part: ticket[field][part] + 1})
                    tickets.append((field + '/' + part, bad))
            tickets.append(('length', dict(ticket, length=ticket['length'] + 1)))
            for label, presented in tickets:
                got = evaluate(evaluator, new, 'ticketMatches', c, presented)
                want = evaluate(evaluator, old, 'ticketMatches', c, presented)
                require(got == want and got is (phase == 'waiting' and owned and label == 'exact'),
                        'ticket custody semantic mismatch: ' + phase + '/' + label)
                ticket_cases += 1
    require((predicate_cases, ticket_cases, cancellation_cases) == (54, 144, 288),
            'missing or duplicated exhaustive semantic cases')
    return {'predicate_constructor_cases': predicate_cases, 'ticket_custody_cases': ticket_cases,
            'cancel_constructor_flag_cases': cancellation_cases}


def negative_controls(evaluator, before, after):
    rejected = []

    def reject(label, expected, gate, mutate):
        node = copy.deepcopy(after['ast'])
        mutate(node)
        try:
            gate(node)
        except ValueError as error:
            require(expected in str(error), 'wrong failure for ' + label + ': ' + str(error))
            rejected.append({'defect': label, 'diagnostic': str(error)})
        else:
            raise ValueError('planted real-source-IR defect escaped: ' + label)
        check_shapes(after['ast'])
        check_semantics(evaluator, before['ast'], after['ast'])

    for name, path, target, _ in SITES:
        label = name + '/' + target
        reject('scalar-equal-restored/' + label, 'expected Phase match', check_shapes,
               lambda n, name=name, path=path, target=target:
               replace(n, name, path, primitive(target)))
        reject('omit-Phase-branch/' + label, 'Phase match branch', check_shapes,
               lambda n, name=name, path=path:
               at(declarations(n)[name], path)['branches'].pop())
        reject('duplicate-Phase-branch/' + label, 'Phase match branch', check_shapes,
               lambda n, name=name, path=path:
               at(declarations(n)[name], path)['branches'].__setitem__(
                   0, copy.deepcopy(at(declarations(n)[name], path)['branches'][1])))
        for wrong_phase in (target, next(p for p in PHASES if p != target)):
            reject('wrong-truth-value/' + label + '/' + wrong_phase,
                   'Phase predicate semantic mismatch',
                   lambda n: check_semantics(evaluator, before['ast'], n),
                   lambda n, name=name, path=path, wrong_phase=wrong_phase:
                   at(declarations(n)[name], path)['branches'][PHASES.index(wrong_phase)]
                   ['body'].update(value=wrong_phase != target))
    reject('add-unhandled-Phase-constructor', 'Phase domain/payload shape changed', check_shapes,
           lambda n: declarations(n)['Phase']['constructors'].append({'name': 'extra', 'fields': []}))
    reject('add-Phase-payload', 'Phase domain/payload shape changed', check_shapes,
           lambda n: declarations(n)['Phase']['constructors'][0]['fields'].append({'kind': 'bool'}))
    reject('change-Phase-scrutinee', 'wrong Phase match scrutinee', check_shapes,
           lambda n: at(declarations(n)['ticketMatches'], SITES[0][1])
           ['scrutinee'].update(field='resume'))
    exact = lambda n: check_exact(before, dict(after, ast=n,
                         raw=after['prefix'] + canonical(n) + after['suffix']))
    reject('change-unrelated-ticket-binder', 'module differs beyond', exact,
           lambda n: declarations(n)['acknowledge']['parameters'][1].update(name='ticket'))
    reject('change-unrelated-cancel-funding-state', 'module differs beyond', exact,
           lambda n: next(f for f in declarations(n)['cancel']['body']['else_value']
                         ['else_value']['fields'] if f['field'] == 'paid')['value'].update(value='1'))
    reject('change-unrelated-ticket-handle-equality', 'module differs beyond', exact,
           lambda n: next(x for x in walk(declarations(n)['ticketMatches'])
                         if x.get('kind') == 'call')['function'].update(name='nullHandle'))
    require(len(rejected) == 21, 'missing or duplicated intended mutant inventory')
    return rejected


IMPLEMENTATION_SHA256 = '2e14c9fc8c49d89b42dd11abd63c8f08d79e326f2f52987bc741938df6544995'


def check_implementation(source=None):
    # Bind the complete executable diagnostic, including its entry point and
    # case/mutant inventories. Exclude only this check and its digest literal.
    # This is an anti-omission control, not a security boundary for hostile code.
    if source is None:
        source = Path(__file__).read_text()
    tree = ast.parse(source)
    require(not any(isinstance(n, ast.Assert) for n in ast.walk(tree)),
            'diagnostic implementation uses removable assertions')
    nodes = [n for n in tree.body if not (
        isinstance(n, ast.FunctionDef) and n.name == 'check_implementation') and not (
        isinstance(n, ast.Assign) and len(n.targets) == 1 and
        isinstance(n.targets[0], ast.Name) and n.targets[0].id == 'IMPLEMENTATION_SHA256')]
    digest = sha(ast.dump(ast.Module(body=nodes, type_ignores=[]),
                          include_attributes=False).encode())
    require(digest == IMPLEMENTATION_SHA256,
            'diagnostic executable inventory changed, filtered, skipped, stale or wrong-file')


def implementation_controls():
    source = Path(__file__).read_text()
    controls = (
        ('filter-constructor-cases', 'for phase in PHASES:', 'for phase in PHASES[:1]:'),
        ('omit-real-source-mutants', 'rejected = negative_controls(evaluator, before, after)',
         'rejected = []'),
        ('disable-filter-guard', "require(len(sys.argv) == 1, 'filters, selection and other arguments are not allowed')",
         "require(True, 'filters, selection and other arguments are not allowed')"),
        ('disable-optimization-guard', "require(sys.flags.optimize == 0 and __debug__, 'optimized Python is not allowed')",
         "require(True, 'optimized Python is not allowed')"),
        ('use-stale-source', 'before, after = parse(baseline), parse((ROOT / SOURCE).read_bytes())',
         'before, after = parse(baseline), parse(baseline)'),
    )
    rejected = []
    for label, before, after in controls:
        # The source also contains each string in this inventory, so replace the
        # executable occurrence selected by its line indentation, not the list.
        lines = source.splitlines(keepends=True)
        locations = [i for i, line in enumerate(lines) if line.strip() == before]
        require(len(locations) == 1, 'ambiguous implementation mutant: ' + label)
        index = locations[0]
        lines[index] = lines[index].replace(before, after)
        try:
            check_implementation(''.join(lines))
        except ValueError as error:
            require('diagnostic executable inventory changed' in str(error),
                    'wrong implementation-control failure: ' + label)
            rejected.append(label)
        else:
            raise ValueError('diagnostic implementation omission escaped: ' + label)
    require(len(rejected) == 5, 'missing implementation-control inventory')
    check_implementation(source)
    return rejected


def main():
    require(sys.flags.optimize == 0 and __debug__, 'optimized Python is not allowed')
    require(len(sys.argv) == 1, 'filters, selection and other arguments are not allowed')
    check_implementation()
    baseline = subprocess.run(['git', '-C', str(ROOT), 'show', BASELINE + ':' + SOURCE],
                              check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE).stdout
    before, after = parse(baseline), parse((ROOT / SOURCE).read_bytes())
    check_exact(before, after)
    import diagnostic as evaluator
    require(canonical(evaluator.mods[MODULE]) == canonical(after['ast']['declarations']),
            'evaluator loaded stale or wrong source module')
    cases = check_semantics(evaluator, before['ast'], after['ast'])
    rejected = negative_controls(evaluator, before, after)
    check_exact(before, after)
    restored = check_semantics(evaluator, before['ast'], after['ast'])
    implementation_rejections = implementation_controls()
    require(cases == restored, 'restored diagnostic inventory mismatch')
    report = {
        'schema': 'uorc/enumeration-phase-comparison-diagnostic/1',
        'status': 'pass-source-diagnostic-only-unaccepted',
        'baseline_commit': BASELINE, 'source_path': SOURCE,
        'baseline_source_sha256': sha(before['raw']), 'current_source_sha256': sha(after['raw']),
        'diagnostic_sha256': sha(Path(__file__).read_bytes()),
        'phase_constructors': list(PHASES), 'phase_constructor_count': len(PHASES),
        'phase_payload_count': 0, 'exact_predicate_replacement_count': len(SITES),
        'sites': [{'definition': name, 'path': list(path), 'target': target,
                   'finding_origin': origin} for name, path, target, origin in SITES],
        'all_other_source_ast_and_bytes_unchanged': True,
        'previous_ticket_binder_repairs_unchanged': True,
        'semantic_cases': cases, 'rejected_real_source_ir_mutants': rejected,
        'restored_source_pass': True,
        'rejected_implementation_omissions': implementation_rejections,
        'qualification': {'finite_source_evaluation_only': True,
            'official_source_check': False, 'kernel_proofs': False,
            'generated_native_execution': False, 'full_sdk_acceptance': False},
    }
    print(json.dumps(report, sort_keys=True, indent=2))


if __name__ == '__main__':
    main()
