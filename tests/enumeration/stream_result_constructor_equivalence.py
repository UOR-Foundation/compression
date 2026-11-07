#!/usr/bin/env python3
"""Four Stream Result annotations only; exact source audit, not qualification.

Erasing exactly the four authorized arrays must recover every original source
byte. No evaluator, compiler, SDK, bootstrap, runtime or proof runner is used.
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
BASELINE = 'a78af7c622cd787d8c1838fb7a369f04248fe9bf'
BASELINE_TREE = '54858e784819b10c92a3f50c297898a272d6cfad'
MODULE = 'Uorc.Search.EnumerationStream'
SOURCE = 'src/Uorc/Search/EnumerationStream.lex.tex'
GATE = 'tests/enumeration/stream_result_constructor_equivalence.py'
BEFORE_SHA256 = '6c0c3275c0670e5ecb61d51457cee9da46a3f3e455df61f3d1d634cfb5cffd02'
AFTER_SHA256 = 'f45eb69da4831cecaaf282310b899c83b1b35098c4b0b97ea7d6e32172b95d6b'
SITES = (
    ('start', ('declarations', 11, 'body', 'then_value'), 'Result.ok'),
    ('start', ('declarations', 11, 'body', 'else_value'), 'Result.error'),
    ('acknowledge', ('declarations', 23, 'body', 'then_value'), 'Result.ok'),
    ('acknowledge', ('declarations', 23, 'body', 'else_value'), 'Result.error'),
)
PROOFS = (
    ('src/Uorc/Search/OwnedEnumerationModel.lex.tex', 45),
    ('src/Uorc/Search/OwnedEnumerationProofs.lex.tex', 18),
)
PRODUCER_RECEIPT = 'docs/implementation/owned-enumeration-result-constructor-evidence.json'
RUNTIME_RECEIPT = 'docs/implementation/enumeration-stream-phase-comparison-evidence.json'
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


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def canonical(node):
    return json.dumps(node, sort_keys=True, separators=(',', ':')).encode()


def at(node, path):
    for key in path:
        node = node[key]
    return node


def pointer(path):
    return '/' + '/'.join(map(str, path))


def unique_object(pairs):
    node = {}
    for key, value in pairs:
        require(key not in node, 'duplicate semantic JSON key: ' + key)
        node[key] = value
    return node


def parse(raw, module=None):
    text = raw.decode('utf-8')
    marker = r'\semanticdata{'
    require(text.count(marker) == 1, 'wrong semanticdata inventory')
    if module is not None:
        require(text.startswith(r'\begin{lexlean}{' + module + '}\n'), 'wrong source module')
    start = text.index(marker) + len(marker)
    node, length = json.JSONDecoder(object_pairs_hook=unique_object).raw_decode(text[start:])
    require(text[start:start + length].encode() == canonical(node), 'noncanonical source JSON')
    require(node.get('spec') == 'lexlean/semantic-module/1', 'wrong semantic schema')
    names = [d['name'] for d in node['declarations']]
    require(len(names) == len(set(names)), 'duplicate declaration')
    return node, text[:start].encode(), text[start + length:].encode()


def git(*arguments):
    return subprocess.run(['git', '-C', str(ROOT), *arguments], check=True,
                          stdout=subprocess.PIPE, stderr=subprocess.PIPE).stdout


def git_file(path):
    return git('show', BASELINE + ':' + path)


def named(name):
    return {'kind': 'named', 'member': {'name': name}, 'arguments': []}


def result_pair():
    return [named('Cursor'), named('Error')]


def constructors(node, path=()):
    found = []
    if isinstance(node, dict):
        if node.get('kind') == 'constructor' and node.get('constructor', {}).get('name') in (
                'Result.ok', 'Result.error'):
            found.append(path)
        for key, child in node.items():
            found.extend(constructors(child, path + (key,)))
    elif isinstance(node, list):
        for key, child in enumerate(node):
            found.extend(constructors(child, path + (key,)))
    return found


def check_source(before, after):
    require(sha(before) == BEFORE_SHA256, 'wrong frozen baseline Stream bytes')
    old, prefix, suffix = parse(before, MODULE)
    current, current_prefix, current_suffix = parse(after, MODULE)
    require(len(old['declarations']) == 38, 'wrong complete Stream declaration inventory')
    allowed = {site[1] for site in SITES}
    require(len(allowed) == 4 and set(constructors(old)) == allowed,
            'baseline Result constructor inventory is not exactly the four authorized sites')
    require(set(constructors(current)) == allowed,
            'current Result constructor inventory is not exactly the four authorized sites')
    expected, erased, mapping = copy.deepcopy(old), copy.deepcopy(current), []
    for name, path, constructor in SITES:
        original, updated = at(old, path), at(current, path)
        declaration, changed = old['declarations'][path[1]], current['declarations'][path[1]]
        require(declaration['name'] == name and changed['name'] == name,
                'wrong declaration at authorized site')
        require(original.get('kind') == 'constructor' and
                original.get('constructor') == {'name': constructor} and
                'type_arguments' not in original, 'wrong original Result constructor shape')
        pair = result_pair()
        require(updated.get('type_arguments') == pair,
                'wrong, swapped, missing or extra Result type pair: ' + pointer(path))
        require(updated.get('constructor') == {'name': constructor}, 'Result constructor changed')
        result_type = {'kind': 'result', 'ok': pair[0], 'error': pair[1]}
        require(declaration['result'] == result_type and changed['result'] == result_type,
                'declared Result return type differs from annotation: ' + pointer(path))
        at(expected, path)['type_arguments'] = copy.deepcopy(pair)
        del at(erased, path)['type_arguments']
        mapping.append({
            'declaration': name, 'site': pointer(path), 'constructor': constructor,
            'classification': 'static same-class source finding; not an independent compiler stop',
            'declared_result_type': result_type,
            'before_type_arguments': None, 'after_type_arguments': pair,
            'before_node_sha256': sha(canonical(original)),
            'after_node_sha256': sha(canonical(updated)),
            'payload_sha256': sha(canonical(original['arguments'])),
            'before_declaration_sha256': sha(canonical(declaration)),
            'after_declaration_sha256': sha(canonical(changed)),
        })
    require(current == expected, 'Stream AST changed outside the four annotations')
    require(current_prefix == prefix and current_suffix == suffix, 'Stream source surface changed')
    require(after == prefix + canonical(expected) + suffix, 'unexpected Stream source bytes')
    require(erased == old and current_prefix + canonical(erased) + current_suffix == before,
            'four-site erasure did not recover complete original Stream bytes')
    require(sha(after) == AFTER_SHA256, 'wrong final Stream source hash')
    return current, mapping


def check_frozen_file(path, original, current):
    require(current == original, 'unrelated baseline source, policy or evidence changed: ' + path)


def check_frozen_repository():
    require(git('rev-parse', BASELINE + '^{tree}').decode().strip() == BASELINE_TREE,
            'wrong baseline tree')
    entries = git('ls-tree', '-r', '-z', BASELINE).split(b'\0')
    manifest = {}
    for entry in entries:
        if not entry:
            continue
        metadata, raw_path = entry.split(b'\t', 1)
        mode, kind, _ = metadata.split()
        path = raw_path.decode()
        require(kind == b'blob' and mode in (b'100644', b'100755'), 'unexpected baseline file kind')
        require(path not in manifest, 'duplicate baseline path')
        if path == SOURCE:
            continue
        original, current = git_file(path), (ROOT / path).read_bytes()
        check_frozen_file(path, original, current)
        manifest[path] = sha(current)
    require(len(manifest) == len(entries) - 2 and len(manifest) > 500,
            'baseline file inventory was filtered or omitted')
    extra = git('ls-files', '--others', '--exclude-standard', '--', 'src', 'model', 'standards',
                'bootstrap', '.github').decode().splitlines()
    require(not extra, 'unrequested new source or policy files')
    return manifest


def check_proofs(manifest):
    result = []
    for path, count in PROOFS:
        raw = (ROOT / path).read_bytes()
        require(sha(raw) == manifest[path], 'proof root is not frozen')
        node = parse(raw)[0]
        theorems = [d for d in node['declarations'] if d['kind'] == 'theorem']
        require(len(theorems) == count, 'wrong complete original theorem inventory')
        result.append({'path': path, 'source_sha256': sha(raw), 'theorem_count': count,
                       'complete_theorems': [{'name': d['name'],
                                             'full_theorem_sha256': sha(canonical(d))}
                                            for d in theorems]})
    require(sum(item['theorem_count'] for item in result) == 63, '63-theorem inventory omitted')
    return result


def check_repository_evidence(manifest):
    producer_raw = (ROOT / PRODUCER_RECEIPT).read_bytes()
    producer = json.loads(producer_raw)
    require(sha(producer_raw) == manifest[PRODUCER_RECEIPT], 'producer receipt not frozen')
    require(producer['compiler_source_audit'] == COMPILER_AUDIT, 'compiler schema audit changed')
    producer_path = 'src/Uorc/Search/OwnedEnumeration.lex.tex'
    require(manifest[producer_path] == producer['after_sha256'] ==
            'a736436cc5c54a4740d1660a0d06f8b5b548ed5051f356d29de22c1e4117cdd0',
            'published annotated producer not preserved')
    path = 'src/Uorc/Core/ByteBuilder.lex.tex'
    raw = (ROOT / path).read_bytes()
    require(sha(raw) == manifest[path], 'schema examples not frozen')
    declaration = parse(raw)[0]['declarations'][7]
    pair = [named('Builder'), named('Error')]
    require(declaration['name'] == 'start' and declaration['result'] == {
        'kind': 'result', 'ok': pair[0], 'error': pair[1]}, 'wrong schema example return type')
    examples = []
    for branch, constructor in (('then_value', 'Result.error'), ('else_value', 'Result.ok')):
        term = declaration['body'][branch]
        require(term['kind'] == 'constructor' and term['constructor'] == {'name': constructor} and
                term['type_arguments'] == pair, 'wrong success/error annotation schema example')
        examples.append({'path': path, 'source_sha256': sha(raw),
                         'site': '/declarations/7/body/' + branch, 'constructor': constructor,
                         'type_arguments': pair, 'node_sha256': sha(canonical(term))})
    require(len(examples) == 2, 'schema example inventory omitted')
    runtime_raw = (ROOT / RUNTIME_RECEIPT).read_bytes()
    runtime = json.loads(runtime_raw)
    require(sha(runtime_raw) == manifest[RUNTIME_RECEIPT] and
            runtime['current_source_sha256'] == BEFORE_SHA256,
            'historical Stream runtime receipt was relabelled')
    return {'compiler_audit_provenance': PRODUCER_RECEIPT,
            'compiler_audit_provenance_sha256': sha(producer_raw),
            'compiler_source_audit': COMPILER_AUDIT,
            'audit_repeated_by_this_gate': False,
            'unchanged_repository_schema_examples': examples,
            'historical_runtime_receipt': RUNTIME_RECEIPT,
            'historical_runtime_receipt_sha256': sha(runtime_raw),
            'historical_runtime_source_sha256': BEFORE_SHA256,
            'historical_runtime_rerun_or_relabelled': False}


def source_mutations(before, after, current):
    _, prefix, suffix = parse(after, MODULE)
    rejected = []

    def reject(label, mutate, diagnostic):
        candidate = copy.deepcopy(current)
        mutate(candidate)
        raw = prefix + canonical(candidate) + suffix
        require(raw != after, 'planted source mutation did not change bytes')
        try:
            check_source(before, raw)
        except ValueError as error:
            require(diagnostic in str(error), 'wrong source mutation diagnostic: ' + str(error))
            rejected.append({'defect': label, 'mutant_source_sha256': sha(raw),
                             'diagnostic': str(error)})
        else:
            raise ValueError('source mutation escaped: ' + label)
        check_source(before, after)

    for index, (_, path, _) in enumerate(SITES):
        for defect in ('missing', 'swapped', 'wrong-ok', 'wrong-error', 'one', 'extra',
                       'qualified-ok', 'parameterized-error'):
            def mutate(node, path=path, defect=defect):
                term = at(node, path)
                if defect == 'missing':
                    del term['type_arguments']
                elif defect == 'swapped':
                    term['type_arguments'].reverse()
                elif defect == 'wrong-ok':
                    term['type_arguments'][0] = {'kind': 'unit'}
                elif defect == 'wrong-error':
                    term['type_arguments'][1] = {'kind': 'unit'}
                elif defect == 'one':
                    term['type_arguments'].pop()
                elif defect == 'extra':
                    term['type_arguments'].append({'kind': 'unit'})
                elif defect == 'qualified-ok':
                    term['type_arguments'][0]['member']['module'] = 'Uorc.Search.OwnedEnumeration'
                else:
                    term['type_arguments'][1]['arguments'].append({'kind': 'unit'})
            reject(str(index) + '-' + defect + '-pair', mutate,
                   'wrong, swapped, missing or extra Result type pair')
    reject('change-start-ticket-payload',
           lambda n: at(n, SITES[0][1])['arguments'][0]['fields'][0]['value'].update(name='enumeration'),
           'Stream AST changed outside')
    reject('change-start-error',
           lambda n: at(n, SITES[1][1])['arguments'][0]['constructor'].update(name='Error.invalidOffer'),
           'Stream AST changed outside')
    reject('change-acknowledge-custody-phase',
           lambda n: at(n, SITES[2][1])['arguments'][0]['fields'][1]['value']['constructor'].update(
               name='Phase.cancelled'), 'Stream AST changed outside')
    reject('change-acknowledge-error',
           lambda n: at(n, SITES[3][1])['arguments'][0]['constructor'].update(name='Error.invalidSource'),
           'Stream AST changed outside')
    for index in (11, 23):
        reject('change-declared-return-type-' + str(index),
               lambda n, i=index: n['declarations'][i]['result'].update(ok={'kind': 'unit'}),
               'declared Result return type differs')
    reject('reorder-condition-arguments',
           lambda n: n['declarations'][11]['body']['condition']['arguments'].reverse(),
           'Stream AST changed outside')
    reject('change-parameter-binder',
           lambda n: n['declarations'][23]['parameters'][1].update(name='other'),
           'Stream AST changed outside')
    reject('change-phase-predicate',
           lambda n: n['declarations'][14]['body']['left']['branches'][4]['body'].update(value=False),
           'Stream AST changed outside')
    reject('change-payment-charge',
           lambda n: n['declarations'][18]['body']['branches'][0].update(body={
               'kind': 'integer', 'representation': 'uint64', 'value': '0'}),
           'Stream AST changed outside')
    reject('change-match-pattern-binder',
           lambda n: n['declarations'][36]['body']['branches'][0].update(binders=['other']),
           'Stream AST changed outside')
    reject('add-unrequested-match-pattern-annotation',
           lambda n: n['declarations'][36]['body']['branches'][0].update(type_arguments=result_pair()),
           'Stream AST changed outside')
    reject('change-declaration-axiom-policy',
           lambda n: n['declarations'][11]['axioms'].append('Classical.choice'),
           'Stream AST changed outside')
    reject('reorder-success-record-fields',
           lambda n: at(n, SITES[2][1])['arguments'][0]['fields'].reverse(),
           'Stream AST changed outside')
    require(len(rejected) == 46, 'source mutation inventory omitted')
    altered_prefix = prefix.replace(b'\\importmodule{Uorc.Search.OwnedEnumeration}',
                                    b'\\importmodule{Uorc.Search.OwnedEnumerationModel}', 1)
    raw = altered_prefix + canonical(current) + suffix
    require(raw != after, 'import mutation did not change bytes')
    try:
        check_source(before, raw)
    except ValueError as error:
        require('Stream source surface changed' in str(error), 'wrong import mutation diagnostic')
        rejected.append({'defect': 'change-import', 'mutant_source_sha256': sha(raw),
                         'diagnostic': str(error)})
    else:
        raise ValueError('import mutation escaped')
    check_source(before, after)
    require(len(rejected) == 47, 'complete source mutation inventory omitted')
    return rejected


def frozen_mutations():
    controls = []
    for path in (PROOFS[0][0], PROOFS[1][0], 'src/Uorc/Search/OwnedEnumeration.lex.tex',
                 'src/Uorc/Search/UnsignedByteConversion.lex.tex', 'model/claim-policy.toml',
                 'lexlean.toml', RUNTIME_RECEIPT):
        original = git_file(path)
        if path in (PROOFS[0][0], PROOFS[1][0]):
            node, prefix, suffix = parse(original)
            theorem = next(d for d in node['declarations'] if d['kind'] == 'theorem')
            theorem['axioms'].append('Classical.choice')
            candidate = prefix + canonical(node) + suffix
            defect = 'change-complete-theorem-axiom-policy'
        else:
            candidate = original + b'\n'
            defect = 'change-frozen-source-policy-or-receipt-bytes'
        try:
            check_frozen_file(path, original, candidate)
        except ValueError as error:
            require('unrelated baseline source, policy or evidence changed' in str(error),
                    'wrong frozen-file mutation diagnostic')
            controls.append({'path': path, 'defect': defect,
                             'mutant_sha256': sha(candidate), 'diagnostic': str(error)})
        else:
            raise ValueError('frozen-file mutation escaped: ' + path)
        check_frozen_file(path, original, original)
    require(len(controls) == 7, 'frozen-file controls omitted')
    return controls


IMPLEMENTATION_SHA256 = '6b19b8c33ec29765554895e3b3c16aaeb5dced3e2c7f8a89e65df51450ad80fd'


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
                'process control escaped or failed for wrong reason: ' + label)
        controls.append({'defect': label, 'returncode': result.returncode, 'diagnostic': diagnostic})

    reject('optimized-python', [sys.executable, '-B', '-O', str(script)],
           'optimized Python is not allowed')
    reject('filtered-process', [sys.executable, '-B', str(script), '--only=start'],
           'filters and other arguments are not allowed')
    source = script.read_text()
    mutants = (
        ('omit-source-check', '    current, mapping = check_source(before, after)\n',
         '    current, mapping = parse(after)[0], []\n'),
        ('omit-frozen-repository-check', '    manifest = check_frozen_repository()\n',
         '    manifest = {}\n'),
        ('omit-full-proof-inventory', '    proofs = check_proofs(manifest)\n',
         '    proofs = []\n'),
        ('omit-schema-examples', '    evidence = check_repository_evidence(manifest)\n',
         '    evidence = {}\n'),
        ('omit-source-mutations', '    negatives = source_mutations(before, after, current)\n',
         '    negatives = []\n'),
        ('omit-frozen-mutations', '    frozen_controls = frozen_mutations()\n',
         '    frozen_controls = []\n'),
        ('select-wrong-source-file', "SOURCE = 'src/Uorc/Search/EnumerationStream.lex.tex'\n",
         "SOURCE = 'src/Uorc/Search/OwnedEnumeration.lex.tex'\n"),
        ('omit-final-source-reread', "    require((ROOT / SOURCE).read_bytes() == after, 'Stream changed during gate')\n",
         '    pass\n'),
    )
    with tempfile.TemporaryDirectory(prefix='uorc-stream-result-controls-') as temporary:
        target = Path(temporary) / 'tests' / 'enumeration' / script.name
        target.parent.mkdir(parents=True)
        for label, needle, replacement in mutants:
            require(source.count(needle) == 1, 'ambiguous gate mutation: ' + label)
            target.write_text(source.replace(needle, replacement, 1))
            reject(label, [sys.executable, '-B', str(target)],
                   'gate implementation skipped, filtered, stale or changed')
        wrong_file = target.with_name('wrong_stream_gate.py')
        wrong_file.write_text(source)
        reject('wrong-gate-file', [sys.executable, '-B', str(wrong_file)],
               'wrong gate file')
    require(len(controls) == 11, 'process controls omitted')
    return controls


def main():
    require(sys.flags.optimize == 0 and __debug__, 'optimized Python is not allowed')
    require(len(sys.argv) == 1, 'filters and other arguments are not allowed')
    check_implementation()
    require(Path(__file__).resolve() == ROOT / GATE, 'wrong gate file')
    before, after = git_file(SOURCE), (ROOT / SOURCE).read_bytes()
    current, mapping = check_source(before, after)
    manifest = check_frozen_repository()
    proofs = check_proofs(manifest)
    evidence = check_repository_evidence(manifest)
    negatives = source_mutations(before, after, current)
    frozen_controls = frozen_mutations()
    controls = process_controls()
    check_implementation()
    require((ROOT / SOURCE).read_bytes() == after, 'Stream changed during gate')
    for path, digest in manifest.items():
        require(sha((ROOT / path).read_bytes()) == digest, 'frozen file changed during gate: ' + path)
    check_source(before, after)
    print(json.dumps({
        'schema': 'uorc/stream-result-constructor-repair/1',
        'status': 'pass-exact-source-annotation-audit-only',
        'baseline_commit': BASELINE, 'baseline_tree': BASELINE_TREE,
        'source_path': SOURCE, 'before_sha256': sha(before), 'after_sha256': sha(after),
        'gate_path': GATE, 'gate_sha256': sha(Path(__file__).read_bytes()),
        'result_constructor_term_count': 4, 'static_site_type_hash_map': mapping,
        'exact_four_site_erasure_recovers_complete_original_bytes': True,
        'all_other_stream_ast_and_surface_bytes_unchanged': True,
        'preserved_complete_theorems': proofs, 'complete_original_theorem_count': 63,
        'frozen_file_count': len(manifest), 'frozen_file_manifest_sha256': sha(canonical(manifest)),
        'frozen_file_sha256': manifest, 'supporting_evidence': evidence,
        'rejected_source_ir_and_surface_mutants': negatives,
        'rejected_frozen_source_policy_proof_receipt_mutants': frozen_controls,
        'rejected_real_process_controls': controls, 'restored_exact_delta_pass': True,
        'qualification': {
            'independently_observed_stream_compiler_error': False,
            'official_source_check_run': False, 'official_source_admission': False,
            'compiler_or_kernel_or_axiom_audit_run': False, 'broad_runtime_rerun': False,
            'native_execution': False, 'full_sdk_acceptance': False,
            'conversion_laws_proved': False, 'historical_receipts_relabelled': False,
        },
    }, sort_keys=True, indent=2))


if __name__ == '__main__':
    main()
