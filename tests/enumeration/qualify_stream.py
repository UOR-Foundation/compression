#!/usr/bin/env python3
"""Exact base/current-Core source unions for finite stream custody diagnostics."""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tarfile
import tempfile
import qualify_current_core as core

BASE = '9ca824709b0c6ffe4dbb647cb11a050b04247c19'
WIRE_SOURCE = 'd27be3e8f7dba73440ec29fa8dba236275162ca7'
WIRE_CACHE = 'f3d3f619cad5f60c632201dae0a7f3b854226769'
WIRE_CACHE_PATH = 'tests/heap/semantic-cache/wire-native-d27be3e.json'
WIRE_HASH = '58e49bc558d38d0414b69a3e27c0380dfaf0f768fce968cb271a8f5f1637ee88'
WIRE_LOCAL_PATH = 'tests/enumeration/semantic-cache/wire-native-d27be3e.json'
EXPECTED_FILES = {
    'src/Uorc/Search/EnumerationStream.lex.tex': 'cfa09481e6d2a18e22752b91217d286913f4f5a22bb10626b20be22012c7225c',
    'tests/enumeration/stream_diagnostic_test.py': '0a85ee4649ed1ade3755b14d1f870cc0d0b52ccf2a696957f1806701a1eefac7',
    'tests/enumeration/stream_mutations.py': '6c345645cacf95558262be2660643ed6a868ed070c53379d1ca949507a2b2ac8',
    WIRE_LOCAL_PATH: WIRE_HASH,
}
CONTROL_LINES = ['PASS stream bytes ' + data.hex() for data in
                 [b'', b'\x00', b'\xff', b'\x00\x00', b'\x00\xff\xff', b'\xff\x00\xff',
                  bytes(range(17)), bytes([0, 255]) * 17]] + [
    'PASS all stream custody prefixes 111 spans 11',
    'PASS funded partial-prepayment cancellation forks 114',
    'PASS defensive post-observer refusal retains real achieved allocation and drains',
    'PASS typed source, reader, resource, heap, offer, funding, span and overflow boundaries',
]
MUTANT_LINES = [
    'PASS original stream mutation probes',
    'REJECTED missing-image-lease offered image lacks its retained lease root',
    'REJECTED unpaid-event unpaid stream event executed',
    'REJECTED stale-generation stale stream generation acknowledged',
    'REJECTED wrong-position wrong logical span position acknowledged',
    'PASS all four stream mutants rejected at intended assertions and restored',
]


def verify_result(name, result):
    core.require(result['returncode'] == 0, name + ': command failed')
    core.require(not result['stderr'], name + ': unexpected stderr')
    expected = {'stream_diagnostic_test.py': CONTROL_LINES, 'stream_mutations.py': MUTANT_LINES}
    core.require(name in expected, 'unregistered stream command')
    core.require(result['stdout'].splitlines() == expected[name],
                 name + ': missing, extra, duplicate, filtered, reordered or wrong semantic result')


def verify_wire(repository, data, source_reader):
    core.require(core.sha(data) == WIRE_HASH, 'wrong authentic Wire cache bytes')
    cache = json.loads(data)
    core.require(cache['schema'] == 'uorc/compiler-parsed-native-wire/1', 'wrong Wire cache schema')
    core.require(cache['source_commit'] == WIRE_SOURCE, 'wrong Wire source checkpoint')
    core.require(cache['qualification'] == {'parser_only': True, 'linked_closure': False,
                                          'kernel_or_export_acceptance': False},
                 'wrong Wire cache qualification boundary')
    modules = cache['modules']
    core.require(len(modules) == 2 and {m['module'] for m in modules} ==
                 {'Uorc.Wire.OwnedRead', 'Uorc.Wire.FrameEventContract'},
                 'missing, duplicate or extra Wire parse modules')
    official = json.loads(core.git(repository, 'show', core.CORE_CACHE + ':' + core.CACHE_PATH))
    core.require(cache['official_reader'] == official['official_reader'],
                 'Wire official-reader provenance differs from frozen authoritative Core cache')
    for module in modules:
        path = module['source_path']
        core.require(path == 'src/' + module['module'].replace('.', '/') + '.lex.tex',
                     'Wire module/source-path mismatch')
        source = source_reader(path)
        core.require(core.sha(source) == module['source_sha256'] and len(source) == module['source_bytes'],
                     path + ': Wire source mismatch')
        blob = core.git(repository, 'rev-parse', WIRE_SOURCE + ':' + path).decode().strip()
        core.require(blob == module['source_git_blob'], path + ': Wire source blob mismatch')
        core.require(sorted(re.findall(r'\\importmodule\{([^}]+)\}', source.decode())) == sorted(module['imports']),
                     path + ': Wire import mismatch')
        core.require(core.sha(core.canonical(module['parsed_module'])) == module['parser_output_sha256'],
                     path + ': Wire parsed-module mismatch')
        core.require(module['parsed_module']['spec'] == 'lexlean/semantic-module/2',
                     path + ': wrong Wire parsed schema')
        core.require(module['repeat_read'] == module['notation_round_trip'] == 'byte_exact',
                     path + ': missing repeat-read/writer equality evidence')
    return cache


def closure(root, module):
    todo = [module]
    result = {}
    while todo:
        name = todo.pop()
        if name in result:
            continue
        path = 'src/' + name.replace('.', '/') + '.lex.tex'
        data = (root / path).read_bytes()
        imports = sorted(re.findall(r'\\importmodule\{([^}]+)\}', data.decode()))
        result[name] = {'path': path, 'sha256': core.sha(data), 'imports': imports}
        todo.extend(imports)
    return dict(sorted(result.items()))


def qualify(repository, authoring):
    wire_bytes = core.git(repository, 'show', WIRE_CACHE + ':' + WIRE_CACHE_PATH)
    wire = verify_wire(repository, wire_bytes,
                       lambda path: core.git(repository, 'show', WIRE_SOURCE + ':' + path))
    core_bytes = core.git(repository, 'show', core.CORE_CACHE + ':' + core.CACHE_PATH)
    frozen_core = core.verify_cache(core_bytes,
                                   lambda path: core.git(repository, 'show', core.CORE_SOURCE + ':' + path))
    contents = {}
    for path, expected in EXPECTED_FILES.items():
        data = (authoring / path).read_bytes()
        core.require(core.sha(data) == expected, path + ': reviewed stream source changed')
        contents[path] = data
    core.require(contents[WIRE_LOCAL_PATH] == wire_bytes, 'local Wire cache differs from public exact bytes')
    archive = core.git(repository, 'archive', '--format=tar', BASE)
    unions = []
    for label in ['reviewed-base-core', 'frozen-current-core-34-source-overlay']:
        with tempfile.TemporaryDirectory(prefix='owned-enumeration-stream-') as directory:
            root = Path(directory)
            with tarfile.open(fileobj=io.BytesIO(archive)) as bundle:
                bundle.extractall(root, filter='data')
            for path, data in contents.items():
                (root / path).parent.mkdir(parents=True, exist_ok=True)
                (root / path).write_bytes(data)
            core_paths = set()
            if label != 'reviewed-base-core':
                for module in frozen_core['modules']:
                    path = module['source_path']
                    (root / path).write_bytes(core.git(repository, 'show', core.CORE_SOURCE + ':' + path))
                    core_paths.add(path)
                (root / core.CACHE_PATH).write_bytes(core_bytes)
            diagnostic = root / 'tests/enumeration/diagnostic.py'
            core.require(core.sha(diagnostic.read_bytes()) == core.EXPECTED_SCRIPTS['diagnostic.py'],
                         'original evaluator changed')
            if core_paths:
                text = diagnostic.read_text()
                core.require(text.count(core.OLD_CACHE_HASH) == 1, 'ambiguous evaluator cache binding')
                diagnostic.write_text(text.replace(core.OLD_CACHE_HASH, core.CACHE_HASH))
            owned_read = next(m for m in wire['modules'] if m['module'] == 'Uorc.Wire.OwnedRead')
            core.require(core.sha((root / owned_read['source_path']).read_bytes()) == owned_read['source_sha256'],
                         'union OwnedRead source is not the exact public source')
            sources = []
            for path in sorted((root / 'src/Uorc').rglob('*.lex.tex')):
                relative = path.relative_to(root).as_posix()
                authority = ('exact new stream source in this receipt' if relative in contents else
                             core.CORE_SOURCE if relative in core_paths else BASE)
                sources.append({'path': relative, 'sha256': core.sha(path.read_bytes()), 'authority': authority})
            results = []
            environment = dict(os.environ, PYTHONDONTWRITEBYTECODE='1')
            environment.pop('PYTHONOPTIMIZE', None)
            environment.pop('PYTHONPATH', None)
            for name in ['stream_diagnostic_test.py', 'stream_mutations.py']:
                process = subprocess.run([sys.executable, '-B', str(diagnostic.parent / name)], cwd=root,
                                         env=environment, capture_output=True, text=True, check=False)
                result = {'command': ['python3', '-B', 'tests/enumeration/' + name],
                          'returncode': process.returncode, 'stdout': process.stdout, 'stderr': process.stderr}
                verify_result(name, result)
                results.append(result)
            unions.append({'name': label, 'base_commit': BASE,
                           'core_overlay_source_commit': core.CORE_SOURCE if core_paths else None,
                           'core_overlay_cache_commit': core.CORE_CACHE if core_paths else None,
                           'core_overlay_sources': len(core_paths), 'source_union': sources,
                           'stream_root_closure': closure(root, 'Uorc.Search.EnumerationStream'),
                           'runtime_cache_sha256': core.sha((root / core.CACHE_PATH).read_bytes()),
                           'evaluator_sha256': core.sha(diagnostic.read_bytes()), 'commands': results})
    return {'schema': 'uorc/owned-enumeration-stream-source-diagnostic/1',
            'base_commit': BASE, 'reviewed_files': EXPECTED_FILES,
            'wire_cache_commit': WIRE_CACHE, 'wire_source_commit': WIRE_SOURCE,
            'wire_cache_sha256': WIRE_HASH, 'wire_official_reader': wire['official_reader'],
            'verified_wire_modules': [{k: v for k, v in m.items() if k != 'parsed_module'}
                                      for m in wire['modules']],
            'wire_projection_boundary': 'Only exact OwnedRead parsed declarations are evaluated. '
                'FrameEventContract source/parser evidence is verified but its differing base source '
                'is not overlaid and its parsed declarations are not injected or exercised.',
            'current_core_projection': 'Only34 verified frozen Core source/cache pairs are overlaid. '
                'The sole evaluator edit is its exact cache fingerprint. This is not the complete '
                'Core checkout or linked native execution; native RuntimeResources remains base source.',
            'source_union_results': unions,
            'ownership_scope': 'Actual HeapAudit.auditRoots on joint enumerator/stream inventories '
                'at every positive execution/cancellation prefix; source ticket remains outstanding. '
                'Finite fixtures use actual ByteBuilder sequences but do not establish enumerator completeness.',
            'qualification': {'finite_generic_source_evaluation_only': True, 'official_source_check': False,
                'generated_native_execution': False, 'kernel_or_general_proof': False,
                'archive_membership_trial': False, 'fair_controller': False,
                'full_sdk_acceptance': False, 'physical_resource_measurement': False}}


def main():
    core.require(__debug__ and sys.flags.optimize == 0, 'optimized Python disables stream assertions')
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repository', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    result = qualify(args.repository.resolve(), Path(__file__).resolve().parents[2])
    result['qualification_runner_sha256'] = core.sha(Path(__file__).read_bytes())
    result['core_verification_helper_sha256'] = core.sha(Path(core.__file__).read_bytes())
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + '\n')
    print('PASS both exact source unions;111 custody prefixes/11 spans each;four intended mutants each')
    print('Generic source diagnostics only; no compiler/native/kernel/membership/full SDK acceptance.')


if __name__ == '__main__':
    main()
