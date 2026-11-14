# A small imperative IR for the uorc codec models (a writing aid; the generated .lex.tex and the
# kernel graph are the sources). One model description compiles to:
#   - LexLean source (functions over a state of byte tables plus a record of scalars),
#   - a UORC program graph (everything in one store), and
#   - a Python interpreter, used to check both against the reference C/Rust coder.
#
# Values are naturals. Every operation must stay total in all three targets: `sub` requires a >= b
# (the graph fails otherwise), and the arms of `sel` are both evaluated by the graph.
#
# Statements:  ('let', x, e) ('set', g, e) ('store', table, width, key, value)
#              ('if', cond, then, else) ('loop', count, ivar, body) ('call', fn, [args])
#              ('loopw', count, cond, ivar, body): at most count rounds, stopping before the first round
#              whose cond (over globals only) is false
# Expressions: int, ('v', x) local/param, ('g', g) global, ('ld', table, width, key),
#              ('rom', name, width, index) constant table, ('in', index) read-only input byte
#              (255 past its end), (op, a, b) for op in add sub mul div mod lt le eq ne and or,
#              ('not', p), ('sel', c, a, b), ('hash', e) the 32-bit mixer H, ('squash', d) the logistic
#              of an offset stretch value d in [1, 4095].

BIN = {'add', 'sub', 'mul', 'div', 'mod', 'lt', 'le', 'eq', 'ne', 'and', 'or'}
PRED = {'lt', 'le', 'eq', 'ne', 'and', 'or', 'not'}


class Prog:
    def __init__(self):
        self.globals = {}      # name -> width in bytes (graph store encoding)
        self.tables = {}       # name -> graph store base key
        self.roms = {}         # name -> (width, [values])
        self.funcs = {}        # name -> (params, body)
        self.order = []

    def glob(self, name, width):
        self.globals[name] = width

    def table(self, name, base):
        self.tables[name] = base

    def rom(self, name, width, values):
        self.roms[name] = (width, list(values))

    def func(self, name, params, body):
        self.funcs[name] = (params, body)
        self.order.append(name)


# ---- expression helpers ----
def V(x): return ('v', x)
def G(x): return ('g', x)
def add(a, b): return ('add', a, b)
def sub(a, b): return ('sub', a, b)
def mul(a, b): return ('mul', a, b)
def div(a, b): return ('div', a, b)
def mod(a, b): return ('mod', a, b)
def lt(a, b): return ('lt', a, b)
def le(a, b): return ('le', a, b)
def eq(a, b): return ('eq', a, b)
def ne(a, b): return ('ne', a, b)
def AND(a, b): return ('and', a, b)
def OR(a, b): return ('or', a, b)
def NOT(a): return ('not', a)
def sel(c, a, b): return ('sel', c, a, b)
def ld(t, w, k): return ('ld', t, w, k)
def rom(n, w, i): return ('rom', n, w, i)
def IN(i): return ('in', i)
def u32(e): return mod(e, 1 << 32)
def shr(e, k): return div(e, 1 << k)
def shl(e, k): return mul(1 << k, e)
def lo(e, k): return mod(e, 1 << k)
def mn(a, b): return sel(lt(a, b), a, b)
def mx(a, b): return sel(lt(a, b), b, a)
def monus(a, b): return sub(a, mn(b, a))


SQT = [1, 2, 3, 6, 10, 16, 27, 45, 73, 120, 194, 310, 488, 747, 1101, 1546, 2047,
       2549, 2994, 3348, 3607, 3785, 3901, 3975, 4022, 4050, 4068, 4079, 4085, 4089, 4092, 4093, 4094]
HK = (0x9E3779B1, 0x2C1B3C6D, 0x297A2D39)
HS = (15, 12, 15)
M32 = (1 << 32) - 1


def H(a):
    for k, s in zip(HK, HS):
        a = (a * k) & M32
        a = (a + (a >> s)) & M32
    return a


def hash_(e): return ('hash', e)
def squash(d): return ('squash', d)


def is_pred(e):
    return isinstance(e, tuple) and e[0] in PRED


def free_vars(stmts, bound=()):
    """Locals read by stmts that they do not bind themselves (in first-use order)."""
    out, seen = [], set()
    bound = set(bound)

    def ex(e, b):
        if isinstance(e, int):
            return
        k = e[0]
        if k == 'v':
            if e[1] not in b and e[1] not in seen:
                seen.add(e[1]); out.append(e[1])
        elif k in ('g',):
            return
        elif k == 'ld':
            ex(e[3], b)
        elif k == 'rom':
            ex(e[3], b)
        elif k == 'in':
            ex(e[1], b)
        else:
            for a in e[1:]:
                ex(a, b)

    def st(ss, b):
        b = set(b)
        for s in ss:
            k = s[0]
            if k == 'let':
                ex(s[2], b); b.add(s[1])
            elif k == 'set':
                ex(s[2], b)
            elif k == 'store':
                ex(s[3], b); ex(s[4], b)
            elif k == 'if':
                ex(s[1], b); st(s[2], b); st(s[3], b)
            elif k == 'loop':
                ex(s[1], b); st(s[3], b | {s[2]})
            elif k == 'loopw':
                ex(s[1], b); ex(s[2], b); st(s[4], b | {s[3]})
            elif k == 'call':
                for a in s[2]:
                    ex(a, b)
            elif k == 'gcopy':
                ex(s[2], b)
    st(stmts, bound)
    return out


# ---- Python interpreter ----
class Machine:
    def __init__(self, prog, inp=b''):
        self.p = prog
        self.g = {k: 0 for k in prog.globals}
        self.t = {k: {} for k in prog.tables}
        self.inp = inp
        self.ops = 0

    def ev(self, e, env):
        if isinstance(e, int):
            return e
        k = e[0]
        if k == 'v':
            return env[e[1]]
        if k == 'g':
            return self.g[e[1]]
        if k == 'ld':
            t = self.t[e[1]]; w = e[2]; key = self.ev(e[3], env); v = 0
            for j in range(w):
                v = v * 256 + t.get(w * key + j, 0)
            return v
        if k == 'rom':
            return self.p.roms[e[1]][1][self.ev(e[3], env)]
        if k == 'in':
            i = self.ev(e[1], env)
            return self.inp[i] if i < len(self.inp) else 255
        if k == 'not':
            return not self.ev(e[1], env)
        if k == 'hash':
            return H(self.ev(e[1], env))
        if k == 'squash':
            d = self.ev(e[1], env); i, w = d // 128, d % 128
            return (SQT[i] * (128 - w) + SQT[i + 1] * w + 64) // 128
        if k == 'sel':
            c = self.ev(e[1], env); a = self.ev(e[2], env); b = self.ev(e[3], env)
            return a if c else b
        a = self.ev(e[1], env); b = self.ev(e[2], env)
        self.ops += 1
        if k == 'add': r = a + b
        elif k == 'sub':
            if b > a: raise ValueError('negative subtraction %r' % (e,))
            r = a - b
        elif k == 'mul': r = a * b
        elif k == 'div': r = a // b
        elif k == 'mod': r = a % b
        elif k == 'lt': return a < b
        elif k == 'le': return a <= b
        elif k == 'eq': return a == b
        elif k == 'ne': return a != b
        elif k == 'and': return bool(a and b)
        elif k == 'or': return bool(a or b)
        else: raise KeyError(k)
        if r >= 1 << 64: raise ValueError('overflow %r' % (e,))
        return r

    def run(self, stmts, env):
        for s in stmts:
            k = s[0]
            if k == 'let':
                v = self.ev(s[2], env)
                env[s[1]] = int(v) if isinstance(v, bool) else v
            elif k == 'set':
                v = int(self.ev(s[2], env))
                assert v < 1 << (8 * self.p.globals[s[1]]), (s[1], v)
                self.g[s[1]] = v
            elif k == 'store':
                key = self.ev(s[3], env); v = int(self.ev(s[4], env)); w = s[2]
                assert v < 1 << (8 * w), (s, v)
                t = self.t[s[1]]
                for j in range(w - 1, -1, -1):
                    t[w * key + j] = v % 256; v //= 256
            elif k == 'if':
                self.run(s[2] if self.ev(s[1], env) else s[3], dict(env))
            elif k == 'loop':
                n = self.ev(s[1], env)
                for i in range(n):
                    e2 = dict(env); e2[s[2]] = i
                    self.run(s[3], e2)
            elif k == 'loopw':
                n = self.ev(s[1], env)
                for i in range(n):
                    if not self.ev(s[2], env if False else self._genv(env)):
                        break
                    e2 = dict(env); e2[s[3]] = i
                    self.run(s[4], e2)
            elif k == 'call':
                params, body = self.p.funcs[s[1]]
                args = [int(self.ev(a, env)) for a in s[2]]
                self.run(body, dict(zip(params, args)))
            elif k == 'gcopy':
                # a lane's globals (the first prog.lane_globals) saved to, or loaded from, its copy at s[2]
                at = self.ev(s[2], env)
                lanes = self.__dict__.setdefault('lanes', {})
                names = list(self.p.globals)[:self.p.lane_globals]
                if s[1] == 'save':
                    lanes[at] = {g: self.g[g] for g in names}
                else:
                    saved = lanes.get(at, {})
                    for g in names:
                        self.g[g] = saved.get(g, 0)
            else:
                raise KeyError(k)

    def _genv(self, env):
        return env

    def call(self, name, *args):
        self.run([('call', name, list(args))], {})
