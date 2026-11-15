# The delta filter family as an ir.Prog: the reference client/gen/ir_check.py runs against the native
# filter (crates/uorc/src/delta.rs); the model states the same filter in Uorc.Codec.Delta.
#
# A frame read as samples w bytes wide (little-endian, w = 1, 2 or 4), interleaved in c channels:
# each sample becomes its difference from the sample c samples before it, modulo 2^(8w), and decoding
# adds the differences back in order. The first c samples, and the bytes past the last whole sample,
# are kept, so decode(encode(x)) = encode(decode(x)) = x for every x.
#
# `measure` counts the magnitude bits of a frame's samples (raw 1) or of their differences (raw 0):
# a sample's bits less its sign, 0 for 0 and -1. The client tries the member whose differences save
# the most bits over the samples at its width (Client.deltaCandidate).
import sys
import os
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from ir import *

P = Prog()
for name, w in [('n', 4), ('dir', 1), ('w', 1), ('c', 1), ('raw', 1), ('acc', 8), ('bef', 8), ('bits', 8),
                ('ml', 1)]:
    P.glob(name, w)
P.table('OUT', 1 << 33)
P.rom('POW2', 5, [1 << k for k in range(33)])

M = rom('POW2', 5, mul(8, G('w')))     # 2^(8w)
TOP = rom('POW2', 5, sub(mul(8, G('w')), 1))


def sample(dst, src, k):
    """dst := the w-byte little-endian sample k of src(i) (a byte reader)."""
    return [('set', dst, 0),
            ('loop', G('w'), 'j', [('set', dst, add(G(dst), mul(rom('POW2', 5, mul(8, V('j'))),
                                                                   src(add(mul(k, G('w')), V('j'))))))])]


def out(i): return ld('OUT', 1, i)


K = add(V('t'), G('c'))
step = (sample('acc', IN, K)
        + [('if', eq(G('dir'), 0), sample('bef', IN, V('t')), sample('bef', out, V('t')))]
        + [('let', 'r', sel(eq(G('dir'), 0), mod(sub(add(G('acc'), M), G('bef')), M), mod(add(G('acc'), G('bef')), M))),
           ('loop', G('w'), 'j', [('store', 'OUT', 1, add(mul(K, G('w')), V('j')),
                                   mod(div(V('r'), rom('POW2', 5, mul(8, V('j')))), 256))])])
# d 0 filters, 1 inverts; OUT holds the n output bytes
P.func('filter', ['size', 'd', 'width', 'channels'],
       [('set', 'n', V('size')), ('set', 'dir', V('d')), ('set', 'w', V('width')), ('set', 'c', V('channels')),
        ('loop', G('n'), 'i', [('store', 'OUT', 1, V('i'), IN(V('i')))]),
        ('loop', monus(div(G('n'), G('w')), G('c')), 't', step)])

# the magnitude bits of every sample k (its difference from sample k - c unless raw, or k < c)
mag = (sample('acc', IN, V('k'))
       + [('if', AND(eq(G('raw'), 0), le(G('c'), V('k'))), sample('bef', IN, sub(V('k'), mn(G('c'), V('k')))),
           [('set', 'bef', 0)])]
       + [('let', 'v', mod(sub(add(G('acc'), M), G('bef')), M)),
          ('let', 'm', sel(lt(V('v'), TOP), V('v'), sub(sub(M, 1), V('v')))),
          ('set', 'ml', 0),
          ('loop', mul(8, G('w')), 'b', [('if', le(rom('POW2', 5, V('b')), V('m')), [('set', 'ml', add(V('b'), 1))], [])]),
          ('set', 'bits', add(G('bits'), G('ml')))])
P.func('measure', ['size', 'width', 'channels', 'r'],
       [('set', 'n', V('size')), ('set', 'w', V('width')), ('set', 'c', V('channels')), ('set', 'raw', V('r')),
        ('set', 'bits', 0), ('loop', div(G('n'), G('w')), 'k', mag)])


def py_filter(data, d, width, channels):
    m = Machine(P, bytes(data))
    m.call('filter', len(data), d, width, channels)
    return bytes(m.t['OUT'].get(k, 0) for k in range(len(data)))


def py_measure(data, width, channels, raw):
    m = Machine(P, bytes(data))
    m.call('measure', len(data), width, channels, raw)
    return m.g['bits']


if __name__ == '__main__':
    import random
    r = random.Random(3)
    for n in (0, 1, 3, 8, 61, 300):
        x = bytes(r.randrange(256) for _ in range(n))
        for width in (1, 2, 4):
            for channels in (1, 2, 3):
                y = py_filter(x, 0, width, channels)
                assert py_filter(y, 1, width, channels) == x, (n, width, channels)
                assert py_filter(py_filter(x, 1, width, channels), 0, width, channels) == x
    print('delta filter round-trips')
