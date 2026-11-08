#!/usr/bin/env python3
"""Exact sixteen-site Option type annotation audit, never source admission."""
import ast
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
BASELINE = '0582704ce41d1a385c23e0ed9a986047a43e18ed'
SOURCE = 'src/Uorc/Search/OwnedEnumeration.lex.tex'
STREAM = 'src/Uorc/Search/EnumerationStream.lex.tex'
BRANCHES = (*range(9), *range(10, 16), 17)
GATE = 'option_constructor_equivalence.py'


def require(value, message):
    if not value:
        raise ValueError(message)


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':')).encode()


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def git(*args):
    return subprocess.run(['git', '-C', str(ROOT), *args], check=True,
                          stdout=subprocess.PIPE, stderr=subprocess.PIPE).stdout


def unique(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, 'duplicate semantic JSON key')
        result[key] = value
    return result


def parse(raw):
    text = raw.decode()
    marker = r'\semanticdata{'
    require(text.count(marker) == 1, 'wrong semanticdata inventory')
    start = text.index(marker) + len(marker)
    node, size = json.JSONDecoder(object_pairs_hook=unique).raw_decode(text[start:])
    require(text[start:start + size].encode() == canonical(node), 'noncanonical JSON')
    return node, text[:start].encode(), text[start + size:].encode()


def inventory(node, path=()):
    found = []
    if isinstance(node, dict):
        if node.get('kind') == 'constructor' and node.get('constructor', {}).get('name') in (
                'Option.some', 'Option.none'):
            found.append(path)
        for key, value in node.items():
            found.extend(inventory(value, path + (key,)))
    elif isinstance(node, list):
        for key, value in enumerate(node):
            found.extend(inventory(value, path + (key,)))
    return found


def check(before, after):
    original, prefix, suffix = parse(before)
    current, new_prefix, new_suffix = parse(after)
    expected, erased = copy.deepcopy(original), copy.deepcopy(current)
    paths = [('declarations', 26, 'body', 'branches', i, 'body') for i in BRANCHES]
    require(inventory(original) == paths and len(paths) == 16,
            'Option construction inventory differs from sixteen exact sites')
    for module in (original, current):
        declaration = module['declarations'][26]
        require(declaration['name'] == 'requiredWork' and
                declaration['result'] == {'kind': 'option', 'value': {'kind': 'uint64'}} and
                len(declaration['body']['branches']) == 18,
                'requiredWork signature or eighteen-branch domain changed')
    mappings = []
    for branch in BRANCHES:
        first = original['declarations'][26]['body']['branches'][branch]['body']
        second = current['declarations'][26]['body']['branches'][branch]['body']
        require(first.get('constructor') == {'name': 'Option.some'} and
                'type_arguments' not in first and len(first['arguments']) == 1,
                'wrong original Option.some expression')
        require(second.get('type_arguments') == [{'kind': 'uint64'}],
                'missing, wrong or additional Option UInt64 annotation')
        expected['declarations'][26]['body']['branches'][branch]['body']['type_arguments'] = [
            {'kind': 'uint64'}]
        del erased['declarations'][26]['body']['branches'][branch]['body']['type_arguments']
        mappings.append({'path': '/declarations/26/body/branches/' + str(branch) + '/body',
                         'constructor': 'Option.some', 'before_node_sha256': sha(canonical(first)),
                         'after_node_sha256': sha(canonical(second)),
                         'classification': 'exact same-class source audit under observed requiredWork stop',
                         'type_arguments': [{'kind': 'uint64'}]})
    require(current == expected, 'computational AST changed outside exact annotations')
    require(new_prefix == prefix and new_suffix == suffix, 'source surface changed')
    require(after == prefix + canonical(expected) + suffix, 'unexpected source bytes')
    require(erased == original and new_prefix + canonical(erased) + new_suffix == before,
            'exact annotation erasure did not recover original complete bytes')
    return current, mappings


IMPLEMENTATION_SHA256 = 'fdb2fd2cd90a2032de8e1943289f8d7e0037dbe1cb98f6849afc52a5e541c6aa'


def check_implementation(source=None):
    source = Path(__file__).read_text() if source is None else source
    tree = ast.parse(source)
    require(not any(isinstance(n, ast.Assert) for n in ast.walk(tree)), 'removable assertion')
    seals = [n for n in tree.body if isinstance(n, ast.Assign) and len(n.targets) == 1 and
             isinstance(n.targets[0], ast.Name) and n.targets[0].id == 'IMPLEMENTATION_SHA256']
    require(len(seals) == 1, 'wrong gate seal inventory')
    seals[0].value = ast.Constant(value='SELF')
    require(sha(ast.dump(tree, include_attributes=False).encode()) == IMPLEMENTATION_SHA256,
            'gate implementation omitted, filtered, stale or changed')


def main():
    require(sys.flags.optimize == 0 and __debug__, 'optimized Python is not allowed')
    require(len(sys.argv) == 1, 'filters and arguments are not allowed')
    require(Path(__file__).name == GATE, 'wrong gate filename')
    check_implementation()
    before, after = git('show', BASELINE + ':' + SOURCE), (ROOT / SOURCE).read_bytes()
    current, mapping = check(before, after)
    entries = git('ls-tree', '-rz', BASELINE, '--', 'src').split(b'\0')
    frozen = {}
    theorem_count = 0
    for entry in entries:
        if not entry:
            continue
        info, name = entry.split(b'\t', 1)
        path = name.decode()
        if path == SOURCE:
            continue
        raw = (ROOT / path).read_bytes()
        require(raw == git('show', BASELINE + ':' + path), 'other source changed: ' + path)
        frozen[path] = sha(raw)
        if path.endswith(('OwnedEnumerationModel.lex.tex', 'OwnedEnumerationProofs.lex.tex')):
            theorem_count += sum(d['kind'] == 'theorem' for d in parse(raw)[0]['declarations'])
    require(theorem_count == 63, 'original complete theorem inventory changed')
    require(not inventory(parse((ROOT / STREAM).read_bytes())[0]),
            'Stream has unhandled Option construction terms')
    rejected = []
    def reject(label, mutate, diagnostic):
        candidate = copy.deepcopy(current)
        mutate(candidate)
        _, prefix, suffix = parse(after)
        try:
            check(before, prefix + canonical(candidate) + suffix)
        except ValueError as error:
            require(diagnostic in str(error), 'wrong failure for ' + label)
            rejected.append({'defect': label, 'diagnostic': str(error)})
        else:
            raise ValueError('source defect escaped: ' + label)
        check(before, after)
    for branch in BRANCHES:
        for defect in ('missing', 'wrong', 'extra'):
            def mutate(node, branch=branch, defect=defect):
                target = node['declarations'][26]['body']['branches'][branch]['body']
                if defect == 'missing':
                    del target['type_arguments']
                elif defect == 'wrong':
                    target['type_arguments'] = [{'kind': 'nat'}]
                else:
                    target['type_arguments'].append({'kind': 'uint64'})
            reject(str(branch) + '-' + defect, mutate, 'Option UInt64 annotation')
    reject('changed-payload', lambda n: n['declarations'][26]['body']['branches'][0]['body']
           ['arguments'].__setitem__(0, {'kind': 'integer', 'representation': 'uint64', 'value': '0'}),
           'computational AST changed')
    reject('changed-return-type', lambda n: n['declarations'][26]['result'].update(value={'kind': 'nat'}),
           'signature or eighteen-branch domain changed')
    reject('omitted-phase-branch', lambda n: n['declarations'][26]['body']['branches'].pop(),
           'signature or eighteen-branch domain changed')
    require(len(rejected) == 51, 'source mutation inventory incomplete')
    process_controls = []
    def process(label, command, diagnostic):
        result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        require(result.returncode != 0 and result.stdout == '' and diagnostic in result.stderr,
                'process control escaped: ' + label)
        process_controls.append({'defect': label, 'returncode': result.returncode,
                                 'diagnostic': diagnostic})
    process('optimized', [sys.executable, '-B', '-O', str(Path(__file__).resolve())],
            'optimized Python is not allowed')
    process('filtered', [sys.executable, '-B', str(Path(__file__).resolve()), '--only=requiredWork'],
            'filters and arguments are not allowed')
    source = Path(__file__).read_text()
    with tempfile.TemporaryDirectory(prefix='uorc-option-gate-') as directory:
        target = Path(directory) / GATE
        for label, old, new in (
                ('omit-source-check', '    current, mapping = check(before, after)\n',
                 '    current, mapping = parse(after)[0], []\n'),
                ('filter-branches', '    for branch in BRANCHES:\n',
                 '    for branch in BRANCHES[:1]:\n')):
            require(old in source, 'missing implementation control target')
            target.write_text(source.replace(old, new))
            process(label, [sys.executable, '-B', str(target)],
                    'gate implementation omitted, filtered, stale or changed')
    require((ROOT / SOURCE).read_bytes() == after, 'source changed during audit')
    check(before, after)
    print(json.dumps({'schema': 'uorc/enumeration-option-annotation-audit/1',
        'status': 'pass-exact-source-erasure-audit-only', 'baseline_commit': BASELINE,
        'source_path': SOURCE, 'before_sha256': sha(before), 'after_sha256': sha(after),
        'gate_sha256': sha(Path(__file__).read_bytes()), 'site_type_hash_map': mapping,
        'exact_sixteen_site_erasure_recovers_original_bytes': True,
        'unchanged_other_source_count': len(frozen), 'unchanged_other_source_sha256': frozen,
        'unchanged_original_complete_theorem_count': theorem_count,
        'stream_option_construction_term_count': 0, 'rejected_source_ir_mutants': rejected,
        'restored_exact_delta_pass': True, 'rejected_process_controls': process_controls,
        'qualification': {'official_source_admission': False, 'broad_runtime_rerun': False,
            'kernel_or_axiom_audit': False, 'native_execution': False,
            'full_sdk_acceptance': False}}, sort_keys=True, indent=2))


if __name__ == '__main__':
    main()
