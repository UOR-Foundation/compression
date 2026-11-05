#!/usr/bin/env python3
"""Project the literal source-owned generic adapter contract; no product behavior."""
import argparse
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]


def literal(term):
    kind = term['kind']
    if kind in ('string', 'bool'):
        return term['value']
    if kind == 'nat':
        return int(term['value'])
    if kind == 'record':
        return {field['field']: literal(field['value']) for field in term['fields']}
    if kind == 'nil':
        return []
    if kind == 'cons':
        return [literal(term['head'])] + literal(term['tail'])
    raise ValueError('The contract projection requires an actual source literal: ' + kind)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write', action='store_true')
    args = parser.parse_args()
    source = (ROOT / 'src/Uorc/Product/HostContract.lex.tex').read_text()
    module = json.loads(re.search(r'\\semanticdata\{(.*)\}', source).group(1))
    declaration = next(d for d in module['declarations'] if d['name'] == 'contract')
    projected = literal(declaration['body'])
    # Flatten only the source-owned metadata groups; no product behavior is evaluated here.
    if 'endpointGroups' in projected:
        projected['endpoints'] = [endpoint for group in projected['endpointGroups']
                                  for endpoint in group['endpoints']]
    expected = json.dumps(projected, sort_keys=True, indent=2) + '\n'
    target = ROOT / 'validation/product/host-contract.json'
    if args.write:
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(expected)
    elif target.read_text() != expected:
        raise SystemExit('Stale generic adapter projection; run scripts/product-host-projection.py --write')
    print('generic adapter contract projection matches its source')


if __name__ == '__main__':
    main()
