#!/usr/bin/env python3
"""Reproduce a frozen source-evaluation union, never compiler/native acceptance."""
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

BASE = '0c812074d6039aa2b6fb77b4a3dd77d07827f7c3'
CORE_SOURCE = '3609a3624f7bf0f0d7c83b181fd88317868bdad5'
CORE_CACHE = 'eb56990bf53cd0c483b40df19557bbe0bfa120be'
CACHE_PATH = 'tests/heap/semantic-cache/runtime-owners.json'
OLD_CACHE_HASH = '19388f9d77aae3d04486816e25eb48e968a0dbc9f7b7c45b92666a3df79afb8e'
CACHE_HASH = '06c2988f850938019f5454439cf130a736aaa81c636c425dff29ce21015e2b04'
EXPECTED_SCRIPTS = {
    'diagnostic.py': '6ad499ad95bc7fec9a5070cd68e87288c864f4cdabb229041c48377f1905da23',
    'diagnostic_test.py': '2a5de381162d2ce05f5b7a71f207de43e14eb9f311b9d69521bbdfe717ae26f0',
    'mutations.py': '4f80c5d3d5736ba4ec2a93cdd51aa90a02a146645535fe2f714a4ad1370849f9',
}
MUTANTS = {
    'omission': 'byte omitted by successor',
    'skip-latest-pivot': 'latest pivot skipped',
    'refusal-as-exclusion': 'refusal advanced/excluded predecessor',
    'stale-empty-ack': 'stale empty ticket was reaccepted',
}
DIAGNOSTIC_MARKERS = [
    'complete empty+all256 one-byte universe, carry to 0000/0001/0002 passed',
    'all successor ownership prefixes cancel to empty arena 60',
    'zero/refusal/split-prepayment/repeated-cancel prefixes passed 120',
    'failed control allocation resumes exact empty predecessor',
    'real allocation refusal and exact retry prefixes passed 9',
    'address overflow retains unresolved predecessor',
]


def sha(data):
    return hashlib.sha256(data).hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def git(root, *args):
    return subprocess.check_output(['git', '-C', str(root), *args])


def canonical(value):
    return json.dumps(value, ensure_ascii=False, separators=(',', ':')).encode()


def verify_cache(data, source_reader):
    require(sha(data) == CACHE_HASH, 'wrong frozen cache bytes')
    cache = json.loads(data)
    require(cache['schema'] == 'uorc/compiler-parsed-runtime-source/1', 'wrong cache schema')
    require(cache['source_commit'] == CORE_SOURCE, 'wrong Core source checkpoint')
    modules = cache['modules']
    names = [m['module'] for m in modules]
    paths = [m['source_path'] for m in modules]
    require(len(modules) == 34 and len(set(names)) == 34 and len(set(paths)) == 34,
            'missing, duplicate or extra frozen Core modules')
    for module in modules:
        path = module['source_path']
        require(path == 'src/' + module['module'].replace('.', '/') + '.lex.tex',
                'module/source-path mismatch')
        require(sha(source_reader(path)) == module['source_sha256'], path + ': source mismatch')
        require(sha(canonical(module['parsed_module'])) == module['parser_output_sha256'],
                path + ': parsed-module mismatch')
        require(module['parsed_module']['spec'] == 'lexlean/semantic-module/2',
                path + ': wrong parsed schema')
        require(module['repeat_read'] == module['notation_round_trip'] == 'byte_exact',
                path + ': missing official read/round-trip evidence')
    return cache


def verify_result(name, result):
    require(result['returncode'] == 0, name + ': command failed')
    output = result['stdout']
    require(not result['stderr'], name + ': unexpected stderr')
    if name == 'diagnostic_test.py':
        for marker in DIAGNOSTIC_MARKERS:
            require(output.splitlines().count(marker) == 1, 'missing/duplicate diagnostic: ' + marker)
    elif name == 'mutations.py':
        rejected = [line for line in output.splitlines() if line.startswith('REJECTED ')]
        require(rejected == ['REJECTED ' + name + ' ' + message for name, message in MUTANTS.items()],
                'missing, duplicate, reordered or wrong-failure mutant result')
        for marker in ['restored exact-source diagnostic probes passed',
                       'all four planted defects rejected and restored probes passed']:
            require(output.splitlines().count(marker) == 1, 'missing/duplicate restoration result')
    else:
        raise ValueError('unregistered diagnostic command')


def qualify(repository):
    cache_bytes = git(repository, 'show', CORE_CACHE + ':' + CACHE_PATH)
    read_core = lambda path: git(repository, 'show', CORE_SOURCE + ':' + path)
    cache = verify_cache(cache_bytes, read_core)
    archive = git(repository, 'archive', '--format=tar', BASE)
    with tempfile.TemporaryDirectory(prefix='owned-enumeration-core-') as directory:
        root = Path(directory)
        with tarfile.open(fileobj=io.BytesIO(archive)) as bundle:
            bundle.extractall(root, filter='data')
        for module in cache['modules']:
            (root / module['source_path']).write_bytes(read_core(module['source_path']))
        (root / CACHE_PATH).write_bytes(cache_bytes)
        diagnostic = root / 'tests/enumeration/diagnostic.py'
        for name, expected in EXPECTED_SCRIPTS.items():
            require(sha((diagnostic.parent / name).read_bytes()) == expected,
                    name + ': reviewed diagnostic changed')
        original = diagnostic.read_text()
        require(original.count(OLD_CACHE_HASH) == 1, 'ambiguous diagnostic cache binding')
        diagnostic.write_text(original.replace(OLD_CACHE_HASH, CACHE_HASH))
        core_paths = {module['source_path'] for module in cache['modules']}
        sources = []
        for path in sorted((root / 'src/Uorc').rglob('*.lex.tex')):
            relative = path.relative_to(root).as_posix()
            sources.append({'path': relative, 'sha256': sha(path.read_bytes()),
                            'source_commit': CORE_SOURCE if relative in core_paths else BASE})
        results = []
        environment = dict(os.environ, PYTHONDONTWRITEBYTECODE='1')
        environment.pop('PYTHONOPTIMIZE', None)
        environment.pop('PYTHONPATH', None)
        for name in ['diagnostic_test.py', 'mutations.py']:
            command = [sys.executable, '-B', str(diagnostic.parent / name)]
            process = subprocess.run(command, cwd=root, env=environment,
                                     capture_output=True, text=True, check=False)
            result = {'command': ['python3', '-B', 'tests/enumeration/' + name],
                      'returncode': process.returncode, 'stdout': process.stdout,
                      'stderr': process.stderr}
            verify_result(name, result)
            results.append(result)
        return {
            'schema': 'uorc/owned-enumeration-current-core-diagnostic/1',
            'base_commit': BASE, 'core_source_commit': CORE_SOURCE, 'core_cache_commit': CORE_CACHE,
            'core_cache_sha256': CACHE_HASH, 'core_modules': 34,
            'composition': 'Exact base sources with only the 34 frozen authoritative Core sources '
                           'and their authentic parsed cache overlaid. Not the complete Core checkout.',
            'evaluator_change': {'kind': 'diagnostic-only exact cache fingerprint substitution',
                                 'original_sha256': EXPECTED_SCRIPTS['diagnostic.py'],
                                 'projected_sha256': sha(diagnostic.read_bytes()),
                                 'old_cache_sha256': OLD_CACHE_HASH, 'new_cache_sha256': CACHE_HASH},
            'test_sources': EXPECTED_SCRIPTS,
            'source_union': sources,
            'parsed_core_modules': [{key: module[key] for key in
                                    ['module', 'source_path', 'source_sha256', 'parser_output_sha256',
                                     'repeat_read', 'notation_round_trip']}
                                   for module in cache['modules']],
            'commands': results,
            'harness_composition_limit': 'The whole Core checkpoint includes native-notation '
                                        'RuntimeResources absent from this parsed cache. The legacy '
                                        'diagnostic loader cannot read that module and reports a '
                                        'missing zeroUsage. This union retains the reviewed base '
                                        'RuntimeResources; it is not whole-Core qualification.',
            'qualification': {'generic_finite_source_evaluation_only': True,
                              'official_source_check': False, 'generated_native_execution': False,
                              'kernel_proofs': False, 'canonical_membership_bridge': False,
                              'full_sdk_acceptance': False, 'physical_resource_measurement': False},
        }


def main():
    require(__debug__, 'optimized Python disables diagnostic assertions')
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repository', type=Path, required=True,
                        help='Git object repository containing the three exact frozen commits')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    evidence = qualify(args.repository.resolve())
    evidence['qualification_runner_sha256'] = sha(Path(__file__).read_bytes())
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(evidence, indent=2) + '\n')
    print('PASS: 34-module frozen source union, original controls and four intended mutants')
    print('Generic source evaluation only; official/native/kernel/full SDK acceptance remain unrun.')


if __name__ == '__main__':
    main()
