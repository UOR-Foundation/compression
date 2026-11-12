# Streams per frame (crates/uorc/src/ncm/streams.rs) in the ncm IR: a frame of n bytes cut into k
# contiguous streams, each coded by its own arithmetic coder, advancing in epochs of e bytes. Within an
# epoch every stream reads the shared state as it was when the epoch began plus what it changed itself;
# at the epoch's end the streams' changes are merged into the shared state, order-free.
#
# The streams run one after another here (the bytes do not depend on how many threads run them). A
# stream's run writes the shared tables in place and logs the first old value of every entry it writes;
# after the run its values move into its delta and the old values come back, so the next stream reads
# the state as the epoch began. Each stream keeps its own scalars (saved in LG between runs), its own
# record model, context hashes and per-bit scratch (the private tables, one region per stream), and its
# own coder input or output region.
#
# The merge, per entry some stream changed (every version is the stream's value, or the base's when the
# stream did not change it):
#   - a context bucket (16 slots, slot 0's count its tag): by tag, the base's context a candidate while a
#     stream holds it (its tag, no count below the base's), every other tag a fresh one; within a group
#     a slot below the count limit merges as a running mean (p (2n+1) adds), at the limit by the mean
#     change of the streams that changed it; the group with the larger first-slot count wins, then the
#     smaller tag, then the first;
#   - mixer weights and probability map points: the base plus the mean change of the streams that
#     changed the entry (truncated toward zero), clamped to the entry's range;
#   - the match models' slots: as context slots;
#   - match tables and history bytes: the streams' values in stream order (the furthest position).
# With one stream every merge is the identity, and the payload is the serial coder's after the length.
import sys
import os
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from ir import *
import ncm_model as nm

P = nm.P
NIN = nm.NIN
STREAMS_AT, EPOCH_AT = 37, 53   # the frame word's fields (ncm::streams::word)
OFF = 1 << 50          # signed sums in offset form
M32 = (1 << 32) - 1

# ---- the tables ----
# merged tables: name -> (width, kind)
MERGED = {'T': (3, 'bucket'), 'W1': (3, 'weight'), 'W2': (3, 'weight'), 'W3': (3, 'weight'),
          'A1': (2, 'apm'), 'A2': (2, 'apm'), 'MS': (3, 'slot'), 'M2S': (3, 'slot'), 'RS': (3, 'slot'),
          'MT': (4, 'last'), 'M2T': (4, 'last'), 'H': (1, 'last')}
TID = {t: i for i, t in enumerate(MERGED)}
# private tables: name -> entries per stream (a power of two above every index the model uses)
PRIVATE = {'HB': 32, 'BB': 32, 'IX': 32, 'ST': 32, 'LP': 256, 'LD': 256, 'RC': 1 << 16}
REGION = 1 << 52
_regions = [0]
def region(name):
    _regions[0] += 1
    P.table(name, _regions[0] * REGION)
for t in PRIVATE:
    region(t + 'L')
# The merged tables share one entry space of zs entries, table t's from z<t> (spaces): K holds the
# stamp of the run that logged an entry, U the epoch stamp of a merged unit (by its first entry), and
# stream j's value of entry i and its epoch stamp are D and E at j * zs + i. UT/UK/UO is a run's log
# (table, key, old value), LT/LK the epoch's merged units, OUTL stream j's output at j * osp.
for name in ['K', 'U', 'D', 'E', 'LG', 'UT', 'UK', 'UO', 'LT', 'LK', 'OUTL', 'GR']:
    region(name)
GSPAN = 17 + 5 * 15    # words per merge group (see gr)

FRAME = {'n', 'tb', 'mb', 'prof', 'cap', 'ovf', 'learn', 'lmask', 'cm', 'ce', 'cfr', 'pi'}
LANE_G = [g for g in P.globals if g not in FRAME]
for name, w in [('seen', 4), ('lane', 2), ('sk', 3), ('ep', 4), ('sr', 4), ('stamp', 8), ('estamp', 8),
                ('ul', 8), ('iend', 8), ('before', 4), ('coded', 4), ('at', 4), ('ng', 2), ('bg', 2),
                ('bkey', 8), ('flen', 8), ('lc', 8)]:
    P.glob(name, w)
LANE_G += ['seen', 'iend']
LGSPAN = len(LANE_G)   # saved scalars per stream
for t in MERGED:
    P.glob('z' + t, 8)
for name in ['zs', 'osp', 'msteps']:
    P.glob(name, 8)
# entries of the merged tables the model sizes by the frame (tb, mb, the history); the others at
# their fixed extent
def extent(t):
    if t == 'T':
        return rom('POW2', P.roms['POW2'][0], G('tb'))
    if t in ('MT', 'M2T'):
        return rom('POW2', P.roms['POW2'][0], G('mb'))
    if t == 'H':
        return add(G('before'), G('n'))
    return {'A2': 1 << 22, 'M2S': 64, 'RS': 128}.get(t) or (min(b for b in P.tables.values() if b > P.tables[t]) - P.tables[t]) // MERGED[t][0]
def spaces():
    b, at = [], 0
    for t in MERGED:
        b.append(('set', 'z' + t, at))
        at = add(G('z' + t), extent(t))
    return b + [('set', 'zs', at)]
def eid(t, key): return add(G('z' + t), key)


# ---- a stream's functions: the serial ones over private regions, with logged writes ----
LANE_FN = ['newByte', 'predict', 'update', 'decodeBit', 'decodeByte', 'encodeBit', 'encodeByte', 'emit']


def lane_ex(e):
    if isinstance(e, int):
        return e
    k = e[0]
    if k in ('v', 'g'):
        return e
    if k == 'ld':
        t, w, key = e[1], e[2], lane_ex(e[3])
        if t in PRIVATE:
            return ('ld', t + 'L', w, add(key, mul(G('lane'), PRIVATE[t])))
        return ('ld', t, w, key)
    if k == 'rom':
        return ('rom', e[1], e[2], lane_ex(e[3]))
    if k == 'in':
        i = lane_ex(e[1])
        return sel(lt(i, G('iend')), ('in', i), 255)
    return (k,) + tuple(lane_ex(a) for a in e[1:])


def log(t, w, key):
    """The first write of a run to entry key of merged table t keeps the old value."""
    return [('if', ne(ld('K', 8, eid(t, key)), G('stamp')),
             [('store', 'K', 8, eid(t, key), G('stamp')),
              ('store', 'UT', 1, G('ul'), TID[t]), ('store', 'UK', 8, G('ul'), key),
              ('store', 'UO', 4, G('ul'), ld(t, w, key)), ('set', 'ul', add(G('ul'), 1))], [])]


def lane_st(ss, fn):
    out = []
    for s in ss:
        k = s[0]
        if k == 'let':
            e = lane_ex(s[2])
            if s[1] == 'lr':
                e = subst_pos(e)
            out.append(('let', s[1], e))
        elif k == 'set':
            out.append(('set', s[1], lane_ex(s[2])))
            if s[1] == 'pos' and fn == 'update':
                out.append(('set', 'seen', add(G('seen'), 1)))
        elif k == 'store':
            t, w = s[1], s[2]
            key, val = lane_ex(s[3]), lane_ex(s[4])
            if t in PRIVATE:
                out.append(('store', t + 'L', w, add(key, mul(G('lane'), PRIVATE[t])), val))
            elif t in MERGED:
                kv, vv = fresh('lk'), fresh('lv')
                out += [('let', kv, key), ('let', vv, val)] + log(t, w, V(kv)) + [('store', t, w, V(kv), V(vv))]
            elif t == 'OUT':
                out.append(('store', 'OUTL', w, add(key, mul(G('lane'), G('osp'))), val))
            elif t == 'GN':
                out.append(('store', t, w, key, val))   # the encoder's learning, never on with streams
            else:
                raise KeyError(t)
        elif k == 'if':
            c = lane_ex(s[1])
            if s[1] in (lt(0, V('cand')), lt(0, V('cand2'))):
                # a stream may read a later stream's position: a match points only into the past
                c = AND(c, lt(s[1][2], add(G('hoff'), G('pos'))))
            out.append(('if', c, lane_st(s[2], fn), lane_st(s[3], fn)))
        elif k == 'loop':
            out.append(('loop', lane_ex(s[1]), s[2], lane_st(s[3], fn)))
        elif k == 'loopw':
            out.append(('loopw', lane_ex(s[1]), lane_ex(s[2]), s[3], lane_st(s[4], fn)))
        elif k == 'call':
            f = s[1] + 'L' if s[1] in LANE_FN else s[1]
            out.append(('call', f, [lane_ex(a) for a in s[2]]))
        else:
            raise KeyError(k)
    return out


def subst_pos(e):
    """The plasticity reads the bytes the shared state has learned (the stream's clock), not its position."""
    if isinstance(e, int):
        return e
    if e == G('pos'):
        return G('seen')
    return (e[0],) + tuple(subst_pos(a) if isinstance(a, tuple) else a for a in e[1:])


fresh = nm.fresh
for f in LANE_FN:
    params, body = P.funcs[f]
    P.func(f + 'L', params, lane_st(body, f))


# ---- a stream's scalars ----
def save(lane):
    return [('store', 'LG', 8, add(mul(lane, LGSPAN), i), G(g)) for i, g in enumerate(LANE_G)]


def load(lane):
    return [('set', g, ld('LG', 8, add(mul(lane, LGSPAN), i))) for i, g in enumerate(LANE_G)]


P.func('laneSave', ['j'], save(V('j')))
P.func('laneLoad', ['j'], load(V('j')))
# the base's private tables (what setup left) are every stream's to start from
cp = []
for t, span in PRIVATE.items():
    if span <= 256:
        w = {'HB': 4, 'BB': 4, 'IX': 4, 'ST': 2, 'LP': 4, 'LD': 4}[t]
        cp.append(('loop', span, 'q', [('store', t + 'L', w, add(mul(V('j'), span), V('q')), ld(t, w, V('q')))]))
P.func('laneInit', ['j'], cp)


# ---- the end of a run: the stream's values into its delta, the old values back ----
def dispatch(tid_e, f):
    """Statements f(t) for the merged table whose id is tid_e."""
    out = []
    for t, i in TID.items():
        out.append(('if', eq(tid_e, i), f(t), []))
    return out


def commit_one(t):
    w = MERGED[t][0]
    dk = add(mul(G('lane'), G('zs')), eid(t, V('ck')))
    u = unit_of(t, V('ck'))
    return [('store', 'E', 8, dk, G('estamp')),
            ('store', 'D', 4, dk, ld(t, w, V('ck'))),
            ('store', t, w, V('ck'), V('co')),
            # the epoch's units, each once
            ('if', ne(ld('U', 8, eid(t, u)), G('estamp')),
             [('store', 'U', 8, eid(t, u), G('estamp')),
              ('store', 'LT', 1, G('lc'), TID[t]), ('store', 'LK', 8, G('lc'), u),
              ('set', 'lc', add(G('lc'), 1))], [])]


def unit_of(t, key):
    return sub(key, mod(key, 16)) if MERGED[t][1] == 'bucket' else key


cm_ = [('let', 'ct', ld('UT', 1, V('u'))), ('let', 'ck', ld('UK', 8, V('u'))), ('let', 'co', ld('UO', 4, V('u')))]
cm_ += dispatch(V('ct'), commit_one)
P.func('laneCommit', [], [('loop', G('ul'), 'u', cm_), ('set', 'ul', 0), ('set', 'stamp', add(G('stamp'), 1))])


# ---- the merge ----
def version(t, j, key):
    """Stream j's value of entry key of merged table t (the base's when it did not change it)."""
    w = MERGED[t][0]
    dk = add(mul(j, G('zs')), eid(t, key))
    return sel(eq(ld('E', 8, dk), G('estamp')), ld('D', 4, dk), ld(t, w, key))


def acc_add(pb, nb, p, n, pre):
    """Statements adding a stream's slot (p, n) over base (pb, nb) to the accumulator named pre."""
    s = lambda x: pre + x
    return [('if', OR(ne(p, pb), ne(n, nb)),
             [('set', s('ch'), add(G(s('ch')), 1)),
              ('let', 'op_', sel(lt(n, nb), 32768, pb)), ('let', 'on_', sel(lt(n, nb), 0, nb)),
              ('set', s('sat'), sel(OR(le(127, n), le(127, nb)), 1, G(s('sat')))),
              ('set', s('sum'), sub(add(G(s('sum')), mul(p, add(mul(2, n), 1))), mul(V('op_'), add(mul(2, V('on_')), 1)))),
              ('set', s('dn'), sub(add(G(s('dn')), n), V('on_'))),
              ('set', s('dp'), sub(add(G(s('dp')), p), V('op_')))], [])]


def acc_done(pb, nb, pre, outp, outn, body):
    """body += the merged slot (outp, outn) over base (pb, nb)."""
    s = lambda x: G(pre + x)
    ch = mx(s('ch'), 1)
    tot0 = monus(add(nb, s('dn')), OFF)                    # tot, at least 0
    num = add(mul(pb, add(mul(2, nb), 1)), s('sum'))      # offset by OFF; below it the mean is 0
    run = div(monus(num, OFF), add(mul(2, tot0), 1))
    dpo = s('dp')                                          # offset by OFF
    mean = sel(le(OFF, dpo), add(pb, div(monus(dpo, OFF), ch)), monus(pb, div(monus(OFF, dpo), ch)))
    body += [('let', outp, sel(eq(s('ch'), 0), pb, mn(sel(eq(s('sat'), 1), mean, run), 65535))),
             ('let', outn, sel(eq(s('ch'), 0), nb, mn(tot0, 127)))]


def acc_reset(pre):
    return [('set', pre + 'ch', 0), ('set', pre + 'sat', 0), ('set', pre + 'sum', OFF), ('set', pre + 'dn', OFF),
            ('set', pre + 'dp', OFF)]


for x, w in [('ch', 4), ('sat', 1), ('sum', 8), ('dn', 8), ('dp', 8)]:
    P.glob('a' + x, w)


def slot_pn(v):
    """A context slot (3 bytes: offset probability, count) as (p, n)."""
    return mod(add(div(v, 256), 32768), 65536), mod(v, 256)


def pn_slot(p, n):
    return add(mul(256, mod(add(p, 32768), 65536)), n)


# merge of a match model slot (p * 256 + count) of table t at key
def merge_small(t):
    b = [('let', 'sb', ld(t, 3, V('mk'))), ('let', 'spb', div(V('sb'), 256)), ('let', 'snb', mod(V('sb'), 256))]
    b += acc_reset('a')
    b += [('loop', G('sk'), 'j', [('let', 'sv', version(t, V('j'), V('mk'))),
                                  ] + acc_add(V('spb'), V('snb'), div(V('sv'), 256), mod(V('sv'), 256), 'a'))]
    acc_done(V('spb'), V('snb'), 'a', 'sp2', 'sn2', b)
    b += [('store', t, 3, V('mk'), add(mul(256, V('sp2')), V('sn2'))), ('set', 'msteps', add(G('msteps'), G('ach')))]
    return b


def merge_weight(t, init, lo_, hi_):
    """Mean change over the streams that changed the entry, in the entry's actual value (stored + init)."""
    w = MERGED[t][0]
    M = 1 << (8 * w)
    b = [('let', 'wx', mod(add(ld(t, w, V('mk')), init), M)), ('set', 'ach', 0), ('set', 'adp', OFF)]
    b += [('loop', G('sk'), 'j', [('let', 'wv', mod(add(version(t, V('j'), V('mk')), init), M)),
                                  ('if', ne(V('wv'), V('wx')), [('set', 'ach', add(G('ach'), 1)),
                                                                ('set', 'adp', sub(add(G('adp'), V('wv')), V('wx')))], [])])]
    ch = mx(G('ach'), 1)
    mean = sel(le(OFF, G('adp')), add(V('wx'), div(monus(G('adp'), OFF), ch)),
               monus(V('wx'), div(monus(OFF, G('adp')), ch)))
    b += [('if', lt(0, G('ach')), [('let', 'wn', mx(mn(mean, hi_), lo_)),
                                   ('store', t, w, V('mk'), mod(sub(add(V('wn'), M), init), M))], []),
          ('set', 'msteps', add(G('msteps'), G('ach')))]
    return b


def merge_last(t):
    w = MERGED[t][0]
    count = [] if t == 'H' else [('set', 'msteps', add(G('msteps'), 1))]
    return [('loop', G('sk'), 'j', [('let', 'dk', add(mul(V('j'), G('zs')), eid(t, V('mk')))),
                                    ('if', eq(ld('E', 8, V('dk')), G('estamp')),
                                     [('store', t, w, V('mk'), ld('D', 4, V('dk')))] + count, [])])]


# ---- a context bucket ----
# groups in GR: group g at g * GSPAN: [0] fresh, [1..16] origin slots (3-byte form), then per slot
# s (1..15) at 17 + 5 (s - 1): changed, sat, sum, dn, dp
def gr(g, i): return add(mul(g, GSPAN), i)
def gacc(g, s, f): return gr(g, add(17, add(mul(5, sub(s, 1)), f)))


def bslot(j, s):
    """Stream j's slot s of the bucket at bkey."""
    return version('T', j, add(G('bkey'), s))


def base_slot(s): return ld('T', 3, add(G('bkey'), s))
def tag_of(v0): return mod(v0, 256)


mb = [('set', 'ng', 0)]
mb += [('set', 'anyb', 0)]
# Machine/graph-friendly: compute "holds" per stream into GR scratch region above the groups
# a stream's hold flag, past the groups (k + 1 at most)
def hold(j): return add(mul(8, mul(add(G('sk'), 1), GSPAN)), j)
hbl = [('let', 'hs', eq(tag_of(bslot(V('j'), 0)), tag_of(base_slot(0)))),
       ('store', 'GR', 1, hold(V('j')), sel(V('hs'), 1, 0)),
       ('loop', 15, 'q', [('if', lt(mod(bslot(V('j'), add(V('q'), 1)), 256), mod(base_slot(add(V('q'), 1)), 256)),
                          [('store', 'GR', 1, hold(V('j')), 0)], [])])]
mb += [('loop', G('sk'), 'j', hbl)]
mb += [('loop', G('sk'), 'j', [('if', eq(ld('GR', 1, hold(V('j'))), 1), [('set', 'anyb', 1)], [])])]
# a step for every stream that wrote the bucket
mb += [('loop', G('sk'), 'j', [('set', 'isb', 0),
                               ('loop', 16, 's', [('if', eq(ld('E', 8, add(mul(V('j'), G('zs')), eid('T', add(G('bkey'), V('s'))))), G('estamp')),
                                                   [('set', 'isb', 1)], [])]),
                               ('set', 'msteps', add(G('msteps'), G('isb')))])]


def new_group(fr, origin_slot):
    """Statements making group ng (fresh fr) with origin slots origin_slot(s), then ng += 1."""
    b = [('store', 'GR', 8, gr(G('ng'), 0), fr)]
    b += [('loop', 16, 's', [('store', 'GR', 8, gr(G('ng'), add(1, V('s'))), origin_slot(V('s')))])]
    b += [('loop', 15, 's', [('store', 'GR', 8, gacc(G('ng'), add(V('s'), 1), 0), 0),
                             ('store', 'GR', 8, gacc(G('ng'), add(V('s'), 1), 1), 0),
                             ('store', 'GR', 8, gacc(G('ng'), add(V('s'), 1), 2), OFF),
                             ('store', 'GR', 8, gacc(G('ng'), add(V('s'), 1), 3), OFF),
                             ('store', 'GR', 8, gacc(G('ng'), add(V('s'), 1), 4), OFF)])]
    b += [('set', 'ng', add(G('ng'), 1))]
    return b


mb += [('if', eq(G('anyb'), 1), new_group(0, lambda s: base_slot(s)), [])]
# each stream's bucket into its group
same = [('set', 'bg', 0)]
# a stream whose bucket is the base's adds nothing
eqb = [('set', 'isb', 1), ('loop', 16, 's', [('if', ne(bslot(V('j'), V('s')), base_slot(V('s'))), [('set', 'isb', 0)], [])])]
find = [('set', 'bg', G('ng')),
        ('loop', G('ng'), 'g', [('if', AND(eq(ld('GR', 8, gr(V('g'), 0)), 1),
                                           AND(eq(tag_of(ld('GR', 8, gr(V('g'), 1))), tag_of(bslot(V('j'), 0))),
                                               eq(G('bg'), G('ng')))),
                                 [('set', 'bg', V('g'))], [])]),
        ('if', eq(G('bg'), G('ng')),
         new_group(1, lambda s: sel(eq(s, 0), tag_of(bslot(V('j'), 0)), 0)), [])]
addacc = []
for f, gname in enumerate(['ch', 'sat', 'sum', 'dn', 'dp']):
    addacc.append(('set', 'a' + gname, ld('GR', 8, gacc(G('bg'), V('s1'), f))))
og = ld('GR', 8, gr(G('bg'), add(1, V('s1'))))
opb, onb = slot_pn(og)
lp_, ln_ = slot_pn(bslot(V('j'), V('s1')))
addacc += [('let', 'opb', opb), ('let', 'onb', onb), ('let', 'lpv', lp_), ('let', 'lnv', ln_)]
addacc += acc_add(V('opb'), V('onb'), V('lpv'), V('lnv'), 'a')
for f, gname in enumerate(['ch', 'sat', 'sum', 'dn', 'dp']):
    addacc.append(('store', 'GR', 8, gacc(G('bg'), V('s1'), f), G('a' + gname)))
each = eqb + [('if', eq(G('isb'), 0),
               [('if', eq(ld('GR', 1, hold(V('j'))), 1), [('set', 'bg', 0)], find),
                ('loop', 15, 's', [('let', 's1', add(V('s'), 1))] + addacc)], [])]
mb += [('loop', G('sk'), 'j', each)]
# the winner: the larger merged first-slot count, then the smaller tag, then the first group
for name, w in [('bestk', 8), ('bestg', 2), ('anyb', 1), ('isb', 1)]:
    P.glob(name, w)
win = []
for f, gname in enumerate(['ch', 'sat', 'sum', 'dn', 'dp']):
    win.append(('set', 'a' + gname, ld('GR', 8, gacc(V('g'), 1, f))))
o1p, o1n = slot_pn(ld('GR', 8, gr(V('g'), 2)))
win += [('let', 'o1p', o1p), ('let', 'o1n', o1n)]
acc_done(V('o1p'), V('o1n'), 'a', 'm1p', 'm1n', win)
win += [('let', 'gkey', add(mul(V('m1n'), 256), sub(255, tag_of(ld('GR', 8, gr(V('g'), 1)))))),
        ('if', OR(eq(V('g'), 0), lt(G('bestk'), V('gkey'))), [('set', 'bestk', V('gkey')), ('set', 'bestg', V('g'))], [])]
mb += [('if', lt(0, G('ng')), [('loop', G('ng'), 'g', win)], [])]
wr = []
for f, gname in enumerate(['ch', 'sat', 'sum', 'dn', 'dp']):
    wr.append(('set', 'a' + gname, ld('GR', 8, gacc(G('bestg'), V('s1'), f))))
ops, ons = slot_pn(ld('GR', 8, gr(G('bestg'), add(1, V('s1')))))
wr += [('let', 'ops', ops), ('let', 'ons', ons)]
acc_done(V('ops'), V('ons'), 'a', 'mp', 'mn', wr)
wr += [('store', 'T', 3, add(G('bkey'), V('s1')), pn_slot(V('mp'), V('mn')))]
mb += [('if', lt(0, G('ng')),
        [('store', 'T', 3, G('bkey'), ld('GR', 8, gr(G('bestg'), 1))),
         ('loop', 15, 's', [('let', 's1', add(V('s'), 1))] + wr)], [])]
P.func('mergeBucket', [], mb)


# ---- the epoch's merge: every unit some stream changed, once ----
def merge_unit(t):
    kind = MERGED[t][1]
    u = unit_of(t, V('lk'))
    b = [('let', 'mk', u)]
    body = []
    if kind == 'bucket':
        body = [('set', 'bkey', V('mk')), ('call', 'mergeBucket', [])]
    elif kind == 'weight':
        body = merge_weight(t, nm.winit(mod(V('mk'), NIN)), 0, 1 << 23)
    elif kind == 'apm':
        sq = fresh('sq')
        j = mod(V('mk'), 33)
        body = [('let', sq, ('squash', mn(mul(128, j), 4095)))]
        init = sel(eq(j, 0), 0, sel(eq(j, 32), 4095 * 16, mul(16, V(sq))))
        body += merge_weight(t, init, 0, 65535)
    elif kind == 'slot':
        body = merge_small(t)
    else:
        body = merge_last(t)
    return b + body


ml = [('let', 'lt_', ld('LT', 1, V('q'))), ('let', 'lk', ld('LK', 8, V('q')))]
ml += dispatch(V('lt_'), merge_unit)
P.func('merge', [], [('loop', G('lc'), 'q', ml), ('set', 'lc', 0), ('set', 'estamp', add(G('estamp'), 1))])


# ---- the regions' sizes, for a lowering that sizes every table by the frame ----
def _widths():
    w = {}
    def walk(ss):
        for st in ss:
            if st[0] == 'store':
                w.setdefault(st[1], st[2])
                w[st[1]] = max(w[st[1]], st[2])
            elif st[0] == 'if':
                walk(st[2]); walk(st[3])
            elif st[0] == 'loop':
                walk(st[3])
            elif st[0] == 'loopw':
                walk(st[4])
    for _, body in P.funcs.values():
        walk(body)
    return w
STATIC = sum(extent(t) for t in MERGED if isinstance(extent(t), int))


def regions(entries):
    """name -> (width, entries) of every streams region, from entries(name) of the frame's quantities:
    'zs' (the merged entry space), 'sk' (streams), 'sr' (stride)."""
    w = _widths()
    zs, sk, sr = entries('zs'), entries('sk'), entries('sr')
    r = {t + 'L': (w[t + 'L'], ('mul', sk, PRIVATE[t])) for t in PRIVATE}
    for t in ['K', 'U', 'UT', 'UK', 'UO', 'LT', 'LK']:
        r[t] = (w[t], zs)
    r['D'] = (w['D'], ('mul', sk, zs))
    r['E'] = (w['E'], ('mul', sk, zs))
    r['LG'] = (w['LG'], ('mul', sk, LGSPAN))
    r['OUTL'] = (1, ('mul', sk, ('add', ('mul', 2, sr), 64)))
    r['GR'] = (1, ('add', ('mul', 8, ('mul', ('add', sk, 1), GSPAN)), sk))
    return r


# ---- the frame ----
def stream_start(i): return mn(mul(i, G('sr')), G('n'))
def stream_end(i): return mn(mul(add(i, 1), G('sr')), G('n'))
# a stream's coder abandons the encoding past twice its length and 64 bytes, as a frame's coder does
def lane_cap(i): return add(mul(2, sub(stream_end(i), stream_start(i))), 64)
# the payload's stream i: its first byte and its end (the last stream reads to the payload's end)
def be32(at): return add(add(add(mul(1 << 24, IN(at)), mul(1 << 16, IN(add(at, 1)))), mul(256, IN(add(at, 2)))), IN(add(at, 3)))


plan = [('set', 'prof', mod(V('pf'), 1 << 32)), ('set', 'sk', add(mod(div(V('pf'), 1 << STREAMS_AT), 1 << 16), 1)),
        ('set', 'ep', rom('POW2', P.roms['POW2'][0], mod(div(V('pf'), 1 << EPOCH_AT), 32))),
        ('set', 'sr', mul(64, div(add(div(add(G('n'), sub(G('sk'), 1)), G('sk')), 63), 64))),
        ('set', 'stamp', 1), ('set', 'estamp', 1), ('set', 'ul', 0), ('set', 'lc', 0), ('set', 'msteps', 0), ('set', 'coded', 0)]


def lanes_start(decoding):
    """Every stream's scalars from the base's, its private tables, its first byte's model (streams after the first)."""
    b = [('set', 'seen', 0)]
    b += spaces() + [('set', 'osp', add(mul(2, G('sr')), 64))]
    b += [('loop', G('sk'), 'j', [('call', 'laneSave', [V('j')]), ('call', 'laneInit', [V('j')])])]
    st = [('let', 'jj', add(V('j'), 1)), ('call', 'laneLoad', [V('jj')]), ('set', 'lane', V('jj')),
          ('set', 'hoff', add(G('before'), stream_start(V('jj')))), ('call', 'newByteL', []), ('call', 'laneCommit', []),
          ('call', 'laneSave', [V('jj')])]
    b += [('loop', sub(G('sk'), 1), 'j', st)]
    if decoding:
        # each stream's coder starts on its first 4 bytes
        b += [('set', 'flen', add(4, mul(4, sub(G('sk'), 1))))]
        co = [('call', 'laneLoad', [V('j')]),
              ('set', 'ip', G('flen')),
              ('set', 'iend', sel(lt(V('j'), sub(G('sk'), 1)), add(G('flen'), be32(add(4, mul(4, V('j'))))), 1 << 60)),
              ('set', 'x', 0),
              ('loop', 4, 'q', [('set', 'x', add(mul(256, G('x')), sel(lt(G('ip'), G('iend')), IN(G('ip')), 255))),
                                ('set', 'ip', add(G('ip'), 1))]),
              ('set', 'x1', 0), ('set', 'x2', M32),
              ('call', 'laneSave', [V('j')]),
              ('set', 'flen', sel(lt(V('j'), sub(G('sk'), 1)), G('iend'), G('flen')))]
        b += [('loop', G('sk'), 'j', co)]
    else:
        b += [('loop', G('sk'), 'j', [('call', 'laneLoad', [V('j')]), ('set', 'op', 0), ('set', 'x1', 0), ('set', 'iend', 1 << 60),
                                      ('set', 'x2', M32), ('call', 'laneSave', [V('j')])])]
    return b


def epochs(byte_call):
    """Every epoch: each stream over its part, then the merge."""
    run = [('let', 'plo', mn(add(stream_start(V('j')), G('at')), stream_end(V('j')))),
           ('let', 'phi', mn(add(stream_start(V('j')), add(G('at'), G('ep'))), stream_end(V('j')))),
           ('call', 'laneLoad', [V('j')]), ('set', 'lane', V('j')), ('set', 'cap', lane_cap(V('j'))),
           ('loop', sub(V('phi'), V('plo')), 'b', byte_call(add(V('plo'), V('b')))),
           ('call', 'laneCommit', []), ('call', 'laneSave', [V('j')]),
           ('set', 'coded', add(G('coded'), sub(V('phi'), V('plo'))))]
    ep = [('loop', G('sk'), 'j', run), ('call', 'merge', []),
          ('loop', G('sk'), 'j', [('store', 'LG', 8, add(mul(V('j'), LGSPAN), LANE_G.index('seen')), G('coded'))]),
          ('set', 'at', add(G('at'), G('ep')))]
    return [('set', 'at', 0), ('loop', div(add(G('sr'), sub(G('ep'), 1)), G('ep')), 'e', ep)]


# decodeStreams h pf: the frame after the h history bytes H holds, from a streams payload, under frame word pf
dec = [('set', 'n', be32(0))] + plan + [('set', 'hoff', V('h')), ('set', 'before', V('h')), ('call', 'setup', [])]
dec += lanes_start(True)
dec += epochs(lambda i: [('call', 'decodeByteL', [])])
P.func('decodeStreams', ['h', 'pf'], dec)

# encodeStreams size h pf: the streams payload of the size input bytes after h history bytes, in OUT
enc = [('set', 'n', V('size')), ('set', 'cap', M32)] + plan + [('set', 'hoff', V('h')), ('set', 'before', V('h')),
                                                                   ('call', 'setup', [])]
enc += lanes_start(False)
enc += epochs(lambda i: [('call', 'encodeByteL', [i])])
# every coder's last byte, then the payload: n, the lengths of every stream's output but the last, the outputs
fin = [('call', 'laneLoad', [V('j')]), ('set', 'lane', V('j')), ('set', 'cap', lane_cap(V('j'))), ('call', 'emitL', [div(G('x1'), 1 << 24)]),
       ('call', 'laneSave', [V('j')])]
enc += [('loop', G('sk'), 'j', fin)]
put = [('loop', 4, 'q', [('store', 'OUT', 1, add(V('at_'), V('q')), mod(div(V('val'), rom('POW2', P.roms['POW2'][0], mul(8, sub(3, V('q'))))), 256))])]
enc += [('let', 'at_', 0), ('let', 'val', G('n'))] + put
lens = [('let', 'at_', add(4, mul(4, V('j')))), ('let', 'val', ld('LG', 8, add(mul(V('j'), LGSPAN), LANE_G.index('op'))))] + put
enc += [('loop', sub(G('sk'), 1), 'j', lens), ('set', 'flen', add(4, mul(4, sub(G('sk'), 1))))]
cpo = [('let', 'oj', ld('LG', 8, add(mul(V('j'), LGSPAN), LANE_G.index('op')))),
       ('loop', V('oj'), 'q', [('store', 'OUT', 1, add(G('flen'), V('q')), ld('OUTL', 1, add(mul(V('j'), G('osp')), V('q'))))]),
       ('set', 'flen', add(G('flen'), V('oj')))]
enc += [('loop', G('sk'), 'j', cpo)]
P.func('encodeStreams', ['size', 'h', 'pf'], enc)


def word(prof, k, epoch):
    if k == 1:
        return prof
    return prof | (k - 1) << STREAMS_AT | (epoch.bit_length() - 1) << EPOCH_AT


def py_encode_streams(data, history=b'', prof=nm.FULL, k=2, epoch=4096):
    m = Machine(P, data)
    for i, b in enumerate(history):
        m.t['H'][i] = b
    m.call('encodeStreams', len(data), len(history), word(prof, k, epoch) if k > 1 else prof | 0)
    return bytes(m.t['OUT'].get(i, 0) for i in range(m.g['flen']))


def py_decode_streams(payload, history=b'', prof=nm.FULL, k=2, epoch=4096):
    m = Machine(P, payload)
    for i, b in enumerate(history):
        m.t['H'][i] = b
    m.call('decodeStreams', len(history), word(prof, k, epoch))
    h = len(history)
    return bytes(m.t['H'].get(h + i, 0) for i in range(m.g['n']))
