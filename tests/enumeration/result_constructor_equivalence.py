#!/usr/bin/env python3
"""Exact four-site Result annotation audit; not compiler or runtime admission.

Only the four named type_arguments arrays may be added. Erasing those exact
arrays must recover every original source byte. Small constructor-fragment
evaluations use symbolic payloads, not heap executions or finite proof evidence.
"""
import ast
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
BASELINE = '48b9436b2443c1fd3df725572cdea10863f11f39'
MODULE = 'Uorc.Search.OwnedEnumeration'
SOURCE = 'src/Uorc/Search/OwnedEnumeration.lex.tex'
BEFORE_SHA256 = '1fac8abfab28e1d3e50e893af515547b4a5a9798ed7b9fefeb9dcee9b2ec71d6'
AFTER_SHA256 = 'a736436cc5c54a4740d1660a0d06f8b5b548ed5051f356d29de22c1e4117cdd0'
PRESERVED = (
    ('src/Uorc/Search/UnsignedByteConversion.lex.tex', 0),
    ('src/Uorc/Search/OwnedEnumerationModel.lex.tex', 45),
    ('src/Uorc/Search/OwnedEnumerationProofs.lex.tex', 18),
)
SITES = (
    ('appendStep', ('declarations', 23, 'body', 'branches', 1, 'body', 'arguments', 2),
     'Result.ok', 'builderResult.parameters[2].type'),
    ('finishStep', ('declarations', 24, 'body', 'branches', 1, 'body', 'arguments', 2),
     'Result.ok', 'builderResult.parameters[2].type'),
    ('acknowledge', ('declarations', 27, 'body', 'then_value'),
     'Result.ok', 'acknowledge.result'),
    ('acknowledge', ('declarations', 27, 'body', 'else_value'),
     'Result.error', 'acknowledge.result'),
)
COMPILER_AUDIT = {
    'revision': '71defb4c663aaec5b9a3aab04cbc6a03e7106227',
    'path': 'crates/lexlean/src/ir/semantic.rs',
    'file_sha256': 'c5309614b5693bed5e5f4c75aa28bcb5c98e88443b0e8095d00605e5a7edef21',
    'constructor_schema_lines': [463, 468],
    'result_constructor_signature_lines': [7669, 7688],
    'order': ['ok', 'error'],
    'method': 'read-only git object inspection; no compiler build or execution',
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':')).encode()


def sha(value):
    return hashlib.sha256(value).hexdigest()


def at(node, path):
    for key in path:
        node = node[key]
    return node


def pointer(path):
    return '/' + '/'.join(map(str, path))


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, 'duplicate semantic JSON key: ' + key)
        result[key] = value
    return result


def parse(raw, module=None):
    text = raw.decode('utf-8')
    marker = r'\semanticdata{'
    require(text.count(marker) == 1, 'wrong semanticdata inventory')
    if module is not None:
        require(text.startswith(r'\begin{lexlean}{' + module + '}\n'),
                'wrong source module')
    start = text.index(marker) + len(marker)
    node, length = json.JSONDecoder(object_pairs_hook=unique_object).raw_decode(text[start:])
    require(text[start:start + length].encode() == canonical(node), 'noncanonical source JSON')
    require(node.get('spec') == 'lexlean/semantic-module/1', 'wrong semantic schema')
    names = [d['name'] for d in node['declarations']]
    require(len(names) == len(set(names)), 'duplicate declaration')
    return node, text[:start].encode(), text[start + length:].encode()


def git_file(path):
    return subprocess.run(['git', '-C', str(ROOT), 'show', BASELINE + ':' + path],
                          check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE).stdout


def named(name, module=None):
    member = {'name': name}
    if module is not None:
        member['module'] = module
    return {'kind': 'named', 'member': member, 'arguments': []}


def expected_pair(name):
    if name in ('appendStep', 'finishStep'):
        return [named('BuilderTransition', 'Uorc.Core.RuntimeByteBuilder'),
                named('BuilderFailure', 'Uorc.Core.RuntimeByteBuilder')]
    return [named('Cursor'), named('Error')]


def constructor_inventory(node, path=()):
    result = []
    if isinstance(node, dict):
        if node.get('kind') == 'constructor' and node.get('constructor', {}).get('name') in (
                'Result.ok', 'Result.error'):
            result.append(path)
        for key, value in node.items():
            result.extend(constructor_inventory(value, path + (key,)))
    elif isinstance(node, list):
        for key, value in enumerate(node):
            result.extend(constructor_inventory(value, path + (key,)))
    return result


def check_source(before, after):
    require(sha(before) == BEFORE_SHA256, 'wrong frozen baseline bytes')
    old, prefix, suffix = parse(before, MODULE)
    current, new_prefix, new_suffix = parse(after, MODULE)
    require(len(old['declarations']) == 31, 'wrong producer declaration count')
    require(set(constructor_inventory(old)) == {s[1] for s in SITES},
            'Result constructor inventory is not exactly the four authorized sites')
    expected = copy.deepcopy(old)
    erased = copy.deepcopy(current)
    mapping = []
    for name, path, constructor, context in SITES:
        original, updated = at(old, path), at(current, path)
        declaration = old['declarations'][path[1]]
        require(declaration['name'] == name, 'wrong declaration at authorized site')
        require(original['kind'] == 'constructor' and original['constructor'] == {
            'name': constructor} and 'type_arguments' not in original,
            'wrong original constructor shape')
        pair = expected_pair(name)
        require(updated.get('type_arguments') == pair,
                'wrong, swapped, missing or additional Result type pair: ' + pointer(path))
        require(updated.get('constructor') == original['constructor'], 'constructor changed')
        if name in ('appendStep', 'finishStep'):
            owner = old['declarations'][13]
            require(owner['name'] == 'builderResult' and owner['parameters'][2]['name'] == 'result',
                    'wrong contextual result parameter')
            result_type = owner['parameters'][2]['type']
            branch = declaration['body']['branches'][1]
            require(branch['binders'] == ['built'] and
                    branch['constructor'] == {'name': 'Result.ok'}, 'wrong original built binder')
            require(branch['body']['kind'] == 'call' and
                    branch['body']['function'] == {'name': 'builderResult'} and
                    len(branch['body']['arguments']) == 3, 'wrong builderResult call')
            require(original['arguments'] == [{'kind': 'var', 'name': 'built'}],
                    'wrong original built payload')
        else:
            result_type = declaration['result']
        require(result_type == {'kind': 'result', 'ok': pair[0], 'error': pair[1]},
                'contextual result type differs from explicit annotation')
        at(expected, path)['type_arguments'] = copy.deepcopy(pair)
        del at(erased, path)['type_arguments']
        mapping.append({
            'declaration': name, 'site': pointer(path), 'constructor': constructor,
            'classification': ('retained compiler first-error localization forwarded by parent'
                               if name == 'appendStep' else
                               'same-class source audit; not a separately observed compiler error'),
            'context': context, 'context_result_type': result_type,
            'before_type_arguments': None, 'after_type_arguments': pair,
            'before_node_sha256': sha(canonical(original)),
            'after_node_sha256': sha(canonical(updated)),
            'payload_sha256': sha(canonical(original['arguments'])),
            'before_declaration_sha256': sha(canonical(declaration)),
            'after_declaration_sha256': sha(canonical(current['declarations'][path[1]])),
        })
    require(current == expected, 'source AST changed outside the four type annotations')
    require(new_prefix == prefix and new_suffix == suffix, 'source surface changed')
    require(after == prefix + canonical(expected) + suffix, 'unexpected source bytes')
    require(erased == old and new_prefix + canonical(erased) + new_suffix == before,
            'exact four-site erasure did not recover the original complete source')
    require(sha(after) == AFTER_SHA256, 'wrong final source hash')
    return current, mapping


def check_preserved(path, before, after, count):
    require(before == after, 'preserved helper or proof root changed: ' + path)
    module = parse(after)[0]
    theorems = [d for d in module['declarations'] if d['kind'] == 'theorem']
    require(len(theorems) == count, 'wrong complete theorem count: ' + path)
    return {'path': path, 'sha256': sha(after), 'theorem_count': count,
            'complete_theorems': [{'name': d['name'], 'full_theorem_sha256': sha(canonical(d))}
                                  for d in theorems]}


def check_repository_examples():
    examples = []
    path = 'src/Uorc/Core/ByteBuilder.lex.tex'
    raw = (ROOT / path).read_bytes()
    require(raw == git_file(path), 'supporting JSON example changed')
    declaration = parse(raw)[0]['declarations'][7]
    require(declaration['name'] == 'start', 'wrong supporting JSON declaration')
    pair = [named('Builder'), named('Error')]
    require(declaration['result'] == {'kind': 'result', 'ok': pair[0], 'error': pair[1]},
            'wrong supporting JSON result type')
    for branch, constructor in (('then_value', 'Result.error'), ('else_value', 'Result.ok')):
        node = declaration['body'][branch]
        require(node['constructor'] == {'name': constructor} and node['type_arguments'] == pair,
                'repository example differs from success/error type argument schema')
        examples.append({'path': path, 'file_sha256': sha(raw), 'declaration': 'start',
                         'site': '/declarations/7/body/' + branch,
                         'constructor': constructor, 'type_arguments': pair,
                         'node_sha256': sha(canonical(node))})
    path = 'src/Uorc/Core/RuntimeByteBuilder.lex.tex'
    raw = (ROOT / path).read_bytes()
    require(raw == git_file(path), 'supporting producer callees changed')
    text = raw.decode()
    for function in ('beginAppendOwned', 'beginFinishOwned'):
        marker = '  name: ' + function + ',\n'
        require(text.count(marker) == 1, 'wrong producer callee inventory')
        declaration = text.split(marker)[1].split('\n}\n')[0]
        require('  result: (Result BuilderTransition BuilderFailure),\n' in declaration,
                'producer callee return type changed')
        require('(ctor Result.ok [BuilderTransition, BuilderFailure]' in declaration and
                '(ctor Result.error [BuilderTransition, BuilderFailure]' in declaration,
                'producer callee source examples differ from success/error schema')
        examples.append({'path': path, 'file_sha256': sha(raw), 'declaration': function,
                         'result': 'Result BuilderTransition BuilderFailure',
                         'surface_constructor_type_arguments': ['BuilderTransition', 'BuilderFailure']})
    require(len(examples) == 4, 'supporting examples were filtered')
    return examples


def evaluate_fragment(term, env):
    """Interpret only the actual four fragments; annotations do not compute."""
    kind = term['kind']
    if kind == 'var':
        return env[term['name']]
    if kind == 'constructor':
        ref = term['constructor']
        name = ref['name'] if ref['name'].startswith('Result.') else (
            ref.get('module', MODULE) + '.' + ref['name'])
        return (name, tuple(evaluate_fragment(a, env) for a in term['arguments']))
    if kind == 'record':
        return {'record_type': term['type'],
                **{f['field']: evaluate_fragment(f['value'], env) for f in term['fields']}}
    if kind == 'project':
        return evaluate_fragment(term['value'], env)[term['field']]
    raise ValueError('unexpected constructor-fragment term kind: ' + kind)


def check_fragment_evaluation(old, new):
    fields = [f['name'] for f in old['declarations'][4]['fields']]
    require(len(fields) == 22 and len(set(fields)) == 22, 'wrong full cursor field inventory')
    cases = []
    for sample in range(3):
        cursor = {f: ('symbolic-cursor-field', sample, f) for f in fields}
        built = {'symbolic_builder_transition': sample, 'payload': object()}
        env = {'cursor': cursor, 'built': built}
        for name, path, constructor, _ in SITES:
            first = evaluate_fragment(at(old, path), env)
            second = evaluate_fragment(at(new, path), env)
            require(first == second, 'constructor-fragment evaluation changed')
            require(second[0] == constructor and len(second[1]) == 1,
                    'constructor or payload arity changed')
            if name in ('appendStep', 'finishStep'):
                require(second[1][0] is built, 'built payload identity changed')
            elif constructor == 'Result.error':
                require(second[1][0] == (MODULE + '.Error.invalidTicket', ()),
                        'acknowledge error changed')
            else:
                expected = {'record_type': {'name': 'Cursor'}, **cursor,
                            'phase': (MODULE + '.Phase.release', ()),
                            'resume': (MODULE + '.Phase.scanStart', ()),
                            'releaseSlot': (MODULE + '.Slot.ticket', ())}
                require(second[1][0] == expected, 'acknowledge custody fields changed')
            cases.append({'site': pointer(path), 'symbolic_payload_sample': sample,
                          'unchanged': True})
    require(len(cases) == 12, 'constructor-fragment evaluation was filtered')
    return cases


def source_mutations(before, after, current):
    _, prefix, suffix = parse(after, MODULE)
    rejected = []

    def reject(label, mutate, diagnostic):
        mutant = copy.deepcopy(current)
        mutate(mutant)
        raw = prefix + canonical(mutant) + suffix
        require(raw != after, 'mutation did not change the actual source IR')
        try:
            check_source(before, raw)
        except ValueError as error:
            require(diagnostic in str(error), 'wrong negative-control diagnostic: ' + str(error))
            rejected.append({'defect': label, 'mutant_source_sha256': sha(raw),
                             'diagnostic': str(error)})
        else:
            raise ValueError('planted source defect escaped: ' + label)
        check_source(before, after)

    for index, (name, path, _, _) in enumerate(SITES):
        for defect in ('missing', 'swapped', 'wrong', 'one', 'extra'):
            def mutate(node, path=path, defect=defect):
                target = at(node, path)
                if defect == 'missing':
                    del target['type_arguments']
                elif defect == 'swapped':
                    target['type_arguments'].reverse()
                elif defect == 'wrong':
                    target['type_arguments'][0] = {'kind': 'unit'}
                elif defect == 'one':
                    target['type_arguments'].pop()
                else:
                    target['type_arguments'].append({'kind': 'unit'})
            reject(str(index) + '-' + name + '-' + defect + '-pair', mutate,
                   'wrong, swapped, missing or additional Result type pair')
    reject('wrong-built-binder-reference',
           lambda n: at(n, SITES[0][1])['arguments'][0].update(name='arena'),
           'source AST changed outside')
    reject('wrong-match-binder',
           lambda n: n['declarations'][24]['body']['branches'][1].update(binders=['other']),
           'source AST changed outside')
    reject('wrong-acknowledge-error-body',
           lambda n: at(n, SITES[3][1])['arguments'][0]['constructor'].update(name='Error.invalidPhase'),
           'source AST changed outside')
    reject('wrong-acknowledge-result-type',
           lambda n: n['declarations'][27]['result'].update(ok={'kind': 'unit'}),
           'source AST changed outside')
    reject('wrong-acknowledge-custody-body',
           lambda n: at(n, SITES[2][1])['arguments'][0]['fields'][0]['value']['constructor'].update(
               name='Phase.cancelled'), 'source AST changed outside')
    reject('wrong-call-argument-order',
           lambda n: n['declarations'][23]['body']['branches'][1]['body']['arguments'].reverse(),
           'wrong, swapped, missing or additional Result type pair')
    reject('extra-unrequested-annotation',
           lambda n: n['declarations'][8]['body']['fields'][0]['value'].update(type_arguments=[]),
           'source AST changed outside')
    require(len(rejected) == 27, 'missing source mutant inventory')
    return rejected


IMPLEMENTATION_SHA256 = 'b1a7b331dbfcf6b25514b67a3394ed23a7b581cb1ddd14e4349c2e5f47f4b01e'


def check_implementation(source=None):
    source = Path(__file__).read_text() if source is None else source
    tree = ast.parse(source)
    require(not any(isinstance(n, ast.Assert) for n in ast.walk(tree)), 'removable assertions')
    seals = [n for n in tree.body if isinstance(n, ast.Assign) and len(n.targets) == 1 and
             isinstance(n.targets[0], ast.Name) and n.targets[0].id == 'IMPLEMENTATION_SHA256']
    require(len(seals) == 1, 'wrong gate seal inventory')
    seals[0].value = ast.Constant(value='SELF')
    require(sha(ast.dump(tree, include_attributes=False).encode()) == IMPLEMENTATION_SHA256,
            'gate implementation skipped, filtered, stale or changed')


def process_controls():
    script = Path(__file__).resolve()
    controls = []

    def reject(label, command, diagnostic):
        result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        require(result.returncode != 0 and result.stdout == '' and diagnostic in result.stderr,
                'process bypass escaped or failed for the wrong reason: ' + label)
        controls.append({'defect': label, 'returncode': result.returncode, 'diagnostic': diagnostic})

    reject('optimized-python', [sys.executable, '-B', '-O', str(script)],
           'optimized Python is not allowed')
    reject('filtered-process', [sys.executable, '-B', str(script), '--only=appendStep'],
           'filters and other arguments are not allowed')
    source = script.read_text()
    mutants = (
        ('skip-exact-source-gate', '    current, mapping = check_source(before, after)\n',
         '    current, mapping = parse(after)[0], []\n'),
        ('omit-full-theorem-inventory',
         "    theorems = [d for d in module['declarations'] if d['kind'] == 'theorem']\n",
         '    theorems = []\n'),
        ('skip-fragment-equivalence', '    evaluations = check_fragment_evaluation(old, current)\n',
         '    evaluations = []\n'),
        ('select-wrong-source-file', "SOURCE = 'src/Uorc/Search/OwnedEnumeration.lex.tex'\n",
         "SOURCE = 'src/Uorc/Search/OwnedEnumerationStream.lex.tex'\n"),
        ('omit-actual-mutants', '    negatives = source_mutations(before, after, current)\n',
         '    negatives = []\n'),
    )
    with tempfile.TemporaryDirectory(prefix='uorc-result-gate-controls-') as temporary:
        target = Path(temporary) / 'tests' / 'enumeration' / script.name
        target.parent.mkdir(parents=True)
        for label, needle, replacement in mutants:
            require(source.count(needle) == 1, 'ambiguous gate mutation: ' + label)
            target.write_text(source.replace(needle, replacement, 1))
            reject(label, [sys.executable, '-B', str(target)],
                   'gate implementation skipped, filtered, stale or changed')
    require(len(controls) == 7, 'missing process control inventory')
    return controls


def main():
    require(sys.flags.optimize == 0 and __debug__, 'optimized Python is not allowed')
    require(len(sys.argv) == 1, 'filters and other arguments are not allowed')
    check_implementation()
    before, after = git_file(SOURCE), (ROOT / SOURCE).read_bytes()
    current, mapping = check_source(before, after)
    old = parse(before, MODULE)[0]
    preserved = []
    proof_controls = []
    for path, count in PRESERVED:
        original, raw = git_file(path), (ROOT / path).read_bytes()
        preserved.append(check_preserved(path, original, raw, count))
        module, prefix, suffix = parse(raw)
        if count:
            theorem = next(d for d in module['declarations'] if d['kind'] == 'theorem')
            theorem['axioms'].append('Classical.choice')
            diagnostic = 'preserved helper or proof root changed'
            try:
                check_preserved(path, original, prefix + canonical(module) + suffix, count)
            except ValueError as error:
                require(diagnostic in str(error), 'wrong preserved-theorem mutation diagnosis')
                proof_controls.append({'path': path, 'defect': 'change-complete-theorem-axiom-policy',
                                       'diagnostic': str(error)})
            else:
                raise ValueError('changed complete theorem escaped preservation check')
            check_preserved(path, original, raw, count)
    require(sum(p['theorem_count'] for p in preserved) == 63 and len(proof_controls) == 2,
            'full 63-theorem contract preservation was skipped')
    examples = check_repository_examples()
    evaluations = check_fragment_evaluation(old, current)
    negatives = source_mutations(before, after, current)
    controls = process_controls()
    check_implementation()
    require((ROOT / SOURCE).read_bytes() == after, 'source changed during gate')
    print(json.dumps({
        'schema': 'uorc/enumeration-result-constructor-repair/1',
        'status': 'pass-exact-source-erasure-and-fragment-evaluation-only',
        'baseline_commit': BASELINE, 'source_path': SOURCE,
        'before_sha256': sha(before), 'after_sha256': sha(after),
        'gate_path': 'tests/enumeration/result_constructor_equivalence.py',
        'gate_sha256': sha(Path(__file__).read_bytes()),
        'compiler_source_audit': COMPILER_AUDIT,
        'unchanged_repository_schema_and_callee_examples': examples,
        'observed_diagnostic': {
            'root': 'Uorc.Search.OwnedEnumerationProofs', 'code': 'LLT4001',
            'message': 'constructor `Result.ok` requires explicit success and error type arguments',
            'source': 'retained qualifier terminal report forwarded by parent',
            'site_localization_from_observed_error': pointer(SITES[0][1]),
            'declaration': 'appendStep',
            'context': 'builderResult third argument, Result.ok built',
            'required_type_arguments': expected_pair('appendStep'),
            'independently_reran_or_remeasured_receipt': False,
        },
        'site_type_hash_map': mapping,
        'exact_four_site_type_erasure_recovers_complete_original_bytes': True,
        'computations_branches_types_binders_ownership_argument_order_unchanged': True,
        'preserved_helper_and_proof_sources': preserved,
        'complete_original_theorem_count': 63,
        'symbolic_constructor_fragment_evaluations': evaluations,
        'rejected_source_ir_mutants': negatives,
        'rejected_complete_theorem_mutants': proof_controls,
        'rejected_real_process_controls': controls,
        'restored_exact_delta_pass': True,
        'qualification': {
            'official_source_check_run': False, 'official_source_admission': False,
            'compiler_or_kernel_or_axiom_audit_run': False, 'broad_runtime_rerun': False,
            'native_execution': False, 'full_sdk_acceptance': False,
            'conversion_laws_proved': False, 'historical_receipts_relabelled': False,
        },
    }, sort_keys=True, indent=2))


if __name__ == '__main__':
    main()
