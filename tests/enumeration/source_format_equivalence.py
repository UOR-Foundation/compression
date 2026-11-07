#!/usr/bin/env python3
"""Exact, negative-tested source-format repair comparison, not SDK acceptance.

Compare the current three owned enumeration modules to the frozen proof-source
checkpoint. Only two omissions of empty linear_arithmetic.definitions fields
and two scope-aware parameter alpha-renamings are permitted.
No theorem/domain/proof-strategy change, general correctness, source acceptance,
kernel replay, observed axiom audit, or native execution is inferred.

Run from any directory in a clone containing the frozen checkpoint:
  PYTHONDONTWRITEBYTECODE=1 python -B tests/enumeration/source_format_equivalence.py
The full JSON report goes to stdout. No source or existing receipt is written.
"""
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
BASELINE = '9ca824709b0c6ffe4dbb647cb11a050b04247c19'
MODEL = 'Uorc.Search.OwnedEnumerationModel'
PROOFS = 'Uorc.Search.OwnedEnumerationProofs'
OWNED = 'Uorc.Search.OwnedEnumeration'
MODULES = (MODEL, PROOFS, OWNED)
PATHS = {m: 'src/' + m.replace('.', '/') + '.lex.tex' for m in MODULES}
FRESH = 'presentedTicket'
RENAME_SPECS = (
    ('ticketMatches', 3, 'retained-host-compiler-LLT4001-capture-diagnostic'),
    ('acknowledge', 1, 'source-audit-under-same-capture-rule-not-official-result'),
)
EMPTY_PROOFS = (
    'model_adjacent_ranks_have_no_intermediate_rank',
    'model_decoder_quotient_strictly_decreases',
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
        require(key not in result, 'duplicate JSON object key: ' + key)
        result[key] = value
    return result


def parse(data, module):
    text = data.decode('utf-8')
    marker = r'\semanticdata{'
    require(text.count(marker) == 1, 'expected one semanticdata form: ' + module)
    require(text.startswith(r'\begin{lexlean}{' + module + '}\n'),
            'module header mismatch: ' + module)
    start = text.index(marker) + len(marker)
    node, length = json.JSONDecoder(object_pairs_hook=unique_object).raw_decode(text[start:])
    require(text[start:start + length].encode() == canonical(node),
            'noncanonical semantic JSON: ' + module)
    require(node.get('spec') == 'lexlean/semantic-module/1', 'unexpected spec: ' + module)
    require(isinstance(node.get('declarations'), list), 'missing declaration list')
    names = [d.get('name') for d in node['declarations']]
    require(all(isinstance(n, str) for n in names) and len(names) == len(set(names)),
            'missing/duplicate declaration name: ' + module)
    return {'raw': data, 'ast': node, 'prefix': text[:start].encode(),
            'suffix': text[start + length:].encode()}


def render(parsed, node):
    return parsed['prefix'] + canonical(node) + parsed['suffix']


def visit(node, path=()):
    if isinstance(node, dict):
        yield path, node
        for key, value in node.items():
            yield from visit(value, path + (key,))
    elif isinstance(node, list):
        for index, value in enumerate(node):
            yield from visit(value, path + (index,))


def omit_empty_proof_fields(module):
    result = copy.deepcopy(module)
    paths = []
    for index, declaration in enumerate(result['declarations']):
        if declaration['kind'] != 'theorem':
            continue
        for path, node in visit(declaration['proof']):
            if node.get('kind') == 'linear_arithmetic' and node.get('definitions') == []:
                del node['definitions']
                paths.append({'theorem': declaration['name'],
                              'path': ['declarations', index, 'proof', *path, 'definitions']})
    return result, paths


# The actual two renamed definitions contain only binder-free forms. Fail closed on any
# other form. The explicit environment resolves each var to the selected
# parameter; member names, constructor names and function references are never
# traversed as variables. let/lambda/match support makes shadowing explicit and
# respects shadowing; let shadowing is exercised below. Textual replacement is not alpha
# equivalence. A newly encountered binding/term form requires manual review.
CHILDREN = {
    'and': ('left', 'right'), 'or': ('left', 'right'),
    'not': ('value',), 'project': ('value',),
    'call': ('arguments',), 'constructor': ('arguments',),
    'primitive': ('arguments',), 'if': ('condition', 'then_value', 'else_value'),
    'bool': (), 'nat': (), 'integer': (), 'string': (), 'unit': (), 'nil': (),
    'cons': ('head', 'tail'), 'apply': ('function', 'arguments'),
}


def rename_bound(term, before, after, environment, selected, path=()):
    result = copy.deepcopy(term)
    changed = []
    kind = term.get('kind')
    if kind == 'var':
        if environment.get(term['name']) is selected:
            require(term['name'] == before, 'selected binding has unexpected spelling')
            require(after not in environment or environment[after] is selected,
                    'reverse rename would capture a bound variable')
            result['name'] = after
            changed.append([*path, 'name'])
        return result, changed

    def child(key, env=environment):
        value = term[key]
        if isinstance(value, list):
            result[key] = []
            for index, item in enumerate(value):
                renamed, locations = rename_bound(item, before, after, env, selected,
                                                   path + (key, index))
                result[key].append(renamed)
                changed.extend(locations)
        else:
            result[key], locations = rename_bound(value, before, after, env, selected,
                                                   path + (key,))
            changed.extend(locations)

    if kind == 'record':
        for index, field in enumerate(term['fields']):
            renamed, locations = rename_bound(field['value'], before, after, environment,
                                               selected, path + ('fields', index, 'value'))
            result['fields'][index]['value'] = renamed
            changed.extend(locations)
    elif kind == 'let':
        child('value')
        nested = dict(environment)
        nested[term['binder']['name']] = object()
        child('body', nested)
    elif kind == 'lambda':
        nested = dict(environment)
        for parameter in term['parameters']:
            nested[parameter['name']] = object()
        child('body', nested)
    elif kind == 'match':
        child('scrutinee')
        for index, branch in enumerate(term['branches']):
            nested = dict(environment)
            for binder in branch['binders']:
                require(isinstance(binder, str), 'unsupported pattern binder')
                nested[binder] = object()
            renamed, locations = rename_bound(branch['body'], before, after, nested,
                                               selected, path + ('branches', index, 'body'))
            result['branches'][index]['body'] = renamed
            changed.extend(locations)
    else:
        require(kind in CHILDREN, 'unsupported term in alpha comparison: ' + str(kind))
        for key in CHILDREN[kind]:
            child(key)
    return result, changed


def reverse_owned_renames(module):
    result = copy.deepcopy(module)
    records = []
    for name, variable_count, origin in RENAME_SPECS:
        declaration = next(d for d in result['declarations'] if d['name'] == name)
        require([p['name'] for p in declaration['parameters']] == ['cursor', FRESH],
                name + ' must bind exactly cursor and ' + FRESH)
        selected = object()
        environment = {'cursor': object(), FRESH: selected}
        declaration['body'], paths = rename_bound(declaration['body'], FRESH, 'ticket',
                                                   environment, selected, ('body',))
        declaration['parameters'][1]['name'] = 'ticket'
        require(len(paths) == variable_count,
                name + ': expected exactly ' + str(variable_count) + ' bound ticket variables')
        records.append({'declaration': name, 'finding_origin': origin,
                        'before': 'ticket', 'after': FRESH,
                        'parameter_count_renamed': 1,
                        'bound_variable_count_renamed': variable_count,
                        'scope_aware_reverse_rename_paths': [['parameters', 1, 'name'], *paths]})
    return result, records


def declarations(module):
    return {d['name']: d for d in module['declarations']}


def function_references(module):
    return [{'path': list(path) + ['function'], 'member': node['function']}
            for path, node in visit(module) if node.get('kind') == 'call']


def check(old, new):
    for module in MODULES:
        require(old[module]['prefix'] == new[module]['prefix'] and
                old[module]['suffix'] == new[module]['suffix'],
                'non-JSON source bytes changed: ' + module)
    expected_model, omissions = omit_empty_proof_fields(old[MODEL]['ast'])
    require(sorted(p['theorem'] for p in omissions) == sorted(EMPTY_PROOFS) and
            len(omissions) == 2, 'frozen Model does not have the two expected empty fields')
    require(new[MODEL]['raw'] == render(old[MODEL], expected_model),
            'Model differs beyond exact empty-field omissions')
    require(new[PROOFS]['raw'] == old[PROOFS]['raw'], 'OwnedEnumerationProofs changed')
    reversed_owned, rename_records = reverse_owned_renames(new[OWNED]['ast'])
    require(reversed_owned == old[OWNED]['ast'], 'production whole-module reverse AST differs')
    require(render(new[OWNED], reversed_owned) == old[OWNED]['raw'],
            'production reverse rename is not byte-identical')
    require(FRESH not in old[OWNED]['raw'].decode(), 'alpha name is not fresh in baseline')
    require(function_references(new[OWNED]['ast']) == function_references(old[OWNED]['ast']),
            'production function reference changed')

    statements = []
    contracts_old, contracts_new = [], []
    for module, count in ((MODEL, 45), (PROOFS, 18)):
        before = declarations(old[module]['ast'])
        after = declarations(new[module]['ast'])
        names = [n for n, d in before.items() if d['kind'] == 'theorem']
        require(len(names) == count, 'unexpected theorem count: ' + module)
        for name in names:
            a, b = before[name], after[name]
            contract_a = {k: v for k, v in a.items() if k != 'proof'}
            contract_b = {k: v for k, v in b.items() if k != 'proof'}
            require(contract_a == contract_b, 'changed theorem/domain/axiom policy: ' + name)
            normalized, _ = omit_empty_proof_fields({'declarations': [a]})
            require(normalized['declarations'][0]['proof'] == b['proof'],
                    'changed proof strategy: ' + name)
            contracts_old.append({'module': module, 'declaration': contract_a})
            contracts_new.append({'module': module, 'declaration': contract_b})
            statements.append({'module': module, 'name': name,
                               'old_statement_sha256': sha(canonical(a['statement'])),
                               'new_statement_sha256': sha(canonical(b['statement'])),
                               'old_contract_sha256': sha(canonical(contract_a)),
                               'new_contract_sha256': sha(canonical(contract_b)),
                               'old_proof_sha256': sha(canonical(a['proof'])),
                               'new_proof_sha256': sha(canonical(b['proof'])),
                               'normalized_old_proof_sha256': sha(canonical(
                                   normalized['declarations'][0]['proof']))})
    require(len(statements) == 63, 'not all 63 theorem statements were compared')
    owned_old = declarations(old[OWNED]['ast'])
    owned_new = declarations(new[OWNED]['ast'])
    renamed_names = {spec[0] for spec in RENAME_SPECS}
    unchanged = [name for name in owned_old if name not in renamed_names]
    require(len(unchanged) == 29 and all(canonical(owned_old[n]) == canonical(owned_new[n])
                                      for n in unchanged), 'other production declaration changed')
    for record in rename_records:
        name = record['declaration']
        record['old_declaration_sha256'] = sha(canonical(owned_old[name]))
        record['new_declaration_sha256'] = sha(canonical(owned_new[name]))
        record['reverse_renamed_declaration_sha256'] = sha(canonical(
            declarations(reversed_owned)[name]))
    return {
        'status': 'exact-source-format-equivalence-pass-unaccepted',
        'boundary': ('Structural/byte equivalence only. Not source elaboration, proof checking, '
                     'kernel replay, observed axiom audit, native execution or SDK acceptance.'),
        'baseline_commit': BASELINE,
        'source_hashes': {module: {'path': PATHS[module],
                                  'old_sha256': sha(old[module]['raw']),
                                  'new_sha256': sha(new[module]['raw'])} for module in MODULES},
        'theorem_count': len(statements),
        'canonical_statement_contracts_old_sha256': sha(canonical(contracts_old)),
        'canonical_statement_contracts_new_sha256': sha(canonical(contracts_new)),
        'empty_linear_arithmetic_fields_omitted': omissions,
        'model_all_other_bytes_and_ast_identical': True,
        'proof_programs_equal_modulo_only_empty_field_omission': True,
        'proofs_module_byte_identical': True,
        'production_alpha_equivalence': {
            'renamings': rename_records,
            'parameter_count_renamed': 2, 'bound_variable_count_renamed': 4,
            'whole_module_ast_reverse_equal': True, 'whole_module_bytes_reverse_equal': True,
            'unchanged_declaration_count': len(unchanged),
            'unchanged_declarations_byte_ast_equal': unchanged,
            'function_reference_count_unchanged': len(function_references(old[OWNED]['ast'])),
            'all_function_members_unchanged': True,
            'global_ticket_references_unchanged': [r for r in function_references(old[OWNED]['ast'])
                                                    if r['member'].get('name') == 'ticket'],
            'old_module_ast_sha256': sha(canonical(old[OWNED]['ast'])),
            'new_module_ast_sha256': sha(canonical(new[OWNED]['ast'])),
            'reverse_renamed_module_ast_sha256': sha(canonical(reversed_owned)),
        },
        'theorems': statements,
    }


def replace_ast(inputs, module, node):
    result = dict(inputs)
    result[module] = parse(render(inputs[module], node), module)
    return result


def negative_controls(old, new):
    rejected = []

    def mutation(label, module, mutate):
        node = copy.deepcopy(new[module]['ast'])
        mutate(node)
        try:
            check(old, replace_ast(new, module, node))
        except ValueError:
            rejected.append(label)
        else:
            raise ValueError('comparison accepted planted defect: ' + label)

    def theorem(node):
        return next(d for d in node['declarations'] if d['kind'] == 'theorem')

    mutation('reintroduce-empty-proof-field', MODEL,
             lambda n: declarations(n)[EMPTY_PROOFS[0]]['proof'].update(definitions=[]))
    mutation('remove-nonempty-proof-definition', MODEL,
             lambda n: theorem(n)['proof']['definitions'].pop())
    mutation('alter-nonempty-definition-list', MODEL,
             lambda n: theorem(n)['proof']['definitions'].append({'name': 'modelDecode'}))
    mutation('change-theorem-statement', MODEL,
             lambda n: next(v for _, v in visit(theorem(n)['statement'])
                            if v.get('kind') == 'nat').update(value='257'))
    mutation('change-theorem-domain', MODEL,
             lambda n: theorem(n)['parameters'][0].update(name='alteredDomain'))
    mutation('change-proof-strategy', MODEL,
             lambda n: theorem(n)['proof'].update(kind='changed_strategy'))
    mutation('change-model-definition', MODEL,
             lambda n: n['declarations'][0].update(axioms=['propext']))
    mutation('change-proofs-module', PROOFS,
             lambda n: theorem(n).update(axioms=['unapproved_axiom']))
    mutation('change-production-function-reference', OWNED,
             lambda n: next(v for _, v in visit(n) if v.get('kind') == 'call')
             ['function'].update(name='changedFunction'))
    mutation('rename-global-ticket-function-reference', OWNED,
             lambda n: next(v for _, v in visit(n) if v.get('kind') == 'call'
                            and v['function'].get('name') == 'ticket')
             ['function'].update(name=FRESH))
    mutation('rename-parameter-without-bound-variables', OWNED,
             lambda n: declarations(n)['ticketMatches']['parameters'][1].update(name='ticket'))
    mutation('change-bound-variable', OWNED,
             lambda n: next(v for _, v in visit(declarations(n)['ticketMatches']['body'])
                            if v.get('kind') == 'var' and v.get('name') == FRESH)
             .update(name='unboundTicket'))
    mutation('acknowledge-parameter-not-renamed', OWNED,
             lambda n: declarations(n)['acknowledge']['parameters'][1].update(name='ticket'))
    mutation('acknowledge-bound-reference-not-renamed', OWNED,
             lambda n: next(v for _, v in visit(declarations(n)['acknowledge']['body'])
                            if v.get('kind') == 'var' and v.get('name') == FRESH)
             .update(name='ticket'))
    mutation('acknowledge-argument-order-changed', OWNED,
             lambda n: declarations(n)['acknowledge']['parameters'].reverse())
    mutation('change-other-production-declaration', OWNED,
             lambda n: declarations(n)['ticket'].update(axioms=['propext']))

    selected = object()
    environment = {FRESH: selected}
    var = {'kind': 'var', 'name': FRESH}
    nested = {'kind': 'let', 'binder': {'name': FRESH, 'type': {'kind': 'bool'}},
              'value': copy.deepcopy(var), 'body': copy.deepcopy(var)}
    renamed, paths = rename_bound(nested, FRESH, 'ticket', environment, selected)
    require(renamed['value']['name'] == 'ticket' and renamed['body']['name'] == FRESH
            and len(paths) == 1, 'scope checker did not respect let shadowing')
    nested['binder']['name'] = 'ticket'
    try:
        rename_bound(nested, FRESH, 'ticket', environment, selected)
    except ValueError:
        rejected.append('reverse-rename-variable-capture')
    else:
        raise ValueError('scope checker accepted a variable-capturing reverse rename')
    call = {'kind': 'call', 'function': {'name': 'ticket'}, 'arguments': [copy.deepcopy(var)]}
    renamed, paths = rename_bound(call, FRESH, 'ticket', environment, selected)
    require(renamed['function'] == call['function'] and renamed['arguments'][0]['name'] == 'ticket'
            and len(paths) == 1, 'scope checker altered a global function reference')
    return {'rejected_defects': rejected, 'restored_baseline_pass': check(old, new)['status'],
            'lexical_shadowing_control': 'pass', 'global_function_reference_control': 'pass'}


def main():
    require(sys.flags.optimize == 0 and __debug__, 'optimized Python is not allowed')
    require(len(sys.argv) == 1, 'filter, selection and other arguments are not allowed')
    old, new = {}, {}
    for module, path in PATHS.items():
        baseline = subprocess.run(['git', '-C', str(ROOT), 'show', BASELINE + ':' + path],
                                  check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE).stdout
        old[module] = parse(baseline, module)
        new[module] = parse((ROOT / path).read_bytes(), module)
    report = check(old, new)
    report['negative_controls'] = negative_controls(old, new)
    report['diagnostic'] = {'path': Path(__file__).relative_to(ROOT).as_posix(),
                            'sha256': sha(Path(__file__).read_bytes())}
    print(json.dumps(report, sort_keys=True, indent=2))


if __name__ == '__main__':
    main()
