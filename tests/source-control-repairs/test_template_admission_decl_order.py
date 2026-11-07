"""Check actual TemplateAdmission source order with four mandatory compiler gates.

The small order manifest supplies permutations. All declaration bodies, types,
proofs, policies, imports and source wrappers come from the prepared project.
The official SDK reads, writes and renders each source variant.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import time

MODULE = Path('src/Uorc/Search/TemplateAdmission.lex.tex')
EXPECTED = 'phase link: forward or missing function `joinSpanPaid`'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def render_variants(helper, source, manifest, output):
    receipts = []

    def sdk(action, text):
        directory = output / 'sdk' / ('%02d-%s' % (len(receipts), action))
        directory.mkdir(parents=True)
        (directory / 'stdin').write_text(text)
        command = [str(helper), action]
        result = subprocess.run(command, input=text, text=True, capture_output=True, timeout=60)
        (directory / 'stdout').write_text(result.stdout)
        (directory / 'stderr').write_text(result.stderr)
        receipts.append({'command': command, 'returncode': result.returncode, 'stdin_sha256': digest(text.encode()), 'stdout_sha256': digest(result.stdout.encode())})
        (output / 'sdk-commands.json').write_text(json.dumps(receipts, indent=2) + '\n')
        if result.returncode != 0:
            raise AssertionError(result.stderr)
        return result.stdout

    actual = json.loads(sdk('read', source))
    assert actual['spec'] in ('lexlean/semantic-module/1', 'lexlean/semantic-module/2')
    declarations = actual['declarations']
    by_name = {declaration['name']: declaration for declaration in declarations}
    assert len(by_name) == len(declarations) == 43
    assert [declaration['name'] for declaration in declarations] == manifest['repaired_order']
    assert set(manifest['original_order']) == set(manifest['repaired_order']) == set(by_name)
    variants = {}
    for name, order in [('original', manifest['original_order']), ('repaired', manifest['repaired_order'])]:
        value = copy.deepcopy(actual)
        value['declarations'] = [copy.deepcopy(by_name[key]) for key in order]
        variants[name] = value
    mutant = copy.deepcopy(variants['repaired'])
    names = [declaration['name'] for declaration in mutant['declarations']]
    left, right = (names.index(name) for name in manifest['mutant_swap'])
    assert right == left + 1
    mutant['declarations'][left], mutant['declarations'][right] = mutant['declarations'][right], mutant['declarations'][left]
    variants['mutant'] = mutant
    position = {declaration['name']: index for index, declaration in enumerate(variants['repaired']['declarations'])}

    def inspect(term, caller):
        if isinstance(term, dict):
            if term.get('kind') == 'call' and not term['function'].get('module'):
                callee = term['function']['name']
                if callee in position and callee != caller:
                    assert position[callee] < position[caller], (caller, callee)
            for child in term.values():
                inspect(child, caller)
        elif isinstance(term, list):
            for child in term:
                inspect(child, caller)

    for declaration in variants['repaired']['declarations']:
        inspect(declaration, declaration['name'])
    rendered = {}
    for name, value in variants.items():
        assert {declaration['name']: declaration for declaration in value['declarations']} == by_name
        assert value['declarations'][:9] == actual['declarations'][:9]
        assert {key: item for key, item in value.items() if key != 'declarations'} == {key: item for key, item in actual.items() if key != 'declarations'}
        result = sdk('write', json.dumps({'source': source, 'value': value}, separators=(',', ':')))
        # A schema-2 data block is not valid language-1.2 source. Always use
        # the official notation renderer, even when write preserved notation.
        if value['spec'] == 'lexlean/semantic-module/2':
            result = sdk('to-notation', result)
            assert '\\semanticdata' not in result
        assert json.loads(sdk('read', result)) == value
        rendered[name] = result
        (output / (name + '.lex.tex')).write_text(result)
    audit = {'status': 'PASS', 'declaration_count': 43, 'all_declaration_values_preserved': True, 'inverse_order_restores_original_ir': True, 'no_repaired_forward_calls': True, 'schema': actual['spec'], 'schema2_official_to_notation_calls': sum(receipt['command'][-1] == 'to-notation' for receipt in receipts), 'source_sha256': {key: digest(value.encode()) for key, value in rendered.items()}}
    assert audit['schema2_official_to_notation_calls'] == (3 if actual['spec'].endswith('/2') else 0)
    (output / 'source-self-audit.json').write_text(json.dumps(audit, indent=2) + '\n')
    return rendered


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--project', type=Path, required=True)
    parser.add_argument('--compiler', type=Path, required=True)
    parser.add_argument('--semantic-helper', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--order-manifest', type=Path, default=Path(__file__).with_name('template_admission_order.json'))
    args = parser.parse_args()
    project, compiler, helper = args.project.resolve(), args.compiler.resolve(), args.semantic_helper.resolve()
    output = args.output.resolve()
    output.mkdir(parents=True)
    manifest = json.loads(args.order_manifest.read_text())
    assert manifest['schema'] == 'uorc/declaration-order-regression/1'
    assert manifest['module'] == 'Uorc.Search.TemplateAdmission'
    source = (project / MODULE).read_text()
    rendered = render_variants(helper, source, manifest, output)
    setup_names = ('lexlean.toml', 'lexlean.lock', 'lean-toolchain', 'lakefile.toml', 'lake-manifest.json')
    source_hashes = {str(path.relative_to(project)): digest(path.read_bytes()) for path in project.glob('src/**/*.lex.tex')}
    setup_hashes = {name: digest((project / name).read_bytes()) for name in setup_names}
    receipts = []
    with tempfile.TemporaryDirectory(prefix='admission-order-', dir=output) as temporary:
        isolated = Path(temporary)
        shutil.copytree(project / 'src', isolated / 'src')
        for name in setup_names:
            shutil.copyfile(project / name, isolated / name)
        gates = [('original-negative', 'check', 'original', False), ('repaired-check', 'check', 'repaired', True), ('repaired-build', 'build', 'repaired', True), ('swapped-order-negative', 'check', 'mutant', False)]
        try:
            for label, action, fixture, should_pass in gates:
                (isolated / MODULE).write_text(rendered[fixture])
                command = [str(compiler), '--project', str(isolated / 'lexlean.toml'), '--diagnostic-format', 'json', action, str(MODULE)]
                started = time.monotonic()
                result = subprocess.run(command, cwd=isolated, text=True, capture_output=True, timeout=60)
                directory = output / label
                directory.mkdir()
                (directory / 'stdout').write_text(result.stdout)
                (directory / 'stderr').write_text(result.stderr)
                receipt = {'command': command, 'returncode': result.returncode, 'elapsed_seconds': time.monotonic() - started, 'source_sha256': digest(rendered[fixture].encode())}
                (directory / 'command.json').write_text(json.dumps(receipt, indent=2) + '\n')
                response = json.loads(result.stdout)
                if should_pass:
                    assert result.returncode == 0 and response['success'], response
                else:
                    assert result.returncode != 0
                    assert any(item.get('code') == 'LLT4001' and item.get('message') == EXPECTED for item in response['diagnostics']), response
                receipt.update({key: response[key] for key in ('source_id', 'semantic_id', 'build_id') if key in response})
                receipts.append(receipt)
        finally:
            (isolated / MODULE).write_text(rendered['repaired'])
            assert (isolated / MODULE).read_text() == rendered['repaired']
            assert all(digest((isolated / name).read_bytes()) == value for name, value in setup_hashes.items())
        assert len(receipts) == 4
    assert source_hashes == {str(path.relative_to(project)): digest(path.read_bytes()) for path in project.glob('src/**/*.lex.tex')}
    assert setup_hashes == {name: digest((project / name).read_bytes()) for name in setup_names}
    receipt = {'gates': receipts, 'all_four_required_gates_completed': True, 'source_self_audit': 'PASS', 'no_skips': True, 'input_project_sources_and_setup_unchanged': True, 'compiler_sha256': digest(compiler.read_bytes()), 'helper_sha256': digest(helper.read_bytes())}
    (output / 'PASS.json').write_text(json.dumps(receipt, indent=2) + '\n')


if __name__ == '__main__':
    main()
