# Compiles an ir.Prog to LexLean source (a writing aid; the .lex.tex is the source).
# The state is a right-nested pair of byte tables ending in a record of the scalars (state.Layout),
# so rust-std moves each table out of the state when it is updated and `prim update` stays in place.
# The small tables share one byte table `mem` at their graph key bases; the large ones (and the
# encoder's output) are tables of their own, sized from the input length.
import sys
import os
import re
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from state import Layout, definer
import ir

TB = 'Uorc.Codec.Table'


def ident(x):
    return 'v' + re.sub(r'[^A-Za-z0-9]', 'X', x)


class Lex:
    def __init__(self, prog, module, entry_names, big_tables, input_field='input', extra_scalars=()):
        self.p = prog
        self.module = module
        self.big = list(big_tables)
        self.tables = ['glob', 'mem'] + self.big
        # The IR's globals live in a small table `glob` at the graph's key offsets (as in irgraph), so a
        # scalar update is one table write rather than a rebuilt record: the record holds only the input.
        scal = [f'{input_field}:Bytes'] + list(extra_scalars)
        self.gkey, k = {}, 0
        for g, w in prog.globals.items():
            self.gkey[g] = k; k += w
        assert k <= min(prog.tables.values()), 'globals overlap the first table'
        self.L = Layout(module[:4] + 'St', self.tables, scal, {})
        self.St = self.L.name
        self.input = input_field
        self.defs = []
        self.D = definer(self.defs, module)
        self.n = 0
        self.done = set()
        self.helpers()
        emitted = set()
        def post(name):
            if name in emitted:
                return
            emitted.add(name)
            import irgraph
            for c in irgraph.calls(self.p.funcs[name][1]):
                post(c)
            self.function(name)
        for e in entry_names:
            post(e)

    def writer(self, t):
        return 'put' + t[0].upper() + t[1:]

    def gread(self, s, g):
        return f"(getAt {self.p.globals[g]} {self.L.P(s, 'glob')} {self.gkey[g]})"

    def fresh(self, base='s'):
        self.n += 1
        return f'{base}{self.n}'

    def reach(self, name, acc):
        if name in acc:
            return
        acc.add(name)
        import irgraph
        for c in irgraph.calls(self.p.funcs[name][1]):
            self.reach(c, acc)

    def helpers(self):
        D = self.D
        D('pow2', ['k : Nat'], 'Nat', "(match k\n    | Nat.zero => 1\n    | Nat.succ m => (prim multiply 2 (pow2 m) : Nat))", 'k')
        D('getAt', ['w : Nat', 't : Bytes', 'at : Nat'], 'Nat', f"({TB}::getLoop w t at 0)")
        D('putAt', ['w : Nat', 't : Bytes', 'at : Nat', 'v : Nat'], 'Bytes', f"({TB}::setLoop w t (prim subtract (at + w) 1 : Nat) v)")
        D('sqt', [], 'Bytes', '0x' + b''.join(v.to_bytes(2, 'big') for v in ir.SQT).hex())
        D('squash', ['d : Nat'], 'Nat',
          "(let i : Nat := (prim quotient d 128 0 : Nat); (let w : Nat := (prim remainder d 128 0 : Nat); "
          "(prim quotient ((prim multiply (getAt 2 (sqt) (prim multiply 2 i : Nat)) (prim subtract 128 w : Nat) : Nat) + "
          "((prim multiply (getAt 2 (sqt) (prim multiply 2 (i + 1) : Nat)) w : Nat) + 64)) 128 0 : Nat)))")
        body = 'a'
        steps = []
        for j, (k, s) in enumerate(zip(ir.HK, ir.HS)):
            steps.append(f"(let m{j} : Nat := (prim remainder (prim multiply {k} {body} : Nat) 4294967296 0 : Nat); "
                         f"(let a{j} : Nat := (prim remainder (m{j} + (prim quotient m{j} {1 << s} 0 : Nat)) 4294967296 0 : Nat); ")
            body = f'a{j}'
        D('hash', ['a : Nat'], 'Nat', ''.join(steps) + body + ')' * (2 * len(ir.HK)))
        # one writer per table: moves the table out of the state, writes it in place, rebuilds the state
        for t in self.tables:
            D(self.writer(t), [f's : {self.St}', 'w : Nat', 'k : Nat', 'v : Nat'], self.St,
              self.L.upd('s', **{t: f"(putAt w {self.L.P('s', t)} k v)"}))

    # ---- expressions: (text, type) ----
    def nat(self, r):
        t, ty = r
        return t if ty == 'Nat' else f'(if {t} then 1 else 0)'

    def boo(self, r):
        t, ty = r
        return t if ty == 'Bool' else f'(0 <? {t})'

    def addr(self, table, w, idx):
        base = self.p.tables[table]
        if table in self.big:
            return idx if w == 1 else f'(prim multiply {w} {idx} : Nat)'
        if w == 1:
            return f'({base} + {idx})'
        return f'({base} + (prim multiply {w} {idx} : Nat))'

    def ex(self, e, env, s):
        if isinstance(e, bool):
            return ('true' if e else 'false', 'Bool')
        if isinstance(e, int):
            return (str(e), 'Nat')
        k = e[0]
        if k == 'v':
            return env[e[1]]
        if k == 'g':
            return (self.gread(s, e[1]), 'Nat')
        if k == 'ld':
            t = e[1] if e[1] in self.big else 'mem'
            return (f"(getAt {e[2]} {self.L.P(s, t)} {self.addr(e[1], e[2], self.nat(self.ex(e[3], env, s)))})", 'Nat')
        if k == 'rom':
            assert e[1] == 'POW2'
            return (f"(pow2 {self.nat(self.ex(e[3], env, s))})", 'Nat')
        if k == 'in':
            i = self.nat(self.ex(e[1], env, s))
            inp = self.L.P(s, self.input)
            return (f"(if ({i} <? (prim length {inp} : Nat)) then ({TB}::readByte {inp} {i}) else 255)", 'Nat')
        if k == 'hash':
            return (f"(hash {self.nat(self.ex(e[1], env, s))})", 'Nat')
        if k == 'squash':
            return (f"(squash {self.nat(self.ex(e[1], env, s))})", 'Nat')
        if k == 'not':
            return (f"(not {self.boo(self.ex(e[1], env, s))})", 'Bool')
        if k == 'sel':
            c = self.boo(self.ex(e[1], env, s))
            a, b = self.ex(e[2], env, s), self.ex(e[3], env, s)
            if a[1] == b[1] == 'Bool':
                return (f"(if {c} then {a[0]} else {b[0]})", 'Bool')
            return (f"(if {c} then {self.nat(a)} else {self.nat(b)})", 'Nat')
        if k in ('and', 'or'):
            a, b = self.boo(self.ex(e[1], env, s)), self.boo(self.ex(e[2], env, s))
            return (f"({a} {'&&' if k == 'and' else '||'} {b})", 'Bool')
        a, b = self.nat(self.ex(e[1], env, s)), self.nat(self.ex(e[2], env, s))
        if k == 'add':
            # Nat.add recurses on its second argument too: a literal goes first
            if isinstance(e[2], int) and not isinstance(e[1], int):
                a, b = b, a
            return (f"({a} + {b})", 'Nat')
        if k == 'sub': return (f"(prim subtract {a} {b} : Nat)", 'Nat')
        if k == 'mul':
            # Nat.mul recurses on its second argument: a literal goes first (certificate kernel)
            if isinstance(e[2], int) and not isinstance(e[1], int):
                a, b = b, a
            return (f"(prim multiply {a} {b} : Nat)", 'Nat')
        if k == 'div': return (f"(prim quotient {a} {b} 0 : Nat)", 'Nat')
        if k == 'mod': return (f"(prim remainder {a} {b} 0 : Nat)", 'Nat')
        if k == 'lt': return (f"({a} <? {b})", 'Bool')
        if k == 'le': return (f"({a} <=? {b})", 'Bool')
        if k == 'eq': return (f"({a} =? {b})", 'Bool')
        if k == 'ne': return (f"(not ({a} =? {b}))", 'Bool')
        raise KeyError(k)

    # ---- statements: an expression that evaluates to the final state ----
    # A long statement list is cut into functions of at most CHUNK statements, each ending in a call
    # of the next with the locals it still reads: Lean's compiler (which the rust-std preservation
    # certificate runs on every exported function) is superlinear in the size of one function.
    CHUNK = 6

    def block(self, ss, env, s, name):
        if len(ss) <= self.CHUNK:
            return self.stmts(ss, env, s, name)
        return self.stmts(list(ss[:self.CHUNK]) + [('jump', list(ss[self.CHUNK:]))], env, s, name)

    def stmts(self, ss, env, s, name):
        if not ss:
            return s
        st, rest = ss[0], ss[1:]
        k = st[0]
        env = dict(env)
        if k == 'jump':
            tail = st[1]
            live = [x for x in ir.free_vars(tail) if x in env]
            fn = f"{name}Part{self.fresh('')}"
            inner = {x: (ident(x), env[x][1]) for x in live}
            params = [f's : {self.St}'] + [f'{ident(x)} : {env[x][1]}' for x in live]
            self.D(fn, params, self.St, self.block(tail, inner, 's', fn))
            return f"({fn} {s}{''.join(' ' + env[x][0] for x in live)})"
        if k == 'let':
            t, ty = self.ex(st[2], env, s)
            v = ident(st[1]) + str(self.fresh(''))
            env[st[1]] = (v, ty)
            return f"(let {v} : {ty} := {t}; {self.stmts(rest, env, s, name)})"
        if k == 'set':
            t = self.nat(self.ex(st[2], env, s))
            v = self.fresh('t'); s2 = self.fresh('s')
            upd = f"({self.writer('glob')} {s} {self.p.globals[st[1]]} {self.gkey[st[1]]} {v})"
            return f"(let {v} : Nat := {t}; (let {s2} : {self.St} := {upd}; {self.stmts(rest, env, s2, name)}))"
        if k == 'store':
            tab = st[1] if st[1] in self.big else 'mem'
            key = self.addr(st[1], st[2], self.nat(self.ex(st[3], env, s)))
            val = self.nat(self.ex(st[4], env, s))
            kv, vv, s2 = self.fresh('k'), self.fresh('t'), self.fresh('s')
            upd = f"({self.writer(tab)} {s} {st[2]} {kv} {vv})"
            return f"(let {kv} : Nat := {key}; (let {vv} : Nat := {val}; (let {s2} : {self.St} := {upd}; {self.stmts(rest, env, s2, name)})))"
        if k == 'if':
            c = self.boo(self.ex(st[1], env, s))
            a = self.block(st[2], env, s, name + 'T')
            b = self.block(st[3], env, s, name + 'F')
            s2 = self.fresh('s')
            return f"(let {s2} : {self.St} := (if {c} then {a} else {b}); {self.stmts(rest, env, s2, name)})"
        if k in ('loop', 'loopw'):
            body = st[3] if k == 'loop' else st[4]
            ivar = st[2] if k == 'loop' else st[3]
            caps = [x for x in ir.free_vars(body, {ivar}) if True]
            cond = None
            if k == 'loopw':
                caps = list(dict.fromkeys(caps + ir.free_vars([('let', '_', st[2])])))
            fn = self.loop_fn(name, body, ivar, caps, env, st[2] if k == 'loopw' else None)
            count = self.nat(self.ex(st[1], env, s))
            if isinstance(st[1], int):
                # a literal trip count would let the certificate kernel unroll the loop over a state it
                # cannot see; naming the state (times 0) keeps the count opaque to it
                count = f"({count} + (prim multiply 0 (getAt 1 {self.L.P(s, 'glob')} 0) : Nat))"
            s2 = self.fresh('s')
            args = ' '.join(env[x][0] for x in caps)
            return f"(let {s2} : {self.St} := ({fn} {count} 0 {s} {args}); {self.stmts(rest, env, s2, name)})"
        if k == 'call':
            args = ' '.join(self.nat(self.ex(a, env, s)) for a in st[2])
            s2 = self.fresh('s')
            return f"(let {s2} : {self.St} := ({st[1]} {s} {args}); {self.stmts(rest, env, s2, name)})"
        raise KeyError(k)

    def loop_fn(self, name, body, ivar, caps, env, cond):
        fn = f"{name}Loop{self.fresh('')}"
        iv = ident(ivar)
        inner = {ivar: (iv, 'Nat')}
        params = ['fuel : Nat', f'{iv} : Nat', f's : {self.St}']
        for x in caps:
            v, ty = env[x]
            pv = ident(x)
            inner[x] = (pv, ty)
            params.append(f'{pv} : {ty}')
        capargs = ' '.join(inner[x][0] for x in caps)
        b = self.block(body, inner, 's', fn)
        step = f"(let next : {self.St} := {b}; ({fn} m ({iv} + 1) next {capargs}))"
        if cond is not None:
            step = f"(if {self.boo(self.ex(cond, inner, 's'))} then {step} else s)"
        self.D(fn, params, self.St, f"(match fuel\n    | Nat.zero => s\n    | Nat.succ m => {step})", 'fuel')
        return fn

    def function(self, name):
        params, body = self.p.funcs[name]
        env = {x: (ident(x), 'Nat') for x in params}
        text = self.block(body, env, 's', name)
        self.D(name, [f's : {self.St}'] + [f'{ident(x)} : Nat' for x in params], self.St, text)

    def source(self, imports, extra_defs=()):
        defs = list(self.defs) + list(extra_defs)
        text = ("\\begin{lexlean}{Uorc.Codec.%s}\n" % self.module + ''.join(f"\\importmodule{{{m}}}\n" for m in imports)
                + "\\title{Parameters}\n\\begin{semanticmodule}\n" + self.L.structure() + "\n" + "\n".join(defs)
                + "\\end{semanticmodule}\n\\end{lexlean}\n")
        return self.L.finish(text)
