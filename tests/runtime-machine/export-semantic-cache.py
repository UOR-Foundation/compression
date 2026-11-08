#!/usr/bin/env python3
"""Export byte-bound diagnostic IR through the existing official SDK reader.

This script contains no notation parser and never builds a reader or compiler.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

READER_COMMIT = '33953384ea84a6196a907982dc30f6a3c0d1a955'
READER_SOURCE = 'crates/conformance/src/bin/prismpm-semantic-source.rs'
READER_SOURCE_SHA = '54fe3a32d1ba75f752df3c802e2fe91ee143fe1d0e01f5c8f32c2cab4d187603'
READER_BINARY_SHA = 'e10afd4f75b6fe239caaab391d75f529e43b88bb4371c536d731d90d0941fde7'
CRATE_SHA = 'f2d579289052493d2c68274b10015d2cce544a78fbb876d73062184f09fb0a18'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def run(argv, *, cwd=None, data=None):
    result = subprocess.run(argv, cwd=cwd, input=data, capture_output=True)
    if result.returncode:
        raise RuntimeError('official command failed: ' + ' '.join(map(str, argv)) + '\n' + result.stderr.decode())
    return result.stdout


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--helper', type=Path, required=True)
    parser.add_argument('--prismpm-root', type=Path, required=True)
    parser.add_argument('--source-ref', required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--module', action='append', dest='modules')
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[2]
    prism = args.prismpm_root.resolve()
    helper = args.helper.resolve()
    if not re.fullmatch('[0-9a-f]{40}', args.source_ref):
        raise RuntimeError('source-ref must be an exact full commit SHA')
    commit = run(['git', 'rev-parse', args.source_ref + '^{commit}'], cwd=repo).decode().strip()
    if commit != args.source_ref:
        raise RuntimeError('resolved source commit differs')
    if run(['git', 'rev-parse', 'HEAD'], cwd=prism).decode().strip() != READER_COMMIT:
        raise RuntimeError('official reader checkout identity differs')
    if run(['git', 'status', '--porcelain'], cwd=prism):
        raise RuntimeError('official reader checkout must be clean')
    if digest((prism / READER_SOURCE).read_bytes()) != READER_SOURCE_SHA:
        raise RuntimeError('official reader source hash differs')
    if digest(helper.read_bytes()) != READER_BINARY_SHA:
        raise RuntimeError('existing authenticated reader binary hash differs')
    if digest((prism / 'vendor/lexlean/lexlean-0.4.0.crate').read_bytes()) != CRATE_SHA:
        raise RuntimeError('official vendored SDK identity differs')
    names = args.modules or ['RuntimeMachine', 'RuntimeMachineEngine', 'RuntimeMachineRefinement']
    if len(names) != len(set(names)) or not names:
        raise RuntimeError('module selection must be nonempty and unique')
    rows = []
    for name in names:
        if not re.fullmatch('[A-Z][A-Za-z0-9]*', name):
            raise RuntimeError('select a Core module basename')
        path = 'src/Uorc/Core/' + name + '.lex.tex'
        source = run(['git', 'show', commit + ':' + path], cwd=repo)
        encoded = run([str(helper), 'read'], data=source)
        if encoded != run([str(helper), 'read'], data=source):
            raise RuntimeError(name + ': repeated official read differs')
        parsed = json.loads(encoded)
        if parsed['spec'] != 'lexlean/semantic-module/2':
            raise RuntimeError(name + ': unexpected official semantic schema')
        request = json.dumps({'source': source.decode(), 'value': parsed}, separators=(',', ':')).encode()
        restored = run([str(helper), 'write'], data=request)
        if restored != source:
            raise RuntimeError(name + ': official notation round trip differs; format source before exporting')
        rows.append({'module': 'Uorc.Core.' + name, 'source_path': path,
                     'source_sha256': digest(source), 'parser_output_sha256': digest(encoded),
                     'repeat_read': 'byte_exact', 'notation_round_trip': 'byte_exact',
                     'imports': re.findall(r'\\importmodule\{([^}]+)\}', source.decode()),
                     'parsed_module': parsed})
    result = {
        'schema': 'uorc/compiler-parsed-runtime-source/1',
        'purpose': 'Diagnostic cache only; the byte-bound committed LexLean source remains authoritative.',
        'source_commit': commit,
        'official_reader': {'repository': 'https://github.com/UOR-Foundation/PrismPM',
                           'commit': READER_COMMIT, 'helper_source': READER_SOURCE,
                           'helper_source_sha256': READER_SOURCE_SHA,
                           'helper_binary_sha256': READER_BINARY_SHA,
                           'cargo_lock_sha256': digest((prism / 'Cargo.lock').read_bytes()),
                           'vendored_lexlean_crate_sha256': CRATE_SHA,
                           'api': 'lexlean::sdk::read_module', 'round_trip_api': 'lexlean::sdk::write_module'},
        'export_script': {'path': 'tests/runtime-machine/export-semantic-cache.py',
                          'sha256': digest(Path(__file__).read_bytes())},
        'modules': rows,
        'qualification': {'semantic_cache_only': True, 'production_export_accepted': False,
                          'pinned_sdk_accepted': False, 'full_release_accepted': False,
                          'physical_memory_measurement': False},
    }
    data = (json.dumps(result, separators=(',', ':')) + '\n').encode()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_bytes(data)
    print(json.dumps({'source_commit': commit, 'modules': len(rows), 'bytes': len(data), 'sha256': digest(data)}))


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, KeyError, RuntimeError) as error:
        print(str(error), file=sys.stderr)
        sys.exit(1)
