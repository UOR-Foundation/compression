# Compiles an ir.Prog to a UORC program graph (g.py). Everything lives in one store: globals at
# low keys, each table at its own key base, the read-only input as complements (so a key past its
# end reads 255), and the constant tables. The store is threaded linearly through every block.
import sys
import os
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from g import Graph, N, Y, P as PR, S, T
import ir

INPUT = 1 << 56       # input byte i at INPUT + i, stored as 255 - byte
ROMS = 1 << 52        # constant table r at ROMS + (r << 20) + width * i


class Compiler:
    def __init__(self, prog, graph=None):
        self.p = prog
        self.g = graph or Graph()
        # globals at low keys, in declaration order
        self.gkey, k = {}, 0
        for name, w in prog.globals.items():
            self.gkey[name] = k; k += w
        self.romkey = {name: ROMS + (i << 20) for i, name in enumerate(n for n in prog.roms if n != 'POW2')}
        self.getw, self.putw = {}, {}
        self._helpers()
        self._pure()
        self.blocks = {}      # id(body) -> Block
        self.memo, self._keep = {}, []
        self.funcs = {}

    # ---- multi-byte store access ----
    def _helpers(self):
        g = self.g
        b = g.block('getStep', [N, N, T, N], N)
        i, acc, st, key = (b.arg(k) for k in range(4))
        b.returns(b.add(b.mul(256, acc), b.b2n(b.sread(st, b.add(key, i)))))
        step = b
        b = g.block('putStep', [N, T, N, N, N], T)
        i, st, key, value, w = (b.arg(k) for k in range(5))
        sh = b.mul(8, b.sub(b.sub(w, 1), i))
        b.returns(b.swrite(st, b.add(key, i), b.n2b(b.band(b.shr(value, sh), 255))))
        pstep = b
        for w in sorted({w for w in self.p.globals.values()} | {2, 3, 4}):
            if w == 1:
                continue
            bb = g.block('get%d' % w, [T, N], N)
            bb.returns(bb.fold(w, bb.nat(0), step, [bb.arg(0), bb.arg(1)]))
            self.getw[w] = bb
            bb = g.block('put%d' % w, [T, N, N], T)
            bb.returns(bb.fold(w, bb.arg(0), pstep, [bb.arg(1), bb.arg(2), w]))
            self.putw[w] = bb

    def _pure(self):
        g = self.g
        b = g.block('hash', [N], N)
        a = b.arg(0)
        for k, s in zip(ir.HK, ir.HS):
            a = b.band(b.mul(a, k), 0xFFFFFFFF)
            a = b.band(b.add(a, b.shr(a, s)), 0xFFFFFFFF)
        b.returns(a)
        self.hashb = b
        b = g.block('squash', [N], N)
        d = b.arg(0)
        q = b.seq(b''.join(v.to_bytes(2, 'big') for v in ir.SQT))
        i = b.shr(d, 7); w = b.band(d, 127)
        def at(j):
            j2 = b.mul(2, j)
            return b.add(b.mul(256, b.b2n(b.sidx(q, j2))), b.b2n(b.sidx(q, b.add(j2, 1))))
        r = b.add(b.add(b.mul(at(i), b.sub(128, w)), b.mul(at(b.add(i, 1)), w)), 64)
        b.returns(b.shr(r, 7))
        self.squashb = b

    def get(self, b, st, key, w):
        if w == 1:
            return b.b2n(b.sread(st, key))
        return b.apply(self.getw[w], [st, key])

    def put(self, b, st, key, value, w):
        if w == 1:
            return b.swrite(st, key, b.n2b(value))
        return b.apply(self.putw[w], [st, key, value])

    # ---- expressions ----
    def nat(self, b, r):
        return b.choose(r, 1, 0) if r.ty == PR else r

    def pred(self, b, r):
        return r if r.ty == PR else b.lt(b._n(0), r)

    def ex(self, b, e, env, st):
        # common subexpressions within a block, for the same store version and the same bindings
        if isinstance(e, tuple) and e[0] != 'v':
            key = (id(b), id(st), e, tuple(sorted((k, id(v)) for k, v in env.items() if k in self._vars(e))))
            hit = self.memo.get(key)
            if hit is not None:
                return hit
            r = self._ex(b, e, env, st)
            self.memo[key] = r
            self._keep.append((b, st))
            return r
        return self._ex(b, e, env, st)

    def _vars(self, e):
        out = set()
        def w(x):
            if isinstance(x, tuple):
                if x[0] == 'v':
                    out.add(x[1])
                else:
                    for y in x[1:]:
                        w(y)
        w(e)
        return out

    def _ex(self, b, e, env, st):
        if isinstance(e, bool):
            return b.pred(e)
        if isinstance(e, int):
            return b._n(e)
        k = e[0]
        if k == 'v':
            return env[e[1]]
        if k == 'g':
            return self.get(b, st, self.gkey[e[1]], self.p.globals[e[1]])
        if k == 'ld':
            base = self.p.tables[e[1]]; w = e[2]
            idx = self.nat(b, self.ex(b, e[3], env, st))
            key = b.add(base, idx if w == 1 else b.mul(w, idx))
            return self.get(b, st, key, w)
        if k == 'rom':
            idx = self.nat(b, self.ex(b, e[3], env, st))
            if e[1] == 'POW2':
                return b.shl(1, idx)
            w = e[2]
            return self.get(b, st, b.add(self.romkey[e[1]], b.mul(w, idx)), w)
        if k == 'in':
            idx = self.nat(b, self.ex(b, e[1], env, st))
            return b.sub(255, b.b2n(b.sread(st, b.add(INPUT, idx))))
        if k == 'not':
            return b.pnot(self.pred(b, self.ex(b, e[1], env, st)))
        if k == 'hash':
            return b.apply(self.hashb, [self.nat(b, self.ex(b, e[1], env, st))])
        if k == 'squash':
            return b.apply(self.squashb, [self.nat(b, self.ex(b, e[1], env, st))])
        if k == 'sel':
            c = self.pred(b, self.ex(b, e[1], env, st))
            x = self.nat(b, self.ex(b, e[2], env, st)); y = self.nat(b, self.ex(b, e[3], env, st))
            return b.choose(c, x, y)
        if k in ('and', 'or'):
            x = self.pred(b, self.ex(b, e[1], env, st)); y = self.pred(b, self.ex(b, e[2], env, st))
            return b.pand(x, y) if k == 'and' else b.por(x, y)
        x = self.nat(b, self.ex(b, e[1], env, st)); y = self.nat(b, self.ex(b, e[2], env, st))
        if k == 'add': return b.add(x, y)
        if k == 'sub': return b.sub(x, y)
        if k == 'mul': return b.mul(x, y)
        if k == 'div':
            if e[2] in [1 << j for j in range(1, 64)]:
                return b.shr(x, b._n(e[2].bit_length() - 1))
            return b.div(x, y)
        if k == 'mod':
            if e[2] in [1 << j for j in range(1, 65)]:
                return b.band(x, b._n(e[2] - 1))
            return b.rem(x, y)
        if k == 'lt': return b.lt(x, y)
        if k == 'le': return b.pnot(b.lt(y, x))
        if k == 'eq': return b.eq(x, y)
        if k == 'ne': return b.pnot(b.eq(x, y))
        raise KeyError(k)

    # ---- statements ----
    def sub_blocks(self, stmts, bound, name):
        """Builds the blocks of every nested body in stmts before the block that uses them."""
        for s in stmts:
            k = s[0]
            if k == 'if':
                self.body_block(s[2], bound, name + '_t', loop=False)
                if s[3]:
                    self.body_block(s[3], bound, name + '_f', loop=False)
            elif k == 'loop':
                self.body_block(s[3], bound, name + '_l', loop=True, ivar=s[2])
            elif k == 'loopw':
                inner = [('if', s[2], s[4], [])]
                s_ = ('loop', s[1], s[3], inner)
                self._loopw_alias[id(s)] = s_
                self.sub_blocks([s_], bound, name)
            if k == 'let':
                bound = bound | {s[1]}

    def body_block(self, stmts, bound, name, loop, ivar=None):
        inner_bound = set(bound) | ({ivar} if ivar else set())
        caps = [x for x in ir.free_vars(stmts, {ivar} if ivar else set())]
        self.sub_blocks(stmts, inner_bound, name)
        params = [N, T] + [N] * len(caps)
        b = self.g.block(name, params, T)
        env = {x: b.arg(2 + i) for i, x in enumerate(caps)}
        if ivar:
            env[ivar] = b.arg(0)
        st = self.stmts(b, stmts, env, b.arg(1))
        b.returns(st)
        self.blocks[id(stmts)] = (b, caps)
        return b

    def stmts(self, b, stmts, env, st):
        env = dict(env)
        for s in stmts:
            k = s[0]
            if k == 'let':
                env[s[1]] = self.nat(b, self.ex(b, s[2], env, st))
            elif k == 'set':
                v = self.nat(b, self.ex(b, s[2], env, st))
                st = self.put(b, st, self.gkey[s[1]], v, self.p.globals[s[1]])
            elif k == 'store':
                base = self.p.tables[s[1]]; w = s[2]
                idx = self.nat(b, self.ex(b, s[3], env, st)); v = self.nat(b, self.ex(b, s[4], env, st))
                st = self.put(b, st, b.add(base, idx if w == 1 else b.mul(w, idx)), v, w)
            elif k == 'if':
                c = self.pred(b, self.ex(b, s[1], env, st))
                blk, caps = self.blocks[id(s[2])]
                st2 = b.when(c, st, blk, [env[x] for x in caps])
                if s[3]:
                    blk, caps = self.blocks[id(s[3])]
                    st2 = b.when(b.pnot(c), st2, blk, [env[x] for x in caps])
                st = st2
            elif k in ('loop', 'loopw'):
                s_ = self._loopw_alias[id(s)] if k == 'loopw' else s
                n = self.nat(b, self.ex(b, s_[1], env, st))
                blk, caps = self.blocks[id(s_[3])]
                st = b.fold(n, st, blk, [env[x] for x in caps])
            elif k == 'call':
                f = self.funcs[s[1]]
                args = [self.nat(b, self.ex(b, a, env, st)) for a in s[2]]
                st = b.apply(f, [st] + args)
            else:
                raise KeyError(k)
        return st

    def function(self, name):
        if name in self.funcs:
            return self.funcs[name]
        params, body = self.p.funcs[name]
        for callee in calls(body):
            self.function(callee)
        self.sub_blocks(body, set(params), name)
        b = self.g.block(name, [T] + [N] * len(params), T)
        env = {x: b.arg(1 + i) for i, x in enumerate(params)}
        b.returns(self.stmts(b, body, env, b.arg(0)))
        self.funcs[name] = b
        return b

    _loopw_alias = {}

    def load_roms(self):
        """A block that writes every constant table into the store."""
        b0 = self.g.block('romByte', [N, T, S, N], T)
        i, st, sq, base = (b0.arg(k) for k in range(4))
        b0.returns(b0.swrite(st, b0.add(base, i), b0.sidx(sq, i)))
        b = self.g.block('loadRoms', [T], T)
        st = b.arg(0)
        for name, key in self.romkey.items():
            w, vals = self.p.roms[name]
            data = b''.join(v.to_bytes(w, 'big') for v in vals)
            sq = b.seq(data)
            st = b.fold(b.slen(sq), st, b0, [sq, b._n(key)])
        b.returns(st)
        return b

    def input_loader(self):
        b = self.g.block('inputByte', [N, T, S], T)
        i, st, sq = b.arg(0), b.arg(1), b.arg(2)
        b.returns(b.swrite(st, b.add(INPUT, i), b.n2b(b.sub(255, b.b2n(b.sidx(sq, i))))))
        return b


def calls(stmts):
    out = []
    for s in stmts:
        k = s[0]
        if k == 'call':
            out.append(s[1])
        elif k == 'if':
            out += calls(s[2]) + calls(s[3])
        elif k == 'loop':
            out += calls(s[3])
        elif k == 'loopw':
            out += calls(s[4])
    return out
