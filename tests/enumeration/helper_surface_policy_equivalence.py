#!/usr/bin/env python3
"""Exact source-surface and measured-policy repair audit, not compilation."""
import ast
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
BASELINE = 'aa4772de020e521ba9310bdaac75cfa638bbf1be'
SOURCE = 'src/Uorc/Search/UnsignedByteConversion.lex.tex'
ROOTS = ('src/Uorc/Search/OwnedEnumerationModel.lex.tex',
         'src/Uorc/Search/OwnedEnumerationProofs.lex.tex')
RECEIPT = {'library_file_id': 'libfile_e5eae0d70ee881919643d3a58fe407ef',
           'sha256': 'f59717fde36f204c2b2c48edad63ff6bfa13fa235a95b43501d70a56fc0528ad',
           'source': 'retained qualifier terminal report forwarded by parent'}


def require(value, message):
    if not value:
        raise ValueError(message)


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':')).encode()


def sha(data):
    return hashlib.sha256(data).hexdigest()


def old_file(path):
    return subprocess.run(['git', '-C', str(ROOT), 'show', BASELINE + ':' + path],
                          check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE).stdout


def parse(data):
    text = data.decode()
    marker = r'\semanticdata{'
    require(text.count(marker) == 1, 'wrong semantic data inventory')
    start = text.index(marker) + len(marker)
    module, length = json.JSONDecoder().raw_decode(text[start:])
    require(text[start:start + length].encode() == canonical(module), 'noncanonical source JSON')
    return module, text[:start].encode(), text[start + length:].encode()


def check(before, after):
    old, prefix, suffix = parse(before)
    current, current_prefix, current_suffix = parse(after)
    require([d['name'] for d in old['declarations']] == ['nextByte', 'fromNat', 'boundary'],
            'wrong baseline declaration inventory')
    expected = copy.deepcopy(old)
    for declaration in expected['declarations'][:2]:
        require(declaration['axioms'] == [], 'wrong original declared policy')
        declaration['axioms'] = ['Quot.sound', 'propext']
    value = expected['declarations'][2]['body']['value']
    require(value.count('n % 256') == 1, 'wrong original boundary text')
    expected['declarations'][2]['body']['value'] = value.replace('n % 256', 'n modulo 256')
    require(expected['declarations'][2]['axioms'] == [], 'boundary policy was not empty')
    old_title = rb'\title{Source-owned natural byte conversion}'
    require(prefix.count(old_title) == 1, 'wrong original title')
    expected_prefix = prefix.replace(old_title, rb'\title{Parameters}')
    require(current == expected, 'unexpected semantic AST or declaration-local policy change')
    require(current_prefix == expected_prefix and current_suffix == suffix,
            'unexpected source surface change')
    require(after == expected_prefix + canonical(expected) + suffix,
            'unexpected source bytes')
    for first, second in zip(old['declarations'][:2], current['declarations'][:2]):
        require({k: v for k, v in first.items() if k != 'axioms'} ==
                {k: v for k, v in second.items() if k != 'axioms'},
                'computation, type or recursion changed')
    return current


IMPLEMENTATION_SHA256 = 'a6c0b49eec82bf548fa74295d060e971e9943393339725bb46cf5df3adab8232'


def check_implementation(source=None):
    source = Path(__file__).read_text() if source is None else source
    tree = ast.parse(source)
    require(not any(isinstance(n, ast.Assert) for n in ast.walk(tree)), 'removable assertions')
    nodes = [n for n in tree.body if not (
        isinstance(n, ast.FunctionDef) and n.name == 'check_implementation') and not (
        isinstance(n, ast.Assign) and len(n.targets) == 1 and
        isinstance(n.targets[0], ast.Name) and n.targets[0].id == 'IMPLEMENTATION_SHA256')]
    require(sha(ast.dump(ast.Module(body=nodes, type_ignores=[]), include_attributes=False).encode()) ==
            IMPLEMENTATION_SHA256, 'gate implementation omitted, filtered, stale or changed')


def main():
    require(sys.flags.optimize == 0 and __debug__, 'optimized Python is not allowed')
    require(len(sys.argv) == 1, 'filters and other arguments are not allowed')
    check_implementation()
    before, after = old_file(SOURCE), (ROOT / SOURCE).read_bytes()
    current = check(before, after)
    roots = {}
    contracts = []
    for path in ROOTS:
        raw = (ROOT / path).read_bytes()
        require(raw == old_file(path), 'existing proof root changed: ' + path)
        module = parse(raw)[0]
        statements = [d for d in module['declarations'] if d['kind'] == 'theorem']
        contracts.extend({'path': path, 'name': d['name'], 'full_theorem_sha256': sha(canonical(d))}
                         for d in statements)
        roots[path] = {'sha256': sha(raw), 'theorem_count': len(statements)}
    require([roots[p]['theorem_count'] for p in ROOTS] == [45, 18], 'wrong full theorem inventory')
    rejected = []
    def reject(label, mutate):
        node = copy.deepcopy(current)
        mutate(node)
        _, prefix, suffix = parse(after)
        try:
            check(before, prefix + canonical(node) + suffix)
        except ValueError as error:
            require('unexpected semantic AST' in str(error), 'wrong mutant failure')
            rejected.append({'defect': label, 'diagnostic': str(error)})
        else:
            raise ValueError('planted defect escaped: ' + label)
        check(before, after)
    reject('omit-nextByte-measured-policy', lambda n: n['declarations'][0].update(axioms=[]))
    reject('omit-fromNat-measured-policy', lambda n: n['declarations'][1].update(axioms=[]))
    reject('blanket-boundary-policy', lambda n: n['declarations'][2].update(axioms=['Quot.sound', 'propext']))
    reject('add-unobserved-axiom', lambda n: n['declarations'][0]['axioms'].append('Classical.choice'))
    reject('change-recursion-argument', lambda n: n['declarations'][1].update(recursive_argument='other'))
    reject('change-function-domain', lambda n: n['declarations'][1]['parameters'][0]['type'].update(kind='uint64'))
    reject('change-zero-branch', lambda n: n['declarations'][1]['body']['branches'][0]['body'].update(value='1'))
    reject('change-checked-add', lambda n: n['declarations'][0]['body']['scrutinee']['arguments'][1].update(value='2'))
    reject('restore-lexical-percent', lambda n: n['declarations'][2]['body'].update(value=n['declarations'][2]['body']['value'].replace('n modulo 256', 'n % 256')))
    require(len(rejected) == 9, 'missing mutant inventory')
    process_controls = []
    for arguments, diagnostic in [(['-O'], 'optimized Python is not allowed'),
                                  ([], 'filters and other arguments are not allowed')]:
        command = [sys.executable, '-B', *arguments, str(Path(__file__).resolve())]
        if not arguments:
            command.append('--filter=nextByte')
        result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        require(result.returncode != 0 and result.stdout == '' and diagnostic in result.stderr,
                'process bypass escaped')
        process_controls.append({'arguments': command[2:], 'returncode': result.returncode,
                                 'diagnostic': diagnostic})
    source = Path(__file__).read_text()
    omission = '    contracts = []\n'
    require(source.count(omission) == 1, 'ambiguous self-omission control')
    try:
        check_implementation(source.replace(omission, '    contracts = [] # omitted inventory\n', 1).replace(
            "if d['kind'] == 'theorem'", "if False"))
    except ValueError as error:
        require('gate implementation omitted' in str(error), 'wrong self-omission failure')
    else:
        raise ValueError('theorem-inventory omission escaped')
    print(json.dumps({'schema': 'uorc/enumeration-helper-surface-policy-repair/1',
        'status': 'pass-exact-delta-audit-only', 'baseline_commit': BASELINE,
        'source_path': SOURCE, 'before_sha256': sha(before), 'after_sha256': sha(after),
        'gate_sha256': sha(Path(__file__).read_bytes()), 'qualifier_receipt_reference': RECEIPT,
        'observed_policies_from_named_qualifier': {d['name']: d['axioms'] for d in current['declarations']},
        'computational_bodies_types_recursion_unchanged': True,
        'proof_roots': roots, 'complete_original_theorem_count': len(contracts),
        'complete_original_theorems': contracts, 'rejected_real_ir_mutants': rejected,
        'restored_exact_delta_pass': True, 'self_omission_rejected': True,
        'process_controls': process_controls,
        'qualification': {'this_gate_compiles_or_measures_axioms': False,
            'public_baseline_admitted': False, 'repaired_public_source_admitted': False,
            'full_Nat_conversion_law_proved': False, 'native_execution': False,
            'full_sdk_acceptance': False}}, sort_keys=True, indent=2))


if __name__ == '__main__':
    main()
