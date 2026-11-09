# The ulz decoder as a UORC program graph (scratch writing aid; mirrors gen_dec2.py step for step).
# Everything the decoder keeps lives in one store: probabilities, scalars, and the output.
import sys
import os
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from g import Graph, N, Y, P, S, T

# scalars take the low keys (one-byte operands on the wire); probabilities follow at PROBS
RANGE, CODE, POS, SIZE, TOTAL, STATE = 0, 4, 8, 16, 24, 32
REP = [40, 48, 56, 64]
KM, KF, KBIT, KACC, KW, KVAL, KLEN, KDIST = 72, 74, 75, 80, 88, 96, 104, 112
PROBS = 128
PAY = 1 << 48
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
    # the payload is copied into the store at PAY; a key past its end reads the default 0
    b = g.block('byteAt', [T, N], N)
    st, pos = b.arg(0), b.arg(1)
    b.returns(b.b2n(b.sread(st, b.add(PAY, pos))))
    BYTEAT = b
    b = g.block('payloadByte', [N, T, S], T)
    i, st, pl = b.arg(0), b.arg(1), b.arg(2)
    b.returns(b.swrite(st, b.add(PAY, i), b.sidx(pl, i)))
    PAYB = b

    # one getter and setter per width, so a use names only the store and the key
    GETW, PUTW = {}, {}
    for w in (2, 4, 8):
        bb = g.block('get%d' % w, [T, N], N)
        bb.returns(bb.fold(w, bb.nat(0), getStep, [bb.arg(0), bb.arg(1)]))
        GETW[w] = bb
        bb = g.block('put%d' % w, [T, N, N], T)
        bb.returns(bb.fold(w, bb.arg(0), putStep, [bb.arg(1), bb.arg(2), w]))
        PUTW[w] = bb
    def get(blk, st, k):
        if W[k] == 1: return blk.b2n(blk.sread(st, k))
        return blk.apply(GETW[W[k]], [st, k])
    def put(blk, st, k, v):
        if W[k] == 1: return blk.swrite(st, k, blk.n2b(v))
        return blk.apply(PUTW[W[k]], [st, k, v])

    # normalize
    b = g.block('normalizeShift', [N, T], T)
    i, st = (b.arg(k) for k in range(2))
    pos = get(b, st, POS)
    st1 = put(b, st, RANGE, b.mul(256, get(b, st, RANGE)))
    code = b.add(b.mul(256, get(b, st1, CODE)), b.apply(BYTEAT, [st1, pos]))
    st2 = put(b, st1, CODE, code)
    b.returns(put(b, st2, POS, b.add(pos, 1)))
    NSHIFT = b
    b = g.block('normalize', [T], T)
    st = b.arg(0)
    b.returns(b.when(b.lt(get(b, st, RANGE), 1 << 24), st, NSHIFT, []))
    NORM = b

    # decodeBit: the bit lands at KBIT
    b = g.block('decodeBit', [T, N], T)
    st, at = (b.arg(k) for k in range(2))
    pk = b.add(b.mul(2, at), PROBS)
    v = b.bxor(b.apply(GETW[2], [st, pk]), 1024)
    r = get(b, st, RANGE); c = get(b, st, CODE)
    bound = b.mul(b.div(r, 2048), v)
    z = b.lt(c, bound)
    nr = b.choose(z, bound, b.apply(MONUS, [r, bound]))
    nc = b.apply(MONUS, [c, b.choose(z, 0, bound)])
    nv = b.choose(z, b.add(v, b.div(b.apply(MONUS, [2048, v]), 32)), b.apply(MONUS, [v, b.div(v, 32)]))
    s1 = put(b, st, RANGE, nr); s2 = put(b, s1, CODE, nc)
    s3 = b.apply(PUTW[2], [s2, pk, b.bxor(nv, 1024)])
    s4 = b.swrite(s3, KBIT, b.n2b(b.choose(z, 0, 1)))
    b.returns(b.apply(NORM, [s4]))
    BIT = b
    def bit_of(blk, st): return blk.b2n(blk.sread(st, KBIT))

    # directLoop step: acc at KACC
    b = g.block('directStep', [N, T], T)
    i, st = (b.arg(k) for k in range(2))
    r = get(b, st, RANGE); c = get(b, st, CODE)
    half = b.div(r, 2)
    z = b.lt(c, half)
    s1 = put(b, st, RANGE, half)
    s2 = put(b, s1, CODE, b.apply(MONUS, [c, b.choose(z, 0, half)]))
    s3 = put(b, s2, KACC, b.add(b.mul(2, get(b, s2, KACC)), b.choose(z, 0, 1)))
    b.returns(b.apply(NORM, [s3]))
    DIRB = b

    # tree: m at KM, result m - 2^bits at KVAL
    b = g.block('treeStep', [N, T, N], T)
    i, st, base = (b.arg(k) for k in range(3))
    m = get(b, st, KM)
    s1 = b.apply(BIT, [st, b.add(base, m)])
    b.returns(put(b, s1, KM, b.add(b.mul(2, m), bit_of(b, s1))))
    TREEB = b
    b = g.block('tree', [T, N, N], T)
    st, base, bits = (b.arg(k) for k in range(3))
    s1 = b.fold(bits, put(b, st, KM, b.nat(1)), TREEB, [base])
    b.returns(put(b, s1, KVAL, b.sub(get(b, s1, KM), b.shl(1, bits))))
    TREE = b

    # reverse: m at KM, weight at KW, acc at KACC
    b = g.block('reverseStep', [N, T, N], T)
    i, st, base = (b.arg(k) for k in range(3))
    m = get(b, st, KM); w = get(b, st, KW); acc = get(b, st, KACC)
    s1 = b.apply(BIT, [st, b.add(base, m)])
    bit = bit_of(b, s1)
    s2 = put(b, s1, KM, b.add(b.mul(2, m), bit))
    s3 = put(b, s2, KACC, b.add(acc, b.mul(bit, w)))
    b.returns(put(b, s3, KW, b.mul(2, w)))
    REVB = b
    b = g.block('reverse', [T, N, N], T)
    st, base, count = (b.arg(k) for k in range(3))
    s1 = put(b, put(b, put(b, st, KM, b.nat(1)), KW, 1), KACC, 0)
    b.returns(b.fold(count, s1, REVB, [base]))
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
    b = g.block('literalStep', [N, T, N, N], T)
    i, st, base, mb = (b.arg(k) for k in range(4))
    k = b.sub(7, i)
    sym = get(b, st, KM)
    flag = b.eq(b.b2n(b.sread(st, KF)), 1)
    mbit = b.band(b.shr(mb, k), 1)
    ctx = b.choose(flag, b.add(b.add(base, b.mul(b.add(1, mbit), 256)), sym), b.add(base, sym))
    s1 = b.apply(BIT, [st, ctx])
    bit = bit_of(b, s1)
    s2 = put(b, s1, KM, b.add(b.mul(2, sym), bit))
    b.returns(b.swrite(s2, KF, b.n2b(b.choose(b.pand(flag, b.eq(bit, mbit)), 1, 0))))
    LITB = b
    def after(blk, s, kind):
        if kind == 'literal':
            return blk.choose(blk.lt(s, 4), 0, blk.choose(blk.lt(s, 10), blk.apply(MONUS, [s, 3]), blk.apply(MONUS, [s, 6])))
        lo, hi = {'match': (7, 10), 'rep': (8, 11), 'short': (9, 11)}[kind]
        return blk.choose(blk.lt(s, 7), lo, hi)
    b = g.block('literal', [N, T], T)
    i, st = (b.arg(k) for k in range(2))
    s0 = get(b, st, STATE); size = get(b, st, SIZE)
    prev = b.choose(b.eq(size, 0), 0, b.b2n(b.sread(st, b.add(OUT, b.apply(MONUS, [size, 1])))))
    base = b.add(LIT, b.mul(768, b.div(prev, 32)))
    mb = b.choose(b.lt(get(b, st, REP[0]), size), b.apply(BACK, [st]), 0)
    s1 = b.swrite(put(b, st, KM, b.nat(1)), KF, b.n2b(b.choose(b.lt(s0, 7), 0, 1)))
    s2 = b.fold(8, s1, LITB, [base, mb])
    s3 = b.apply(EMIT, [s2, b.apply(MONUS, [get(b, s2, KM), 256])])
    b.returns(put(b, s3, STATE, after(b, s0, 'literal')))
    LITERAL = b

    # length: result at KLEN
    def tree_branch(name, off, bits, add):
        bb = g.block(name, [N, T, N], T)
        i, st, base = (bb.arg(k) for k in range(3))
        s1 = bb.apply(TREE, [st, bb.add(base, off), bits])
        bb.returns(put(bb, s1, KLEN, bb.add(add, get(bb, s1, KVAL))))
        return bb
    LEN0 = tree_branch('lengthLow', 2, 3, 0)
    LEN1 = tree_branch('lengthMid', 18, 3, 8)
    LEN2 = tree_branch('lengthHigh', 34, 8, 16)
    b = g.block('lengthChoice', [N, T, N], T)
    i, st, base = (b.arg(k) for k in range(3))
    s1 = b.apply(BIT, [st, b.add(base, 1)])
    hi = b.eq(bit_of(b, s1), 1)
    s2 = b.when(b.pnot(hi), s1, LEN1, [base])
    b.returns(b.when(hi, s2, LEN2, [base]))
    LENC = b
    b = g.block('length', [T, N], T)
    st, base = (b.arg(k) for k in range(2))
    s1 = b.apply(BIT, [st, base])
    lo = b.eq(bit_of(b, s1), 0)
    s2 = b.when(lo, s1, LEN0, [base])
    b.returns(b.when(b.pnot(lo), s2, LENC, [base]))
    LENGTH = b

    # distance: result at KDIST
    b = g.block('distanceShort', [N, T, N, N, N], T)   # slot in [4, 14)
    i, st, slot, fb, base = (b.arg(k) for k in range(5))
    s1 = b.apply(REV, [st, b.sub(b.add(SPEC, base), slot), fb])
    b.returns(put(b, s1, KDIST, b.add(base, get(b, s1, KACC))))
    DSHORT = b
    b = g.block('distanceLong', [N, T, N, N, N], T)    # slot >= 14
    i, st, slot, fb, base = (b.arg(k) for k in range(5))
    s1 = b.fold(b.sub(fb, 4), put(b, st, KACC, b.nat(0)), DIRB, [])
    hv = get(b, s1, KACC)
    s2 = b.apply(REV, [s1, ALIGN, 4])
    b.returns(put(b, s2, KDIST, b.add(b.add(base, b.mul(16, hv)), get(b, s2, KACC))))
    DLONG = b
    b = g.block('distance', [T, N], T)
    st, lenCode = (b.arg(k) for k in range(2))
    ls = b.choose(b.lt(lenCode, 3), lenCode, 3)
    s1 = b.apply(TREE, [st, b.add(SLOT, b.mul(ls, 64)), 6])
    slot = get(b, s1, KVAL)
    small = b.lt(slot, 4)
    s2 = put(b, s1, KDIST, slot)
    fb = b.apply(MONUS, [b.div(slot, 2), 1])
    base = b.shl(b.add(2, b.rem(slot, 2)), b.choose(small, 0, fb))
    mid = b.pand(b.pnot(small), b.lt(slot, 14))
    s3 = b.when(mid, s2, DSHORT, [slot, fb, base])
    b.returns(b.when(b.pnot(b.lt(slot, 14)), s3, DLONG, [slot, fb, base]))
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

    b = g.block('newMatch', [N, T], T)
    i, st = (b.arg(k) for k in range(2))
    s1 = b.apply(LENGTH, [st, LENL])
    ln = get(b, s1, KLEN)
    s2 = b.apply(DIST, [s1, ln])
    dv = get(b, s2, KDIST)
    r0, r1, r2 = get(b, s2, REP[0]), get(b, s2, REP[1]), get(b, s2, REP[2])
    s3 = put(b, put(b, put(b, put(b, s2, REP[3], r2), REP[2], r1), REP[1], r0), REP[0], dv)
    s4 = put(b, s3, STATE, after(b, get(b, s3, STATE), 'match'))
    b.returns(b.fold(b.add(ln, 2), s4, COPYB, []))
    NEWMATCH = b

    b = g.block('repMatch', [N, T], T)
    i, st = (b.arg(k) for k in range(2))
    s1 = b.apply(LENGTH, [st, LENR])
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
        bb = g.block(name, [N, T], T)
        i, st = (bb.arg(k) for k in range(2))
        vals = [get(bb, st, REP[k]) for k in range(4)]
        s = st
        for k, src in enumerate(order):
            s = put(bb, s, REP[k], vals[src])
        bb.returns(bb.apply(REPMATCH, [i, s]) if False else bb.fold(bb.nat(1), s, REPMATCH, []))
        return bb
    ROT1 = rot('rep1', [1, 0])
    ROT2 = rot('rep2', [2, 0, 1])
    ROT3 = rot('rep3', [3, 0, 1, 2])
    b = g.block('repLong', [N, T], T)       # G0 = 0 and long: repMatch at rep0
    i, st = (b.arg(k) for k in range(2))
    b.returns(b.fold(b.nat(1), st, REPMATCH, []))
    REPLONG = b
    b = g.block('rep0', [N, T], T)
    i, st = (b.arg(k) for k in range(2))
    s = get(b, st, STATE)
    s1 = b.apply(BIT, [st, b.add(60, s)])
    lng = b.eq(bit_of(b, s1), 1)
    s2 = b.when(b.pnot(lng), s1, SHORT, [])
    b.returns(b.when(lng, s2, REPLONG, []))
    REP0 = b
    b = g.block('repG2', [N, T], T)
    i, st = (b.arg(k) for k in range(2))
    s = get(b, st, STATE)
    s1 = b.apply(BIT, [st, b.add(48, s)])
    three = b.eq(bit_of(b, s1), 1)
    s2 = b.when(b.pnot(three), s1, ROT2, [])
    b.returns(b.when(three, s2, ROT3, []))
    REPG2 = b
    b = g.block('repG1', [N, T], T)
    i, st = (b.arg(k) for k in range(2))
    s = get(b, st, STATE)
    s1 = b.apply(BIT, [st, b.add(36, s)])
    more = b.eq(bit_of(b, s1), 1)
    s2 = b.when(b.pnot(more), s1, ROT1, [])
    b.returns(b.when(more, s2, REPG2, []))
    REPG1 = b
    b = g.block('rep', [N, T], T)
    i, st = (b.arg(k) for k in range(2))
    s = get(b, st, STATE)
    s1 = b.apply(BIT, [st, b.add(24, s)])
    g0 = b.eq(bit_of(b, s1), 1)
    s2 = b.when(b.pnot(g0), s1, REP0, [])
    b.returns(b.when(g0, s2, REPG1, []))
    REPB = b
    b = g.block('matchKind', [N, T], T)
    i, st = (b.arg(k) for k in range(2))
    s = get(b, st, STATE)
    s1 = b.apply(BIT, [st, b.add(12, s)])
    isrep = b.eq(bit_of(b, s1), 1)
    s2 = b.when(b.pnot(isrep), s1, NEWMATCH, [])
    b.returns(b.when(isrep, s2, REPB, []))
    MKIND = b
    b = g.block('item', [N, T], T)
    i, st = (b.arg(k) for k in range(2))
    s1 = b.apply(BIT, [st, get(b, st, STATE)])
    ism = b.eq(bit_of(b, s1), 1)
    s2 = b.when(b.pnot(ism), s1, LITERAL, [])
    b.returns(b.when(ism, s2, MKIND, []))
    ITEM = b
    b = g.block('step', [N, T], T)
    i, st = (b.arg(k) for k in range(2))
    b.returns(b.when(b.lt(get(b, st, SIZE), get(b, st, TOTAL)), st, ITEM, []))
    STEP = b
    b = g.block('lengthByte', [N, N, T], N)   # little-endian total, top byte first
    i, acc, st = (b.arg(k) for k in range(3))
    b.returns(b.add(b.mul(256, acc), b.apply(BYTEAT, [st, b.sub(7, i)])))
    LENB = b
    b = g.block('outByte', [N, T], Y)
    i, st = b.arg(0), b.arg(1)
    b.returns(b.sread(st, b.add(OUT, i)))
    OUTB = b

    b = g.block('entry', [], S)
    pl = b.seq(payload)
    st = b.fold(b.slen(pl), b.snew(b.byte(0)), PAYB, [pl])
    total = b.fold(8, b.nat(0), LENB, [st])
    code = b.nat(0)
    for k in range(9, 13):
        code = b.add(b.mul(256, code), b.apply(BYTEAT, [st, k]))
    st = put(b, st, RANGE, 0xFFFFFFFF); st = put(b, st, CODE, code); st = put(b, st, POS, 13); st = put(b, st, TOTAL, total)
    st = b.fold(b.add(total, 1), st, STEP, [])
    b.returns(b.tabulate(get(b, st, SIZE), OUTB, [st]))
    return g, b
