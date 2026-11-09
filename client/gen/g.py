# UORC Graph builder and a Python mirror of Uorc.Core.ReferenceSemantics (scratch writing aid).
# Slots are the block's arguments followed by one value per instruction; an operand is a distance,
# 1 naming the most recent slot. Blocks name earlier blocks by distance in the same way.
N, Y, P, S, T = 'natural', 'byte', 'predicate', 'sequence', 'store'

class Fail(Exception):
    pass

class Ref:
    def __init__(self, b, i, ty):
        self.b, self.i, self.ty = b, i, ty

class Block:
    def __init__(self, g, name, params, result):
        self.g, self.name, self.params, self.result = g, name, params, result
        self.types = list(params)
        self.ins = []          # (op, operands) with operands as Refs / ints / lists
        self.ret = None
        self.index = len(g.blocks)
        g.blocks.append(self)

    def arg(self, k):
        return Ref(self, k, self.params[k])

    def _emit(self, op, ty, *ops):
        for o in ops:
            if isinstance(o, Ref):
                assert o.b is self, (self.name, op)
        self.ins.append((op, ops))
        self.types.append(ty)
        return Ref(self, len(self.types) - 1, ty)

    # constants
    def nat(self, c): return self._emit('naturalConstant', N, c)
    def byte(self, c): return self._emit('byteConstant', Y, c)
    def pred(self, c): return self._emit('predicateConstant', P, c)
    def seq(self, c): return self._emit('sequenceConstant', S, c)

    def _n(self, x):
        if isinstance(x, int):
            if x not in self.g_consts():
                self.g_consts()[x] = self.nat(x)
            return self.g_consts()[x]
        return x

    def g_consts(self):
        if not hasattr(self, '_consts'):
            self._consts = {}
        return self._consts

    def _bin(self, op, ty, a, b):
        a, b = self._n(a), self._n(b)
        assert a.ty == N and b.ty == N, (op, a.ty, b.ty)
        return self._emit(op, ty, a, b)

    def add(self, a, b): return self._bin('naturalAdd', N, a, b)
    def sub(self, a, b): return self._bin('naturalSubtract', N, a, b)
    def mul(self, a, b): return self._bin('naturalMultiply', N, a, b)
    def div(self, a, b): return self._bin('naturalDivide', N, a, b)
    def rem(self, a, b): return self._bin('naturalRemainder', N, a, b)
    def band(self, a, b): return self._bin('naturalAnd', N, a, b)
    def bor(self, a, b): return self._bin('naturalOr', N, a, b)
    def bxor(self, a, b): return self._bin('naturalXor', N, a, b)
    def shl(self, a, b): return self._bin('naturalShiftLeft', N, a, b)
    def shr(self, a, b): return self._bin('naturalShiftRight', N, a, b)
    def eq(self, a, b): return self._bin('naturalEqual', P, a, b)
    def lt(self, a, b): return self._bin('naturalLess', P, a, b)
    def n2b(self, a): return self._emit('naturalToByte', Y, self._n(a))
    def b2n(self, a):
        assert a.ty == Y
        return self._emit('byteToNatural', N, a)
    def pnot(self, a): return self._emit('predicateNot', P, a)
    def pand(self, a, b): return self._emit('predicateAnd', P, a, b)
    def por(self, a, b): return self._emit('predicateOr', P, a, b)
    def slen(self, s): return self._emit('sequenceLength', N, s)
    def sidx(self, s, i): return self._emit('sequenceIndex', Y, s, self._n(i))
    def sslice(self, s, a, c): return self._emit('sequenceSlice', S, s, self._n(a), self._n(c))
    def sconcat(self, a, b): return self._emit('sequenceConcat', S, a, b)
    def single(self, b): return self._emit('singletonSequence', S, b)
    def basis(self, i): return self._emit('basisGet', S, self._n(i))
    def snew(self, b): return self._emit('storeNew', T, b)
    def sread(self, st, k): return self._emit('storeRead', Y, st, self._n(k))
    def swrite(self, st, k, v): return self._emit('storeWrite', T, st, self._n(k), v)
    def choose(self, c, t, f):
        t, f = self._n(t), self._n(f)
        assert c.ty == P and t.ty == f.ty, (c.ty, t.ty, f.ty)
        return self._emit('choose', t.ty, c, t, f)
    def apply(self, blk, args):
        args = [self._n(a) for a in args]
        assert [a.ty for a in args] == blk.params, (self.name, blk.name, [a.ty for a in args], blk.params)
        assert blk.index < self.index
        return self._emit('apply', blk.result, blk, args)
    def fold(self, count, init, blk, caps):
        count = self._n(count); caps = [self._n(c) for c in caps]
        assert blk.params == [N, init.ty] + [c.ty for c in caps] and blk.result == init.ty, (self.name, blk.name)
        assert blk.index < self.index
        return self._emit('fold', init.ty, count, init, blk, caps)
    def tabulate(self, count, blk, caps):
        count = self._n(count); caps = [self._n(c) for c in caps]
        assert blk.params == [N] + [c.ty for c in caps] and blk.result == Y
        return self._emit('tabulate', S, count, blk, caps)
    # fold(0 or 1): runs blk once when c holds, else returns the state untouched
    def when(self, c, state, blk, caps):
        return self.fold(self.choose(c, 1, 0), state, blk, caps)

    def returns(self, r):
        assert r.ty == self.result, (self.name, r.ty, self.result)
        self.ret = r

    # wire-level operands: distances
    def lowered(self):
        out = []
        for k, (op, ops) in enumerate(self.ins):
            here = len(self.params) + k   # slots before this instruction
            def d(r):
                return here - r.i
            def bd(blk):
                return self.index - blk.index
            if op in ('naturalConstant', 'byteConstant', 'predicateConstant', 'sequenceConstant'):
                out.append((op, ops[0]))
            elif op == 'apply':
                out.append((op, bd(ops[0]), [d(a) for a in ops[1]]))
            elif op == 'fold':
                out.append((op, d(ops[0]), d(ops[1]), bd(ops[2]), [d(a) for a in ops[3]]))
            elif op == 'tabulate':
                out.append((op, d(ops[0]), bd(ops[1]), [d(a) for a in ops[2]]))
            else:
                out.append((op,) + tuple(d(a) for a in ops))
        return out, len(self.params) + len(self.ins) - self.ret.i

class Graph:
    def __init__(self):
        self.blocks = []
    def block(self, name, params, result):
        return Block(self, name, params, result)

# ---- reference mirror ----
M64 = (1 << 64) - 1

class Store:
    # Mirrors the functional store, but mutates one shared table and marks the old version stale,
    # so a non-linear use of a store (reading a version after writing past it) is caught.
    __slots__ = ('default', 'm', 'ver', 'box')
    def __init__(self, default, m=None, box=None):
        self.default, self.m = default, (m if m is not None else {})
        self.box = box if box is not None else [0]
        self.ver = self.box[0]
    def live(self):
        if self.ver != self.box[0]: raise RuntimeError('stale store read: the graph uses a store non-linearly')

def run_block(g, blk, args, counter):
    slots = list(args)
    for op, ops in blk.ins:
        counter[0] += 1
        v = lambda r: slots[r.i]
        if op in ('naturalConstant', 'byteConstant', 'predicateConstant'):
            r = ops[0]
        elif op == 'sequenceConstant':
            r = bytes(ops[0])
        elif op == 'naturalAdd':
            r = v(ops[0]) + v(ops[1])
            if r > M64: raise Fail('overflow')
        elif op == 'naturalSubtract':
            if v(ops[1]) > v(ops[0]): raise Fail('negative')
            r = v(ops[0]) - v(ops[1])
        elif op == 'naturalMultiply':
            r = v(ops[0]) * v(ops[1])
            if r > M64: raise Fail('overflow')
        elif op == 'naturalDivide':
            if v(ops[1]) == 0: raise Fail('div0')
            r = v(ops[0]) // v(ops[1])
        elif op == 'naturalRemainder':
            if v(ops[1]) == 0: raise Fail('div0')
            r = v(ops[0]) % v(ops[1])
        elif op == 'naturalAnd': r = v(ops[0]) & v(ops[1])
        elif op == 'naturalOr': r = v(ops[0]) | v(ops[1])
        elif op == 'naturalXor': r = v(ops[0]) ^ v(ops[1])
        elif op == 'naturalShiftLeft':
            if v(ops[1]) >= 64: raise Fail('shift')
            r = v(ops[0]) << v(ops[1])
            if r > M64: raise Fail('overflow')
        elif op == 'naturalShiftRight':
            if v(ops[1]) >= 64: raise Fail('shift')
            r = v(ops[0]) >> v(ops[1])
        elif op == 'naturalEqual' or op == 'byteEqual': r = v(ops[0]) == v(ops[1])
        elif op == 'naturalLess' or op == 'byteLess': r = v(ops[0]) < v(ops[1])
        elif op == 'naturalToByte':
            if v(ops[0]) > 255: raise Fail('byte')
            r = v(ops[0])
        elif op == 'byteToNatural': r = v(ops[0])
        elif op == 'predicateNot': r = not v(ops[0])
        elif op == 'predicateAnd': r = v(ops[0]) and v(ops[1])
        elif op == 'predicateOr': r = v(ops[0]) or v(ops[1])
        elif op == 'sequenceLength': r = len(v(ops[0]))
        elif op == 'sequenceIndex':
            s, i = v(ops[0]), v(ops[1])
            if i >= len(s): raise Fail('index')
            r = s[i]
        elif op == 'sequenceSlice':
            s, a, c = v(ops[0]), v(ops[1]), v(ops[2])
            if a + c > len(s): raise Fail('slice')
            r = s[a:a + c]
        elif op == 'sequenceConcat': r = v(ops[0]) + v(ops[1])
        elif op == 'singletonSequence': r = bytes([v(ops[0])])
        elif op == 'basisGet':
            if v(ops[0]) >= len(BASIS): raise Fail('basis')
            r = bytes(BASIS[v(ops[0])])
        elif op == 'storeNew': r = Store(v(ops[0]))
        elif op == 'storeRead':
            st = v(ops[0]); st.live(); r = st.m.get(v(ops[1]), st.default)
        elif op == 'storeWrite':
            st = v(ops[0]); st.live(); st.m[v(ops[1])] = v(ops[2]); st.box[0] += 1; r = Store(st.default, st.m, st.box)
        elif op == 'choose': r = v(ops[1]) if v(ops[0]) else v(ops[2])
        elif op == 'apply': r = run_block(g, ops[0], [v(a) for a in ops[1]], counter)
        elif op == 'fold':
            n, st = v(ops[0]), v(ops[1]); caps = [v(a) for a in ops[3]]
            for i in range(n):
                st = run_block(g, ops[2], [i, st] + caps, counter)
            r = st
        elif op == 'tabulate':
            n = v(ops[0]); caps = [v(a) for a in ops[2]]
            r = bytes(run_block(g, ops[1], [i] + caps, counter) for i in range(n))
        else:
            raise NotImplementedError(op)
        slots.append(r)
    return slots[blk.ret.i]

BASIS = []  # the basis roots basisGet reads during evaluate


def evaluate(g, entry, counter=None, basis=()):
    counter = counter if counter is not None else [0]
    BASIS[:] = list(basis)
    return run_block(g, entry, [], counter)
