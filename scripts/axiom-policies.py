#!/usr/bin/env python3
"""Rewrite each definition's `axioms:` member to the set its Lean elaboration observes.

LexLean enforces exact axiom policies and stops at the first declaration whose
stated set differs (LLV7005), so a model change that adds or removes a use of
propext or Quot.sound costs one client export per declaration. This tool reads
the whole observed map of one run (`name<TAB>axiom,axiom` per line, Lean full
names `Uorc.Uorc.<Module>.<definition>`) and rewrites every stated set that
differs, in the client sources (client/src, which the stage lays over src) and
then src. Definitions without an `axioms:` member, and names it cannot place,
are listed and left alone.

usage: scripts/axiom-policies.py OBSERVED.tsv [--check]
"""
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def source(module):
    rel = os.path.join(*module.split('.')) + '.lex.tex'
    for base in ('client/src', 'src'):
        p = os.path.join(ROOT, base, 'Uorc', rel)
        if os.path.exists(p):
            return p
    return None


def stated(text, name):
    """The span of `axioms: [...]` in the block whose `name:` is name, or None."""
    m = re.search(r'\n  name: ' + re.escape(name) + r',\n', text)
    if not m:
        return None
    end = text.find('\n}\n', m.end())
    a = re.compile(r'\n  axioms: \[([^\]]*)\]').search(text, m.end(), end if end >= 0 else len(text))
    return a


def main():
    observed = {}
    for line in open(sys.argv[1]):
        if '\t' not in line:
            continue
        name, axioms = line.rstrip('\n').split('\t')
        observed[name] = [x for x in axioms.split(',') if x]
    check = '--check' in sys.argv
    files, changed, unplaced = {}, [], []
    for full, axioms in sorted(observed.items()):
        parts = full.split('.')
        if parts[:2] != ['Uorc', 'Uorc'] or len(parts) < 4:
            continue
        module, name = '.'.join(parts[2:-1]), parts[-1]
        path = source(module)
        if path is None:
            unplaced.append(full)
            continue
        text = files.setdefault(path, open(path).read())
        a = stated(text, name)
        if a is None:
            continue
        want = sorted(axioms)  # the order LexLean states and compares them in
        have = [x.strip() for x in a.group(1).split(',') if x.strip()]
        if have != want:
            changed.append(f'{full}: [{", ".join(have)}] -> [{", ".join(want)}]')
            files[path] = text[:a.start(1)] + ', '.join(want) + text[a.end(1):]
    for line in changed:
        print(line)
    if unplaced:
        print(f'unplaced: {len(unplaced)} (no source module)', file=sys.stderr)
    if check:
        sys.exit(1 if changed else 0)
    for path, text in files.items():
        if open(path).read() != text:
            open(path, 'w').write(text)
    print(f'axiom-policies: {len(changed)} rewritten')


if __name__ == '__main__':
    main()
