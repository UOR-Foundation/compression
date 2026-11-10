# The x86 branch filter (uorc's code filter) as an ir.Prog: one description that irlex compiles to
# LexLean (Uorc.Codec.X86) and irgraph to the kernel graph block that undoes it.
#
# x86 code is full of near calls and jumps (E8/E9 rel32) whose targets repeat at different offsets;
# rewriting each displacement as an absolute target makes the repeats literal. At every opcode E8 or
# E9 with four bytes after it, a displacement v whose top byte is 00 or FF (|v| < 2^24) becomes
# sext25(v + (base + i + 5)), and decoding subtracts instead; the four bytes are then skipped either
# way. The opcode bytes never change and a rewritten displacement keeps a top byte of 00 or FF, so
# both directions scan the same positions and decode(encode(x)) = encode(decode(x)) = x for every x.
# `base` places the bytes in a longer stream: a window frame is filtered at the offset that follows
# its (filtered) history, so a target reads the same in both. `hits` counts the rewritten displacements.
import sys
import os
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from ir import *

P = Prog()
for name, w in [('n', 4), ('dir', 1), ('skip', 1), ('base', 4), ('hits', 4)]:
    P.glob(name, w)
P.table('OUT', 1 << 33)
P.rom('POW2', 4, [1 << k for k in range(33)])

M32 = 1 << 32
step = [
    ('if', lt(0, G('skip')), [('set', 'skip', sub(G('skip'), 1))],
     [('let', 'b', IN(V('i'))),
      ('store', 'OUT', 1, V('i'), V('b')),
      ('if', AND(OR(eq(V('b'), 0xE8), eq(V('b'), 0xE9)), le(add(V('i'), 5), G('n'))),
       [('let', 'top', IN(add(V('i'), 4))),
        ('let', 'v', add(add(IN(add(V('i'), 1)), mul(256, IN(add(V('i'), 2)))), add(mul(65536, IN(add(V('i'), 3))), mul(1 << 24, V('top'))))),
        ('let', 'pos', mod(add(add(G('base'), V('i')), 5), M32)),
        ('let', 'wsum', mod(sel(eq(G('dir'), 0), add(V('v'), V('pos')), sub(add(V('v'), M32), V('pos'))), M32)),
        ('let', 'wlow', mod(V('wsum'), 1 << 25)),
        ('let', 'hit', sel(OR(eq(V('top'), 0), eq(V('top'), 255)), 1, 0)),
        ('let', 'w', sel(eq(V('hit'), 1), sel(lt(V('wlow'), 1 << 24), V('wlow'), add(V('wlow'), 0xFE000000)), V('v'))),
        ('set', 'hits', add(G('hits'), V('hit'))),
        ('loop', 4, 'k', [('store', 'OUT', 1, add(add(V('i'), 1), V('k')), mod(div(V('w'), rom('POW2', 4, mul(8, V('k')))), 256))]),
        ('set', 'skip', 4)], [])])]
# dir 0 encodes (relative to absolute), 1 decodes; OUT holds the n output bytes
P.func('filter', ['size', 'd', 'at'], [('set', 'n', V('size')), ('set', 'dir', V('d')), ('set', 'base', V('at')),
                                       ('set', 'skip', 0), ('set', 'hits', 0), ('loop', G('n'), 'i', step)])


def py_run(data, d, base=0):
    m = Machine(P, bytes(data))
    m.call('filter', len(data), d, base)
    return bytes(m.t['OUT'].get(k, 0) for k in range(len(data))), m.g['hits']


def py_filter(data, d, base=0):
    return py_run(data, d, base)[0]


if __name__ == '__main__':
    import random
    r = random.Random(3)
    for n in (0, 1, 4, 5, 6, 64, 3000):
        x = bytes(r.choice([0xE8, 0xE9, 0, 255, r.randrange(256)]) for _ in range(n))
        for base in (0, 7, M32 - 3):
            y = py_filter(x, 0, base)
            assert py_filter(y, 1, base) == x, n
            assert py_filter(py_filter(x, 1, base), 0, base) == x, n
    print('x86 filter round-trips')
