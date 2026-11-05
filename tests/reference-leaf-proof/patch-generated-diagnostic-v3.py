#!/usr/bin/env python3
"""Make a metadata-derived constructor-whitelist diagnostic; never source acceptance."""
import argparse
import hashlib
import json
from pathlib import Path
import re

NAME = 'leaf_index_step_refines_independent_sequence'
EXPECTED = '909ffa4f488e18258b4ee25a5459bc5df1bd5d3d8df308b09313e95f723af3ed'
HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]


def constructor_whitelist():
    """Mirror checked finite-inductive metadata from the hash-bound source packet."""
    frozen = json.loads((HERE / 'manifest.json').read_text())
    modules = {}
    for row in frozen['original_sources']:
        path = REPO / row['path']
        data = path.read_bytes()
        if sha(data) != row['sha256']:
            raise SystemExit(f"Frozen metadata source mismatch: {row['path']}")
        source = data.decode()
        key = re.search(r'\\begin\{lexlean\}\{([^}]+)\}', source).group(1)
        body = json.JSONDecoder().raw_decode(source.split('\\semanticdata{', 1)[1])[0]
        modules[key] = (source, body, row)
    source, local, _ = modules['Uorc.Core.ReferenceSemantics']
    imports = sorted(set(re.findall(r'\\importmodule\{([^}]+)\}', source)))
    config_bytes = (REPO / 'lexlean.toml').read_bytes()
    if sha(config_bytes) != 'c382da4b5003d697067b70f882d29a7a4ef0f9f0bae035b0860d6ca717914deb':
        raise SystemExit('Frozen namespace configuration mismatch')
    config = config_bytes.decode()
    prefix = re.search(r'^module_prefix = "([^"]+)"$', config, re.M).group(1)
    rows = []
    for key in imports + ['Uorc.Core.ReferenceSemantics']:
        _, module, provenance = modules[key]
        for declaration in module['declarations']:
            if key == 'Uorc.Core.ReferenceSemantics' and declaration['name'] == NAME:
                break
            if declaration['kind'] != 'inductive':
                continue
            for constructor in declaration['constructors']:
                if not constructor['fields']:
                    continue
                segments = (prefix + '.' + key + '.' + declaration['name'] + '.' + constructor['name'] + '.injEq').split('.')
                # Quoting every segment preserves Name identity without assuming
                # that a checked semantic identifier is an unreserved Lean token.
                lemma = '.'.join('«' + segment + '»' for segment in segments)
                rows.append({'lemma': lemma, 'module': key,
                    'inductive': declaration['name'], 'constructor': constructor['name'],
                    'field_count': len(constructor['fields']),
                    'source': provenance['path'], 'source_sha256': provenance['sha256']})
    rows.sort(key=lambda row: row['lemma'])
    lemmas = ['Option.some.injEq'] + [row['lemma'] for row in rows]
    return lemmas, rows

DISCRIMINATE = 'all_goals (try contradiction)'

def sha(data):
    return hashlib.sha256(data).hexdigest()

def delimiter(text, target):
    stack = []
    pairs = {')': '(', ']': '[', '}': '{'}
    for i, ch in enumerate(text):
        if not stack and text.startswith(target, i):
            return i
        if ch in '([{':
            stack.append(ch)
        elif ch in ')]}':
            if not stack or stack.pop() != pairs[ch]:
                raise ValueError('Unexpected generated Lean delimiter')
    raise ValueError(f'No outer delimiter {target!r}')

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('generated_module', type=Path)
parser.add_argument('--out', type=Path, required=True)
parser.add_argument('--expected-sha256', default=EXPECTED)
a = parser.parse_args()
lemmas, metadata = constructor_whitelist()
INSERT = 'all_goals (try set_option linter.unusedSimpArgs false in simp_all only [' + ', '.join(lemmas) + '])'
raw = a.generated_module.read_bytes()
if sha(raw) != a.expected_sha256:
    raise SystemExit('Generated module hash mismatch; inspect fresh provenance before retrying')
s = raw.decode()
starts = list(re.finditer(r'(?m)^.*\btheorem ' + re.escape(NAME) + r'\b.* := by\n', s))
if len(starts) != 1:
    raise SystemExit('Expected exactly one target theorem')
start = starts[0].start()
header_end = starts[0].end()
following = re.search(r'(?m)^(?:@\[.*\] )?(?:public )?(?:def|theorem|inductive|structure)\b|^end\b', s[header_end:])
end = header_end + following.start() if following else len(s)
block = s[start:end]
pattern = re.compile(r"(?m)^(?P<pad> *)all_goals \(repeat' split\)\n(?P=pad)all_goals omega\n")
matches = list(pattern.finditer(block))
if len(matches) != 1:
    raise SystemExit('Target proof is not the exact expected PF-21 tail')
m = matches[0]
patched = block[:m.start()] + m.group('pad') + "all_goals (repeat' split)\n" + m.group('pad') + INSERT + '\n' + m.group('pad') + DISCRIMINATE + '\n' + m.group('pad') + 'all_goals omega\n' + block[m.end():]
header, proof = patched.split(' := by\n', 1)
pos = delimiter(header, ' : ')
binder_prefix, statement = header[:pos], header[pos + 3:]
if not statement.startswith('(') or not statement.endswith(')'):
    raise SystemExit('Expected parenthesized original equality')
inside = statement[1:-1]
eqpos = delimiter(inside, ' = ')
left = inside[:eqpos]
if a.out.exists():
    raise SystemExit('Output must be absent; refusing to overwrite diagnostics')
a.out.mkdir(parents=True)
variants = {'positive': patched,
    'false-none': binder_prefix + ' : (' + left + ' = (Option.none : Option Nat)) := by\n' + proof,
    'false-zero': binder_prefix + ' : (' + left + ' = (Option.some 0 : Option Nat)) := by\n' + proof}
rows = []
for label, replacement in variants.items():
    data = (s[:start] + replacement + s[end:]).encode()
    d = a.out / label
    d.mkdir()
    p = d / 'ReferenceSemantics.lean'
    p.write_bytes(data)
    rows.append({'variant': label, 'path': str(p), 'sha256': sha(data), 'original_statement_unchanged': label == 'positive', 'parameters_unchanged': True, 'compiler_run': False})
result = {'schema': 'uorc/generated-proof-diagnostic-copies-v3/1', 'original_generated_module_sha256': sha(raw), 'inserted_proof_lines': [INSERT, DISCRIMINATE], 'predecessor_experiment': 'https://github.com/UOR-Foundation/compression/blob/1a02afa2fb304f58856f743f32dae9febefe9457/tests/canonical-composition/reference-leaf-experiment-v2/result.json', 'predecessor_positive_sufficient': True, 'predecessor_scope': 'Modified-generated v2 only; no transfer of acceptance to v3', 'constructor_whitelist': metadata, 'constructor_whitelist_count': len(metadata), 'original_source_edits': False, 'compiler_or_sdk_edits': False, 'variants': rows, 'counterexamples': {'false-none': 'backing=[1], index=0, continuations=[]', 'false-zero': 'backing=[], index=0, continuations=[]'}, 'acceptance': False}
(a.out / 'manifest.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result, indent=2))
