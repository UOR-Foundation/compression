#!/usr/bin/env python3
"""Materialize the unchanged-statement diagnostic fixture. Never invoke a compiler."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def read_module(path):
    return json.JSONDecoder().raw_decode(path.read_text().split('\\semanticdata{', 1)[1])[0]

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--out', type=Path, required=True)
a = parser.parse_args()
manifest = json.loads((HERE / 'manifest.json').read_text())
for source in manifest['original_sources']:
    p = REPO / source['path']
    if digest(p) != source['sha256']:
        raise SystemExit(f"Frozen source mismatch: {source['path']}")
fixture = HERE / manifest['fixture_path']
if digest(fixture) != manifest['fixture_sha256']:
    raise SystemExit('Fixture bytes differ from manifest')
original = read_module(REPO / 'src/Uorc/Core/ReferenceSemantics.lex.tex')
small = read_module(fixture)
originals = {d['name']: d for d in original['declarations']}
for declaration in small['declarations']:
    if declaration != originals[declaration['name']]:
        raise SystemExit(f"Changed declaration: {declaration['name']}")
if a.out.exists():
    raise SystemExit('Output must be absent; refusing to overwrite an existing fixture')
(a.out / 'src/Uorc/Core').mkdir(parents=True)
for source in manifest['original_sources']:
    name = Path(source['path']).name
    src = fixture if name == 'ReferenceSemantics.lex.tex' else REPO / source['path']
    shutil.copyfile(src, a.out / 'src/Uorc/Core' / name)
for name in ['lakefile.toml', 'lake-manifest.json', 'lean-toolchain']:
    shutil.copyfile(REPO / name, a.out / name)
config = (REPO / 'lexlean.toml').read_text()
config = config.replace('name = "uorc-authority"', 'name = "uorc-reference-leaf-diagnostic"')
config = config.replace('entrypoints = ["src/Uorc/Registry.lex.tex", "src/Uorc/Specification.lex.tex"]', 'entrypoints = ["src/Uorc/Core/ReferenceSemantics.lex.tex"]')
(a.out / 'lexlean.toml').write_text(config)
print(json.dumps({'materialized': str(a.out), 'source_count': 6, 'reference_sha256': digest(a.out / 'src/Uorc/Core/ReferenceSemantics.lex.tex'), 'compiler_run': False, 'product_files_changed': False}, indent=2))
