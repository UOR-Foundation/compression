# The ulz decoder as a UORC program graph (scratch writing aid; mirrors gen_dec2.py step for step).
# Everything the decoder keeps lives in one store: probabilities, scalars, and the output.
import sys
import os
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from g import Graph, N, Y, P, S, T

RANGE, CODE, POS, SIZE, TOTAL, STATE = 20000, 20004, 20008, 20016, 20024, 20032
REP = [20040, 20048, 20056, 20064]
KM, KF, KBIT, KACC, KW, KVAL, KLEN, KDIST = 20072, 20074, 20075, 20080, 20088, 20096, 20104, 20112
OUT = 1 << 20
SLOT, SPEC, ALIGN, LENL, LENR, LIT = 6216, 6472, 6587, 6603, 7149, 72
W = {RANGE: 4, CODE: 4, POS: 8, SIZE: 8, TOTAL: 8, STATE: 1, KM: 2, KACC: 8, KW: 8, KVAL: 8, KLEN: 8, KDIST: 8}
for r in REP: W[r] = 8

def build(payload):
    g = Graph()
    # get: acc*256 + store[key+i]
    b = g.block('getStep', [N, N, T, N], N)
    i, acc, st, key = (b.arg(k) for k in range(4))
    b.returns(b.add(b.mul(256, acc), b.b2n(b.sread(st, b.add(key, i)))))
    getStep = b
    b = g.block('get', [T, N, N], N)
    st, key, w = (b.arg(k) for k in range(3))
    b.returns(b.fold(w, b.nat(0), getStep, [st, key]))
    GET = b
    b = g.block('putStep', [N, T, N, N, N], T)
    i, st, key, value, w = (b.arg(k) for k in range(5))
    shift = b.mul(8, b.sub(b.sub(w, 1), i))
    b.returns(b.swrite(st, b.add(key, i), b.n2b(b.band(b.shr(value, shift), 255))))
    putStep = b
    b = g.block('put', [T, N, N, N], T)
    st, key, value, w = (b.arg(k) for k in range(4))
    b.returns(b.fold(w, st, putStep, [key, value, w]))
    PUT = b
    # saturating a - b
    b = g.block('monus', [N, N], N)
    x, y = b.arg(0), b.arg(1)
    b.returns(b.sub(x, b.choose(b.lt(y, x), y, x)))
    MONUS = b
    b = g.block('byteAt', [S, N, N], N)
    p0, plen, pos = (b.arg(k) for k in range(3))
    b.returns(b.b2n(b.sidx(p0, b.choose(b.lt(pos, plen), pos, plen))))
    BYTEAT = b

    def get(blk, st, k): return blk.apply(GET, [st, k, W[k]])
    def put(blk, st, k, v): return blk.apply(PUT, [st, k, v, W[k]])

    # normalize
    b = g.block('normalizeShift', [N, T, S, N], T)
    i, st, p0, plen = (b.arg(k) for k in range(4))
    pos = get(b, st, POS)
    st1 = put(b, st, RANGE, b.mul(256, get(b, st, RANGE)))
    code = b.add(b.mul(256, get(b, st1, CODE)), b.apply(BYTEAT, [p0, plen, pos]))
    st2 = put(b, st1, CODE, code)
    b.returns(put(b, st2, POS, b.add(pos, 1)))
    NSHIFT = b
    b = g.block('normalize', [T, S, N], T)
    st, p0, plen = (b.arg(k) for k in range(3))
    b.returns(b.when(b.lt(get(b, st, RANGE), 1 << 24), st, NSHIFT, [p0, plen]))
    NORM = b

    # decodeBit: the bit lands at KBIT
    b = g.block('decodeBit', [T, N, S, N], T)
    st, at, p0, plen = (b.arg(k) for k in range(4))
    pk = b.mul(2, at)
    v = b.bxor(b.apply(GET, [st, pk, 2]), 1024)
    r = get(b, st, RANGE); c = get(b, st, CODE)
    bound = b.mul(b.div(r, 2048), v)
    z = b.lt(c, bound)
    nr = b.choose(z, bound, b.apply(MONUS, [r, bound]))
    nc = b.apply(MONUS, [c, b.choose(z, 0, bound)])
    nv = b.choose(z, b.add(v, b.div(b.apply(MONUS, [2048, v]), 32)), b.apply(MONUS, [v, b.div(v, 32)]))
    s1 = put(b, st, RANGE, nr); s2 = put(b, s1, CODE, nc)
    s3 = b.apply(PUT, [s2, pk, b.bxor(nv, 1024), 2])
    s4 = b.swrite(s3, KBIT, b.n2b(b.choose(z, 0, 1)))
    b.returns(b.apply(NORM, [s4, p0, plen]))
    BIT = b
    def bit_of(blk, st): return blk.b2n(blk.sread(st, KBIT))

    # directLoop step: acc at KACC
    b = g.block('directStep', [N, T, S, N], T)
    i, st, p0, plen = (b.arg(k) for k in range(4))
    r = get(b, st, RANGE); c = get(b, st, CODE)
    half = b.div(r, 2)
    z = b.lt(c, half)
    s1 = put(b, st, RANGE, half)
    s2 = put(b, s1, CODE, b.apply(MONUS, [c, b.choose(z, 0, half)]))
    s3 = put(b, s2, KACC, b.add(b.mul(2, get(b, s2, KACC)), b.choose(z, 0, 1)))
    b.returns(b.apply(NORM, [s3, p0, plen]))
    DIRB = b

    # tree: m at KM, result m - 2^bits at KVAL
    b = g.block('treeStep', [N, T, N, S, N], T)
    i, st, base, p0, plen = (b.arg(k) for k in range(5))
    m = get(b, st, KM)
    s1 = b.apply(BIT, [st, b.add(base, m), p0, plen])
    b.returns(put(b, s1, KM, b.add(b.mul(2, m), bit_of(b, s1))))
    TREEB = b
    b = g.block('tree', [T, N, N, S, N], T)
    st, base, bits, p0, plen = (b.arg(k) for k in range(5))
    s1 = b.fold(bits, put(b, st, KM, b.nat(1)), TREEB, [base, p0, plen])
    b.returns(put(b, s1, KVAL, b.sub(get(b, s1, KM), b.shl(1, bits))))
    TREE = b

    # reverse: m at KM, weight at KW, acc at KACC
    b = g.block('reverseStep', [N, T, N, S, N], T)
    i, st, base, p0, plen = (b.arg(k) for k in range(5))
    m = get(b, st, KM); w = get(b, st, KW); acc = get(b, st, KACC)
    s1 = b.apply(BIT, [st, b.add(base, m), p0, plen])
    bit = bit_of(b, s1)
    s2 = put(b, s1, KM, b.add(b.mul(2, m), bit))
    s3 = put(b, s2, KACC, b.add(acc, b.mul(bit, w)))
    b.returns(put(b, s3, KW, b.mul(2, w)))
    REVB = b
    b = g.block('reverse', [T, N, N, S, N], T)
    st, base, count, p0, plen = (b.arg(k) for k in range(5))
    s1 = put(b, put(b, put(b, st, KM, b.nat(1)), KW, 1), KACC, 0)
    b.returns(b.fold(count, s1, REVB, [base, p0, plen]))
    REV = b

    # output
    b = g.block('emit', [T, N], T)
    st, value = b.arg(0), b.arg(1)
    size = get(b, st, SIZE)
    s1 = b.swrite(st, b.add(OUT, size), b.n2b(value))
    b.returns(put(b, s1, SIZE, b.add(size, 1)))
    EMIT = b
    b = g.block('back', [T], N)   # the byte rep0 back: out[size - (rep0 + 1)], saturating
    st = b.arg(0)
    at = b.apply(MONUS, [get(b, st, SIZE), b.add(get(b, st, REP[0]), 1)])
    b.returns(b.b2n(b.sread(st, b.add(OUT, at))))
    BACK = b

    # literal, matched or plain: sym at KM, still-matching flag at KF
    b = g.block('literalStep', [N, T, N, N, S, N], T)
    i, st, base, mb, p0, plen = (b.arg(k) for k in range(6))
    k = b.sub(7, i)
    sym = get(b, st, KM)
    flag = b.eq(b.b2n(b.sread(st, KF)), 1)
    mbit = b.band(b.shr(mb, k), 1)
    ctx = b.choose(flag, b.add(b.add(base, b.mul(b.add(1, mbit), 256)), sym), b.add(base, sym))
    s1 = b.apply(BIT, [st, ctx, p0, plen])
    bit = bit_of(b, s1)
    s2 = put(b, s1, KM, b.add(b.mul(2, sym), bit))
    b.returns(b.swrite(s2, KF, b.n2b(b.choose(b.pand(flag, b.eq(bit, mbit)), 1, 0))))
    LITB = b
    def after(blk, s, kind):
        if kind == 'literal':
            return blk.choose(blk.lt(s, 4), 0, blk.choose(blk.lt(s, 10), blk.apply(MONUS, [s, 3]), blk.apply(MONUS, [s, 6])))
        lo, hi = {'match': (7, 10), 'rep': (8, 11), 'short': (9, 11)}[kind]
        return blk.choose(blk.lt(s, 7), lo, hi)
    b = g.block('literal', [N, T, S, N], T)
    i, st, p0, plen = (b.arg(k) for k in range(4))
    s0 = get(b, st, STATE); size = get(b, st, SIZE)
    prev = b.choose(b.eq(size, 0), 0, b.b2n(b.sread(st, b.add(OUT, b.apply(MONUS, [size, 1])))))
    base = b.add(LIT, b.mul(768, b.div(prev, 32)))
    mb = b.choose(b.lt(get(b, st, REP[0]), size), b.apply(BACK, [st]), 0)
    s1 = b.swrite(put(b, st, KM, b.nat(1)), KF, b.n2b(b.choose(b.lt(s0, 7), 0, 1)))
    s2 = b.fold(8, s1, LITB, [base, mb, p0, plen])
    s3 = b.apply(EMIT, [s2, b.apply(MONUS, [get(b, s2, KM), 256])])
    b.returns(put(b, s3, STATE, after(b, s0, 'literal')))
    LITERAL = b

    # length: result at KLEN
    def tree_branch(name, off, bits, add):
        bb = g.block(name, [N, T, N, S, N], T)
        i, st, base, p0, plen = (bb.arg(k) for k in range(5))
        s1 = bb.apply(TREE, [st, bb.add(base, off), bits, p0, plen])
        bb.returns(put(bb, s1, KLEN, bb.add(add, get(bb, s1, KVAL))))
        return bb
    LEN0 = tree_branch('lengthLow', 2, 3, 0)
    LEN1 = tree_branch('lengthMid', 18, 3, 8)
    LEN2 = tree_branch('lengthHigh', 34, 8, 16)
    b = g.block('lengthChoice', [N, T, N, S, N], T)
    i, st, base, p0, plen = (b.arg(k) for k in range(5))
    s1 = b.apply(BIT, [st, b.add(base, 1), p0, plen])
    hi = b.eq(bit_of(b, s1), 1)
    s2 = b.when(b.pnot(hi), s1, LEN1, [base, p0, plen])
    b.returns(b.when(hi, s2, LEN2, [base, p0, plen]))
    LENC = b
    b = g.block('length', [T, N, S, N], T)
    st, base, p0, plen = (b.arg(k) for k in range(4))
    s1 = b.apply(BIT, [st, base, p0, plen])
    lo = b.eq(bit_of(b, s1), 0)
    s2 = b.when(lo, s1, LEN0, [base, p0, plen])
    b.returns(b.when(b.pnot(lo), s2, LENC, [base, p0, plen]))
    LENGTH = b

    # distance: result at KDIST
    b = g.block('distanceShort', [N, T, N, N, N, S, N], T)   # slot in [4, 14)
    i, st, slot, fb, base, p0, plen = (b.arg(k) for k in range(7))
    s1 = b.apply(REV, [st, b.sub(b.add(SPEC, base), slot), fb, p0, plen])
    b.returns(put(b, s1, KDIST, b.add(base, get(b, s1, KACC))))
    DSHORT = b
    b = g.block('distanceLong', [N, T, N, N, N, S, N], T)    # slot >= 14
    i, st, slot, fb, base, p0, plen = (b.arg(k) for k in range(7))
    s1 = b.fold(b.sub(fb, 4), put(b, st, KACC, b.nat(0)), DIRB, [p0, plen])
    hv = get(b, s1, KACC)
    s2 = b.apply(REV, [s1, ALIGN, 4, p0, plen])
    b.returns(put(b, s2, KDIST, b.add(b.add(base, b.mul(16, hv)), get(b, s2, KACC))))
    DLONG = b
    b = g.block('distance', [T, N, S, N], T)
    st, lenCode, p0, plen = (b.arg(k) for k in range(4))
    ls = b.choose(b.lt(lenCode, 3), lenCode, 3)
    s1 = b.apply(TREE, [st, b.add(SLOT, b.mul(ls, 64)), 6, p0, plen])
    slot = get(b, s1, KVAL)
    small = b.lt(slot, 4)
    s2 = put(b, s1, KDIST, slot)
    fb = b.apply(MONUS, [b.div(slot, 2), 1])
    base = b.shl(b.add(2, b.rem(slot, 2)), b.choose(small, 0, fb))
    mid = b.pand(b.pnot(small), b.lt(slot, 14))
    s3 = b.when(mid, s2, DSHORT, [slot, fb, base, p0, plen])
    b.returns(b.when(b.pnot(b.lt(slot, 14)), s3, DLONG, [slot, fb, base, p0, plen]))
    DIST = b

    # copy
    b = g.block('copyByte', [N, T], T)
    i, st = b.arg(0), b.arg(1)
    b.returns(b.apply(EMIT, [st, b.apply(BACK, [st])]))
    COPYBYTE = b
    b = g.block('copyStep', [N, T], T)
    i, st = b.arg(0), b.arg(1)
    b.returns(b.when(b.lt(get(b, st, SIZE), get(b, st, TOTAL)), st, COPYBYTE, []))
    COPYB = b

    b = g.block('newMatch', [N, T, S, N], T)
    i, st, p0, plen = (b.arg(k) for k in range(4))
    s1 = b.apply(LENGTH, [st, LENL, p0, plen])
    ln = get(b, s1, KLEN)
    s2 = b.apply(DIST, [s1, ln, p0, plen])
    dv = get(b, s2, KDIST)
    r0, r1, r2 = get(b, s2, REP[0]), get(b, s2, REP[1]), get(b, s2, REP[2])
    s3 = put(b, put(b, put(b, put(b, s2, REP[3], r2), REP[2], r1), REP[1], r0), REP[0], dv)
    s4 = put(b, s3, STATE, after(b, get(b, s3, STATE), 'match'))
    b.returns(b.fold(b.add(ln, 2), s4, COPYB, []))
    NEWMATCH = b

    b = g.block('repMatch', [N, T, S, N], T)
    i, st, p0, plen = (b.arg(k) for k in range(4))
    s1 = b.apply(LENGTH, [st, LENR, p0, plen])
    ln = get(b, s1, KLEN)
    s2 = put(b, s1, STATE, after(b, get(b, s1, STATE), 'rep'))
    b.returns(b.fold(b.add(ln, 2), s2, COPYB, []))
    REPMATCH = b
    b = g.block('shortRep', [N, T], T)
    i, st = b.arg(0), b.arg(1)
    s0 = get(b, st, STATE)
    s1 = b.apply(EMIT, [st, b.apply(BACK, [st])])
    b.returns(put(b, s1, STATE, after(b, s0, 'short')))
    SHORT = b
    # rep distance rotations, then repMatch
    def rot(name, order):
        bb = g.block(name, [N, T, S, N], T)
        i, st, p0, plen = (bb.arg(k) for k in range(4))
        vals = [get(bb, st, REP[k]) for k in range(4)]
        s = st
        for k, src in enumerate(order):
            s = put(bb, s, REP[k], vals[src])
        bb.returns(bb.apply(REPMATCH, [i, s, p0, plen]) if False else bb.fold(bb.nat(1), s, REPMATCH, [p0, plen]))
        return bb
    ROT1 = rot('rep1', [1, 0])
    ROT2 = rot('rep2', [2, 0, 1])
    ROT3 = rot('rep3', [3, 0, 1, 2])
    b = g.block('repLong', [N, T, S, N], T)       # G0 = 0 and long: repMatch at rep0
    i, st, p0, plen = (b.arg(k) for k in range(4))
    b.returns(b.fold(b.nat(1), st, REPMATCH, [p0, plen]))
    REPLONG = b
    b = g.block('rep0', [N, T, S, N], T)
    i, st, p0, plen = (b.arg(k) for k in range(4))
    s = get(b, st, STATE)
    s1 = b.apply(BIT, [st, b.add(60, s), p0, plen])
    lng = b.eq(bit_of(b, s1), 1)
    s2 = b.when(b.pnot(lng), s1, SHORT, [])
    b.returns(b.when(lng, s2, REPLONG, [p0, plen]))
    REP0 = b
    b = g.block('repG2', [N, T, S, N], T)
    i, st, p0, plen = (b.arg(k) for k in range(4))
    s = get(b, st, STATE)
    s1 = b.apply(BIT, [st, b.add(48, s), p0, plen])
    three = b.eq(bit_of(b, s1), 1)
    s2 = b.when(b.pnot(three), s1, ROT2, [p0, plen])
    b.returns(b.when(three, s2, ROT3, [p0, plen]))
    REPG2 = b
    b = g.block('repG1', [N, T, S, N], T)
    i, st, p0, plen = (b.arg(k) for k in range(4))
    s = get(b, st, STATE)
    s1 = b.apply(BIT, [st, b.add(36, s), p0, plen])
    more = b.eq(bit_of(b, s1), 1)
    s2 = b.when(b.pnot(more), s1, ROT1, [p0, plen])
    b.returns(b.when(more, s2, REPG2, [p0, plen]))
    REPG1 = b
    b = g.block('rep', [N, T, S, N], T)
    i, st, p0, plen = (b.arg(k) for k in range(4))
    s = get(b, st, STATE)
    s1 = b.apply(BIT, [st, b.add(24, s), p0, plen])
    g0 = b.eq(bit_of(b, s1), 1)
    s2 = b.when(b.pnot(g0), s1, REP0, [p0, plen])
    b.returns(b.when(g0, s2, REPG1, [p0, plen]))
    REPB = b
    b = g.block('matchKind', [N, T, S, N], T)
    i, st, p0, plen = (b.arg(k) for k in range(4))
    s = get(b, st, STATE)
    s1 = b.apply(BIT, [st, b.add(12, s), p0, plen])
    isrep = b.eq(bit_of(b, s1), 1)
    s2 = b.when(b.pnot(isrep), s1, NEWMATCH, [p0, plen])
    b.returns(b.when(isrep, s2, REPB, [p0, plen]))
    MKIND = b
    b = g.block('item', [N, T, S, N], T)
    i, st, p0, plen = (b.arg(k) for k in range(4))
    s1 = b.apply(BIT, [st, get(b, st, STATE), p0, plen])
    ism = b.eq(bit_of(b, s1), 1)
    s2 = b.when(b.pnot(ism), s1, LITERAL, [p0, plen])
    b.returns(b.when(ism, s2, MKIND, [p0, plen]))
    ITEM = b
    b = g.block('step', [N, T, S, N], T)
    i, st, p0, plen = (b.arg(k) for k in range(4))
    b.returns(b.when(b.lt(get(b, st, SIZE), get(b, st, TOTAL)), st, ITEM, [p0, plen]))
    STEP = b
    b = g.block('lengthByte', [N, N, S, N], N)   # little-endian total, top byte first
    i, acc, p0, plen = (b.arg(k) for k in range(4))
    b.returns(b.add(b.mul(256, acc), b.apply(BYTEAT, [p0, plen, b.sub(7, i)])))
    LENB = b
    b = g.block('outByte', [N, T], Y)
    i, st = b.arg(0), b.arg(1)
    b.returns(b.sread(st, b.add(OUT, i)))
    OUTB = b

    b = g.block('entry', [], S)
    pl = b.seq(payload)
    p0 = b.sconcat(pl, b.single(b.byte(0)))
    plen = b.slen(pl)
    total = b.fold(8, b.nat(0), LENB, [p0, plen])
    code = b.nat(0)
    for k in range(9, 13):
        code = b.add(b.mul(256, code), b.apply(BYTEAT, [p0, plen, k]))
    st = b.snew(b.byte(0))
    st = put(b, st, RANGE, 0xFFFFFFFF); st = put(b, st, CODE, code); st = put(b, st, POS, 13); st = put(b, st, TOTAL, total)
    st = b.fold(b.add(total, 1), st, STEP, [p0, plen])
    b.returns(b.tabulate(get(b, st, SIZE), OUTB, [st]))
    return g, b
