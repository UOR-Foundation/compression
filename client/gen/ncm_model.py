# The ncm coder (uorc's neuromorphic context-mixing kernel) as an ir.Prog: one description that
# ir.py compiles to LexLean (encoder and decoder) and to the UORC kernel graph (decoder).
#
# The model, per bit: 17 hashed contexts (orders 0-4, 6, 8, words, sparse and aligned contexts, a
# record model that finds the input's row length, and the byte a long match predicts) each give an
# adaptive probability; a match model adds two inputs and a bias completes the 20. Two integer
# neural mixers (selected by the partial byte, and by the previous byte and bit position) combine
# them in the logistic domain; each weight learns by a bounded Hebbian rule, w += x*err, clamped
# to +-2^22. The mixed probability drives a 32-bit binary arithmetic coder.
#
# Signed quantities are kept in offset form so every target computes on naturals:
#   stretch values x in [-2047, 2047] as x + 2048; weights w as w + 2^22 in [0, 2^23], stored
#   as (w + 2^22 + KW) mod 2^24 so that a fresh (zero) table holds the initial weight 2^14;
#   slot probabilities p as (p + 32768) mod 65536 so that zero holds p = 1/2.
import sys
import os
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from ir import *

NC = 17               # hashed contexts
NIN = NC + 3          # mixer inputs: contexts, match probability, match vote, bias
TBMAX = 24
SQT = [1, 2, 3, 6, 10, 16, 27, 45, 73, 120, 194, 310, 488, 747, 1101, 1546, 2047,
       2549, 2994, 3348, 3607, 3785, 3901, 3975, 4022, 4050, 4068, 4079, 4085, 4089, 4092, 4093, 4094]
WINIT = (1 << 22) + (1 << 14)
KW = (1 << 24) - WINIT
M32 = 1 << 32

P = Prog()
for name, w in [('n', 4), ('tb', 1), ('mb', 1), ('pos', 4), ('c4', 4), ('c8', 4), ('wordh', 4), ('word1', 4),
                ('c0', 2), ('bit', 1), ('mptr', 4), ('mlen', 4), ('mexp', 1), ('mbucket', 1), ('eb', 1),
                ('r', 4), ('pm1', 2), ('pm2', 2), ('pr', 2), ('acc', 8), ('sso', 4), ('sw', 8),
                ('acc2', 8), ('sw2', 8), ('ml', 4), ('mf', 1), ('pi', 2), ('x1', 4), ('x2', 4), ('x', 4),
                ('ip', 4), ('op', 4), ('y', 1), ('cap', 4), ('ovf', 1), ('pm3', 2), ('acc3', 8), ('sw3', 8),
                ('ai1', 4), ('aj1', 1), ('ai2', 4), ('aj2', 1), ('hoff', 4)]:
    P.glob(name, w)
# graph store regions (key bases); every table is addressed as width * index + byte
for name, base in [('HB', 1 << 8), ('BB', 1 << 9), ('IX', 1 << 10), ('ST', 1 << 11), ('MS', 1 << 12), ('LP', 1 << 13),
                   ('LD', 1 << 14), ('STR', 1 << 15), ('RC', 1 << 18), ('W1', 1 << 19), ('W2', 1 << 20), ('W3', 1 << 21),
                   ('A1', (1 << 21) + (1 << 17)), ('A2', 1 << 23), ('MT', 1 << 26),
                   ('T', 1 << 27), ('H', 1 << 33), ('OUT', 1 << 34)]:
    P.table(name, base)
P.rom('POW2', 4, [1 << k for k in range(25)])

_tmp = [0]
def fresh(base):
    _tmp[0] += 1
    return '%s%d' % (base, _tmp[0])

def hash_(out, e, body):
    body.append(('let', out, ('hash', e)))

def squash_(out, di, body):
    body.append(('let', out, ('squash', di)))

def hist(i): return ld('H', 1, i)
# H holds the window history (hoff bytes, the preceding frames' outputs) and then the coded bytes: the
# match model works on absolute positions PA, everything else on positions in the coded bytes
PA = add(G('hoff'), G('pos'))
def coded(i): return hist(add(G('hoff'), i))

def match_key(p, body):
    """body += the match-model key of absolute position p (the hash of the 7 bytes before it), as mk."""
    body.append(('let', 'mh0', 0))
    last = 'mh0'
    for i in range(1, 8):
        nxt = fresh('mh')
        body.append(('let', nxt, u32(add(add(mul(0x2F0B4C27, V(last)), hist(sub(p, i))), 1))))
        last = nxt
    hash_('mhh', V(last), body)
    body.append(('let', 'mk', div(V('mhh'), rom('POW2', 4, sub(32, G('mb'))))))

# ---- one-time setup ----
init = []
# table size: the smallest 2^tb >= 1024 n, between 2^16 and 2^24
init += [('set', 'tb', 16),
         ('loop', TBMAX - 16, 'k', [('if', lt(rom('POW2', 4, G('tb')), mul(1024, G('n'))), [('set', 'tb', add(G('tb'), 1))], [])]),
         ('set', 'mb', sel(lt(22, G('tb')), 22, sub(G('tb'), 2))),
         ('set', 'c0', 1), ('set', 'x2', M32 - 1)]
# stretch: STR[j] = the least x with squash(x) >= j, else 2047 (offset form)
sb = []
squash_('sv', add(V('k'), 1), sb)
sb += [('let', 'cnt', monus(add(V('sv'), 1), G('pi'))),
       ('loop', V('cnt'), 'j', [('store', 'STR', 2, add(G('pi'), V('j')), add(V('k'), 1))]),
       ('set', 'pi', mx(G('pi'), add(V('sv'), 1)))]
init += [('set', 'pi', 0), ('loop', 4095, 'k', sb),
         ('loop', monus(4096, G('pi')), 'j', [('store', 'STR', 2, add(G('pi'), V('j')), 4095)])]
init += [('loop', 64, 'k', [('store', 'MS', 3, V('k'), sel(eq(mod(V('k'), 2), 1), 60000 * 256, 5536 * 256))])]
# the match model starts holding every history position (absolute positions 7 .. hoff - 1)
pf = []
match_key(add(7, V('j')), pf)
pf.append(('store', 'MT', 4, V('mk'), add(7, V('j'))))
init += [('loop', monus(G('hoff'), 7), 'j', pf)]
init += [('call', 'newByte', [])]
P.func('setup', [], init)

# ---- byte boundary: match model, then the context hashes ----
nb = []
nb += [('if', lt(0, G('mlen')), [('set', 'mlen', mn(add(G('mlen'), 1), 65535)), ('set', 'mptr', add(G('mptr'), 1))], [])]
mm = []
match_key(PA, mm)
look = [('let', 'cand', ld('MT', 4, V('mk'))),
        ('if', lt(0, V('cand')),
         [('set', 'ml', 0), ('set', 'mf', 1),
          ('loopw', 1024, AND(eq(G('mf'), 1), lt(G('ml'), V('cand'))), 'j',
           [('if', eq(hist(monus(V('cand'), add(G('ml'), 1))), hist(monus(PA, add(G('ml'), 1)))),
             [('set', 'ml', add(G('ml'), 1))], [('set', 'mf', 0)])]),
          ('if', le(7, G('ml')), [('set', 'mptr', V('cand')), ('set', 'mlen', G('ml'))], [])], [])]
mm += [('if', eq(G('mlen'), 0), look, []), ('store', 'MT', 4, V('mk'), PA)]
nb += [('if', le(7, PA), mm, [])]
L_ = G('mlen')
bucket = sel(lt(L_, 16), L_, sel(lt(L_, 32), add(16, div(monus(L_, 16), 4)), sel(lt(L_, 64), add(20, div(monus(L_, 32), 8)),
             sel(lt(L_, 512), add(24, div(monus(L_, 64), 64)), 31))))
nb += [('if', lt(0, G('mlen')), [('set', 'mexp', hist(G('mptr'))), ('set', 'mbucket', bucket)],
        [('set', 'mexp', 0), ('set', 'mbucket', 0)])]
R, pos, c4, c8 = G('r'), G('pos'), G('c4'), G('c8')
nb += [('let', 'b1', mod(c4, 256)), ('let', 'p3', mod(pos, 4)),
       ('let', 'ab', sel(AND(lt(0, R), le(R, pos)), coded(monus(pos, R)), 0)),
       ('let', 'ab1', sel(AND(lt(0, R), le(R, add(pos, 1))), coded(monus(add(pos, 1), R)), 0)),
       ('let', 'ab2', sel(AND(lt(0, R), le(mul(2, R), pos)), coded(monus(pos, mul(2, R))), 0))]
hash_('hc4', c4, nb)
hash_('hw1', G('word1'), nb)
b1, p3 = V('b1'), V('p3')
ctx = [0, add(b1, 256), mod(c4, 65536), add(mod(c4, 1 << 24), 1 << 24), u32(add(c4, 0x3000000)),
       u32(add(V('hc4'), mod(c8, 65536))), u32(add(V('hc4'), c8)), u32(add(G('wordh'), V('hw1'))), G('wordh'),
       add(sub(mod(c4, 65536), b1), mul(65536, p3)), add(sub(c4, mod(c4, 65536)), p3),
       add(add(b1, mul(256, p3)), 0x800000), add(add(mod(c4, 65536), mul(65536, p3)), 0x4000000),
       add(b1, mul(65536, mod(div(c4, 65536), 256))),
       u32(add(add(add(V('ab'), mul(256, b1)), mul(65536, R)), 0x10000000)),
       u32(add(add(add(add(V('ab'), mul(256, V('ab1'))), mul(65536, V('ab2'))), mul(1 << 24, R)), 0x80000000)),
       u32(add(add(sel(lt(0, G('mlen')), add(G('mexp'), 256), 0), mul(512, G('mbucket'))), 0x77000000))]
assert len(ctx) == NC
for k, e in enumerate(ctx):
    nb.append(('store', 'HB', 4, k, e))
nb.append(('loop', NC, 'k', [('store', 'HB', 4, V('k'), ('hash', u32(add(ld('HB', 4, V('k')), mul(0x1000193, V('k'))))))]))
P.func('newByte', [], nb)

# ---- prediction ----
pr = []
bk = [('let', 'nc', sel(eq(G('bit'), 0), 0, add(mod(G('c0'), 16), 16)))]
bkl = []
hash_('bh', u32(add(ld('HB', 4, V('k')), mul(0x2F0B4C27, V('nc')))), bkl)
bkl += [('let', 'bm', mod(V('bh'), rom('POW2', 4, G('tb')))), ('let', 'b0', sub(V('bm'), mod(V('bm'), 16))),
        ('let', 'h24', div(V('bh'), 1 << 24)), ('let', 'ck', add(V('h24'), sub(1, mod(V('h24'), 2)))),
        ('let', 'alt', sel(eq(mod(div(V('b0'), 16), 2), 1), monus(V('b0'), 16), add(V('b0'), 16)))]
# a bucket carries its context's check byte in the count of its unused slot 0: use the one of the pair
# that does, else recycle the less used one (by slot 1's count), cleared and tagged
recycle = [('let', 'bs', sel(lt(mod(ld('T', 3, add(V('alt'), 1)), 256), mod(ld('T', 3, add(V('b0'), 1)), 256)), V('alt'), V('b0'))),
           ('loop', 16, 'j', [('store', 'T', 3, add(V('bs'), V('j')), 0)]),
           ('store', 'T', 3, V('bs'), V('ck')), ('store', 'BB', 4, V('k'), V('bs'))]
bkl += [('if', eq(mod(ld('T', 3, V('b0')), 256), V('ck')), [('store', 'BB', 4, V('k'), V('b0'))],
         [('if', eq(mod(ld('T', 3, V('alt')), 256), V('ck')), [('store', 'BB', 4, V('k'), V('alt'))], recycle)])]
bk += [('loop', NC, 'k', bkl)]
pr += [('if', OR(eq(G('bit'), 0), eq(G('bit'), 4)), bk, [])]
pr += [('let', 'part', sel(lt(G('bit'), 4), G('c0'),
                          add(mod(G('c0'), rom('POW2', 4, monus(G('bit'), 4))), rom('POW2', 4, monus(G('bit'), 4)))))]
sl = [('let', 'ix', add(ld('BB', 4, V('k')), V('part'))), ('store', 'IX', 4, V('k'), V('ix')),
      ('let', 'sp', mod(add(div(ld('T', 3, V('ix')), 256), 32768), 65536)),
      ('store', 'ST', 2, V('k'), ld('STR', 2, div(V('sp'), 16)))]
pr += [('loop', NC, 'k', sl)]
pr += [('set', 'eb', 0)]
bp = sub(7, G('bit'))
mt = [('if', eq(div(add(G('mexp'), 256), rom('POW2', 4, add(bp, 1))), G('c0')),
       [('set', 'eb', mod(div(G('mexp'), rom('POW2', 4, bp)), 2))],
       [('set', 'mlen', 0), ('set', 'mbucket', 0)])]
pr += [('if', lt(0, G('mlen')), mt, [])]
pr += [('let', 'mi', add(mul(2, G('mbucket')), G('eb'))),
       ('store', 'ST', 2, NC, sel(lt(0, G('mlen')), ld('STR', 2, div(div(ld('MS', 3, V('mi')), 256), 16)), 2048)),
       ('let', 'vote', sel(lt(15, G('mbucket')), 1024, 256)),
       ('store', 'ST', 2, NC + 1, sel(lt(0, G('mlen')), sel(eq(G('eb'), 1), add(2048, V('vote')), sub(2048, V('vote'))), 2048)),
       ('store', 'ST', 2, NC + 2, 2048 + 256)]
# the two mixers' dot products, accumulated over the inputs
pr += [('set', 'acc', 0), ('set', 'sso', 0), ('set', 'sw', 0), ('set', 'acc2', 0), ('set', 'sw2', 0),
       ('set', 'acc3', 0), ('set', 'sw3', 0),
       ('let', 's1', mul(NIN, G('c0'))), ('let', 's2', mul(NIN, add(mul(8, mod(G('c4'), 256)), G('bit')))),
       ('let', 's3', mul(NIN, add(mul(8, mod(div(G('c4'), 256), 256)), G('bit'))))]
dl = [('let', 'so', ld('ST', 2, V('i'))),
      ('let', 'w1', mod(add(ld('W1', 3, add(V('s1'), V('i'))), WINIT), 1 << 24)),
      ('let', 'w2', mod(add(ld('W2', 3, add(V('s2'), V('i'))), WINIT), 1 << 24)),
      ('set', 'acc', add(G('acc'), mul(V('so'), V('w1')))), ('set', 'acc2', add(G('acc2'), mul(V('so'), V('w2')))),
      ('set', 'sso', add(G('sso'), V('so'))), ('set', 'sw', add(G('sw'), V('w1'))), ('set', 'sw2', add(G('sw2'), V('w2'))),
      ('let', 'w3', mod(add(ld('W3', 3, add(V('s3'), V('i'))), WINIT), 1 << 24)),
      ('set', 'acc3', add(G('acc3'), mul(V('so'), V('w3')))), ('set', 'sw3', add(G('sw3'), V('w3')))]
pr += [('loop', NIN, 'i', dl)]

def dot(out, accv, swv, body):
    a = fresh('da'); b = fresh('db')
    body.append(('let', a, add(accv, NIN * 2048 * (1 << 22))))
    body.append(('let', b, add(mul(1 << 22, G('sso')), mul(2048, swv))))
    body.append(('let', out, sel(le(V(b), V(a)), add(2048, mn(div(monus(V(a), V(b)), 65536), 2047)),
                                 sub(2048, mn(div(add(monus(V(b), V(a)), 65535), 65536), 2047)))))
dot('d1', G('acc'), G('sw'), pr)
dot('d2', G('acc2'), G('sw2'), pr)
dot('d3', G('acc3'), G('sw3'), pr)
squash_('q1', V('d1'), pr)
squash_('q2', V('d2'), pr)
squash_('q3', V('d3'), pr)
squash_('qm', div(mul(21846, add(add(V('d1'), V('d2')), V('d3'))), 65536), pr)
pr += [('set', 'pm1', V('q1')), ('set', 'pm2', V('q2')), ('set', 'pm3', V('q3')), ('let', 'pa', mx(mn(V('qm'), 4095), 1))]

# adaptive probability maps: rows of 33 points over the stretch domain, stored as offsets from their
# initial values (point j starts at 16 * squash(128 j - 2048)) so that a fresh table holds the start
def ainit(j):
    sq = fresh('sq')
    return sq, [('let', sq, ('squash', mn(mul(128, j), 4095)))], sel(eq(j, 0), 0, sel(eq(j, 32), 4095 * 16, mul(16, V(sq))))

def apm_get(table, idx, j, body):
    sq, pre, init = ainit(j)
    body += pre
    return mod(add(ld(table, 2, idx), init), 65536)

def refine(out, table, cx, gi, gj, body):
    s, l, w = fresh('as'), fresh('al'), fresh('aw')
    body += [('let', s, ld('STR', 2, V('pa'))), ('let', l, div(V(s), 128)), ('let', w, mod(V(s), 128))]
    base = fresh('ab')
    body += [('let', base, add(mul(33, cx), V(l)))]
    t0 = apm_get(table, V(base), V(l), body)
    t1 = apm_get(table, add(V(base), 1), add(V(l), 1), body)
    body += [('let', out, div(add(mul(t0, sub(128, V(w))), mul(t1, V(w))), 2048)),
             ('set', gi, add(V(base), div(V(w), 64))), ('set', gj, add(V(l), div(V(w), 64)))]

refine('r1', 'A1', G('c0'), 'ai1', 'aj1', pr)
hash_('ah2', u32(add(mul(0x2F0B4C27, mod(G('c4'), 1 << 24)), G('c0'))), pr)
refine('r2', 'A2', div(V('ah2'), 65536), 'ai2', 'aj2', pr)
pr += [('set', 'pr', mx(mn(div(add(add(V('pa'), V('r1')), mul(2, V('r2'))), 4), 4095), 1))]
P.func('predict', [], pr)

# ---- learning ----
def adapt(stored, y):
    """(stored p, count) -> new stored entry: p += (target - p) / (count + 1.5), count saturates at 127."""
    pass

up = []
for table, gi, gj in (('A1', 'ai1', 'aj1'), ('A2', 'ai2', 'aj2')):
    sq, pre, init = ainit(G(gj))
    t = fresh('at')
    up += pre + [('let', t, mod(add(ld(table, 2, G(gi)), init), 65536)),
                 ('let', t + 'n', sel(eq(G('y'), 1), add(V(t), div(sub(65598, V(t)), 64)), sub(V(t), div(add(V(t), 63), 64)))),
                 ('store', table, 2, G(gi), mod(sub(add(V(t + 'n'), 65536), init), 65536))]
sv = [('let', 'ix', ld('IX', 4, V('k'))), ('let', 'sv', ld('T', 3, V('ix'))),
      ('let', 'cn', mod(V('sv'), 256)), ('let', 'pp', mod(add(div(V('sv'), 256), 32768), 65536)),
      ('let', 'rt', div(131072, add(mul(2, V('cn')), 3))),
      ('let', 'np', sel(eq(G('y'), 1), add(V('pp'), div(mul(sub(65535, V('pp')), V('rt')), 65536)),
                        sub(V('pp'), div(add(mul(V('pp'), V('rt')), 65535), 65536)))),
      ('store', 'T', 3, V('ix'), add(mul(256, mod(add(V('np'), 32768), 65536)), mn(add(V('cn'), 1), 127)))]
up += [('loop', NC, 'k', sv)]
ms = [('let', 'mi', add(mul(2, G('mbucket')), G('eb'))), ('let', 'sv', ld('MS', 3, V('mi'))),
      ('let', 'cn', mod(V('sv'), 256)), ('let', 'pp', div(V('sv'), 256)),
      ('let', 'rt', div(131072, add(mul(2, V('cn')), 3))),
      ('let', 'np', sel(eq(G('y'), 1), add(V('pp'), div(mul(sub(65535, V('pp')), V('rt')), 65536)),
                        sub(V('pp'), div(add(mul(V('pp'), V('rt')), 65535), 65536)))),
      ('store', 'MS', 3, V('mi'), add(mul(256, V('np')), mn(add(V('cn'), 1), 127)))]
up += [('if', lt(0, G('mlen')), ms, [])]
up += [('let', 's1', mul(NIN, G('c0'))), ('let', 's2', mul(NIN, add(mul(8, mod(G('c4'), 256)), G('bit')))),
       ('let', 'e1', sel(eq(G('y'), 1), sub(4096, G('pm1')), G('pm1'))),
       ('let', 'e2', sel(eq(G('y'), 1), sub(4096, G('pm2')), G('pm2'))),
       ('let', 's3', mul(NIN, add(mul(8, mod(div(G('c4'), 256), 256)), G('bit')))),
       ('let', 'e3', sel(eq(G('y'), 1), sub(4096, G('pm3')), G('pm3'))),
       # plasticity decays from 128/65536 to 24/65536 per unit of x * err as the input is seen
       ('let', 'lr', add(24, div(104 * 4096, add(4096, G('pos')))))]

def learn(table, key, err, body):
    w = fresh('w'); m = fresh('m'); s = fresh('s')
    body += [('let', w, mod(add(ld(table, 3, key), WINIT), 1 << 24)),
             ('let', m, mul(mul(V('mag'), err), V('lr'))),
             ('let', s, sel(V('pos_'), mn(add(V(w), div(V(m), 65536)), 1 << 23), monus(V(w), div(add(V(m), 65535), 65536)))),
             ('store', table, 3, key, mod(add(V(s), KW), 1 << 24))]

wl = [('let', 'so', ld('ST', 2, V('i'))),
      ('let', 'mag', add(monus(V('so'), 2048), monus(2048, V('so')))),
      ('let', 'pos_', OR(AND(lt(2048, V('so')), eq(G('y'), 1)), AND(lt(V('so'), 2048), eq(G('y'), 0))))]
learn('W3', add(V('s3'), V('i')), V('e3'), wl)
learn('W1', add(V('s1'), V('i')), V('e1'), wl)
learn('W2', add(V('s2'), V('i')), V('e2'), wl)
up += [('loop', NIN, 'i', wl)]
up += [('set', 'c0', add(mul(2, G('c0')), G('y'))), ('set', 'bit', add(G('bit'), 1))]
be = [('let', 'b', sub(G('c0'), 256)), ('store', 'H', 1, PA, V('b')), ('set', 'pos', add(G('pos'), 1)),
      ('set', 'c8', u32(add(mul(256, G('c8')), div(G('c4'), 1 << 24)))), ('set', 'c4', u32(add(mul(256, G('c4')), V('b'))))]
letter = OR(OR(AND(le(97, V('b')), le(V('b'), 122)), AND(le(65, V('b')), le(V('b'), 90))), le(128, V('b')))
lw = []
hash_('wh', u32(add(G('wordh'), sel(eq(mod(div(V('b'), 32), 2), 1), V('b'), add(V('b'), 32)))), lw)
lw += [('set', 'wordh', V('wh'))]
be += [('if', letter, lw, [('if', lt(0, G('wordh')), [('set', 'word1', G('wordh')), ('set', 'wordh', 0)], [])])]
# record model: a distance seen twice in a row for the same byte value votes for that row length
be += [('let', 'lp', ld('LP', 4, V('b'))), ('let', 'dd', sub(G('pos'), V('lp'))),
       ('if', AND(AND(lt(0, V('lp')), lt(1, V('dd'))), AND(lt(V('dd'), 65536), eq(V('dd'), ld('LD', 4, V('b'))))),
        [('let', 'rc', ld('RC', 2, V('dd'))),
         ('if', lt(V('rc'), 65535), [('store', 'RC', 2, V('dd'), add(V('rc'), 1))], []),
         ('if', lt(ld('RC', 2, G('r')), ld('RC', 2, V('dd'))), [('set', 'r', V('dd'))], [])], []),
       ('store', 'LD', 4, V('b'), mn(V('dd'), M32 - 1)), ('store', 'LP', 4, V('b'), G('pos')),
       ('set', 'c0', 1), ('set', 'bit', 0), ('call', 'newByte', [])]
up += [('if', eq(G('bit'), 8), be, [])]
P.func('update', [], up)

# ---- arithmetic coder ----
def split():
    return [('let', 'xm', add(G('x1'), div(mul(sub(G('x2'), G('x1')), G('pr')), 4096)))]

dec = [('call', 'predict', [])] + split() + [
    ('set', 'y', sel(le(G('x'), V('xm')), 1, 0)),
    ('if', eq(G('y'), 1), [('set', 'x2', V('xm'))], [('set', 'x1', add(V('xm'), 1))]),
    ('loop', 4, 'k', [('if', eq(div(G('x1'), 1 << 24), div(G('x2'), 1 << 24)),
                       [('set', 'x1', u32(mul(256, G('x1')))), ('set', 'x2', add(u32(mul(256, G('x2'))), 255)),
                        ('set', 'x', add(u32(mul(256, G('x'))), IN(G('ip')))), ('set', 'ip', add(G('ip'), 1))], [])]),
    ('call', 'update', [])]
P.func('decodeBit', [], dec)
P.func('decodeByte', [], [('loop', 8, 'k', [('call', 'decodeBit', [])])])
dstart = [('set', 'n', add(add(add(mul(1 << 24, IN(0)), mul(1 << 16, IN(1))), mul(256, IN(2))), IN(3))),
          ('set', 'x', add(add(add(mul(1 << 24, IN(4)), mul(1 << 16, IN(5))), mul(256, IN(6))), IN(7))),
          ('set', 'ip', 8), ('call', 'setup', [])]
P.func('decodeStart', [], dstart)
P.func('decode', [], [('call', 'decodeStart', []), ('loop', G('n'), 'i', [('call', 'decodeByte', [])])])
# against a window: H already holds the h history bytes
P.func('decodeWith', ['h'], [('set', 'hoff', V('h')), ('call', 'decode', [])])

enc = [('call', 'predict', [])] + split() + [
    ('if', eq(G('y'), 1), [('set', 'x2', V('xm'))], [('set', 'x1', add(V('xm'), 1))]),
    ('loop', 4, 'k', [('if', eq(div(G('x1'), 1 << 24), div(G('x2'), 1 << 24)),
                       [('call', 'emit', [div(G('x2'), 1 << 24)]),
                        ('set', 'x1', u32(mul(256, G('x1')))), ('set', 'x2', add(u32(mul(256, G('x2'))), 255))], [])]),
    ('call', 'update', [])]
P.func('encodeBit', [], enc)
# the encoder's output is bounded by cap; past it the encoding is abandoned (ovf)
P.func('emit', ['v'], [('if', lt(G('op'), G('cap')), [('store', 'OUT', 1, G('op'), V('v')), ('set', 'op', add(G('op'), 1))],
                         [('set', 'ovf', 1)])])
eb_ = [('let', 'byte', IN(V('i'))),
       ('loop', 8, 'k', [('set', 'y', mod(div(V('byte'), rom('POW2', 4, sub(7, V('k')))), 2)), ('call', 'encodeBit', [])])]
P.func('encodeByte', ['i'], eb_)
P.func('encode', ['size', 'limit'], [('set', 'n', V('size')), ('set', 'cap', V('limit')), ('call', 'setup', []),
                                     ('loop', G('n'), 'i', [('call', 'encodeByte', [V('i')])]),
                                     ('call', 'emit', [div(G('x1'), 1 << 24)])])
P.func('encodeWith', ['size', 'limit', 'h'], [('set', 'hoff', V('h')), ('call', 'encode', [V('size'), V('limit')])])


def py_encode(data, history=b''):
    m = Machine(P, data)
    for i, b in enumerate(history):
        m.t['H'][i] = b
    m.call('encodeWith', len(data), 2 * len(data) + 64, len(history))
    assert not m.g['ovf']
    out = bytes(m.t['OUT'].get(i, 0) for i in range(m.g['op']))
    return len(data).to_bytes(4, 'big') + out

def py_decode(payload, history=b''):
    m = Machine(P, payload)
    for i, b in enumerate(history):
        m.t['H'][i] = b
    m.call('decodeWith', len(history))
    h = len(history)
    return bytes(m.t['H'].get(h + i, 0) for i in range(m.g['n']))
