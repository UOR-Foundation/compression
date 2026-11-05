#!/usr/bin/env python3
"""Make a second, constructor-discrimination diagnostic; never source acceptance."""
import argparse
import hashlib
import json
from pathlib import Path
import re

NAME = 'leaf_index_step_refines_independent_sequence'
EXPECTED = '909ffa4f488e18258b4ee25a5459bc5df1bd5d3d8df308b09313e95f723af3ed'
INSERT = 'all_goals (try set_option linter.unusedSimpArgs false in simp_all only [Option.some.injEq, Bytes.WorkEvent.byteDone.injEq])'
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
result = {'schema': 'uorc/generated-proof-diagnostic-copies/1', 'original_generated_module_sha256': sha(raw), 'inserted_proof_lines': [INSERT, DISCRIMINATE], 'predecessor_experiment': 'https://github.com/UOR-Foundation/compression/blob/9d995c291b9e84c31871f16baee96578271f0a52/tests/canonical-composition/reference-leaf-experiment-0e3/result.json', 'predecessor_positive_sufficient': False, 'original_source_edits': False, 'compiler_or_sdk_edits': False, 'variants': rows, 'counterexamples': {'false-none': 'backing=[1], index=0, continuations=[]', 'false-zero': 'backing=[], index=0, continuations=[]'}, 'acceptance': False}
(a.out / 'manifest.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result, indent=2))
