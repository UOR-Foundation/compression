# The domains of a coder's quantities, and the pass that derives the figures its IR states from them.
#
# A coder's numbers are not free: a probability is a 16-bit slot, a logistic value lives in the
# stretch domain, a weight in its offset form, a stored cell spans its width in bytes. A model declares
# those domains once, each by its extent (a format identity, stated as a named numeral of the wire
# module the coder lowers to), and which tables and globals hold each one. This pass infers the domain
# of every operand of the IR to a fixpoint (a table or global takes the domains of what is stored in
# it), and then states each literal that is a characteristic quantity of the domain of the operand it
# acts on (the extent, its top, its half, an origin, a rescale between two domains in scope) as that
# quantity, derived from the extent. A literal the pass cannot place keeps its numeral, and is reported
# with the domains in scope, so the figures it leaves are the next declarations to make.
import collections

import ir
from ir import K

BYTE = K('byteBase', 256)
RADIX = K('radix', 2)            # the coder is binary: one decision per coding step


class Domain:
    """A declared domain: its name and extent (a named quantity); an origin if the domain is in offset
    form (the value that stands for zero, by default its half); a unit if it is fixed point (the value
    that stands for one)."""

    def __init__(self, name, extent, origin=None, unit=None, offset=False):
        self.name, self.extent, self.unit = name, extent, unit
        self.half = K(name + 'Half', ('div', extent, RADIX))
        self.origin = self.half if offset and origin is None else origin

    def quantities(self):
        n, r = self.name, self.extent
        half = self.half
        out = [r, K(n + 'Last', ('sub', r, 1)), half, K(n + 'HalfLast', ('sub', half, 1))]
        if self.unit is not None:
            out += [self.unit, K(n + 'UnitLast', ('sub', self.unit, 1))]
        if self.origin is not None and self.origin is not half:
            o = self.origin       # an offset domain's zero; its top, the most it holds above zero
            out += [o, K(n + 'OriginLast', ('sub', o, 1)), K(n + 'Top', ('add', o, o))]
        return out


def origins(n, *ds):
    """The constant term of a sum of n products of offset-form values, one from each domain of ds:
    n times the product of their origins (what (x + ox)(w + ow) = xw + x ow + w ox + ox ow leaves)."""
    e = n
    for d in ds:
        e = K('%s%s' % (e.name if isinstance(e, K) else 'n%d' % int(e), d.name.capitalize() + 'Origin'), ('mul', e, d.origin))
    return e


def cell(w):
    """The domain of a stored cell of w bytes: w byte lanes."""
    e = BYTE
    for _ in range(int(w) - 1):
        e = K('cell%dBase' % (_ + 2), ('mul', e, BYTE))
    d = Domain('cell%d' % int(w), e)
    d.lanes = w
    return d


def lanes(d):
    """A cell's byte lanes: the radix of each byte place."""
    out, e = [], BYTE
    for j in range(1, getattr(d, 'lanes', 0)):
        out.append(e)
        e = K('cell%dBase' % (j + 1), ('mul', e, BYTE))
    return out


def rescale(a, b):
    """A rescale from domain b to domain a (a the wider), when the extents divide."""
    if int(a.extent) > int(b.extent) and int(a.extent) % int(b.extent) == 0:
        return [K('%sPer%s' % (a.name, b.name[0].upper() + b.name[1:]), ('div', a.extent, b.extent))]
    return []


class Pass:
    """Infer every operand's domains over prog (tables and globals seeded by tables, globals) and
    derive the literals. Domains are ordered: the first declared wins a value two of them share."""

    def __init__(self, prog, domains, tables, globals_):
        self.p, self.dom = prog, {d.name: d for d in domains}
        self.order = [d.name for d in domains]
        self.seed_t = {t: set(ds) for t, ds in tables.items()}
        self.seed_g = {g: set(ds) for g, ds in globals_.items()}
        self.cells = {}
        self.residue = collections.Counter()
        self.derived = collections.Counter()

    def cell(self, w):
        w = int(w)
        if w not in self.cells:
            self.cells[w] = cell(w)
        return self.cells[w]

    def domains(self, names):
        ds = [self.dom[n] for n in self.order if n in names]
        ds += [self.cells[w] for w in sorted(self.cells) if ('cell', w) in names]
        return ds

    def place(self, v, names, want=frozenset()):
        """The quantity of value v among the domains names (of the operands it acts with) and want (of
        where the value goes), or a rescale between two of them; None if there is none."""
        ds = self.domains(names | want)
        for d in ds:
            for q in d.quantities() + lanes(d):
                if int(q) == v:
                    return q
        for a in ds:
            for b in ds:
                for q in rescale(a, b):
                    if int(q) == v:
                        return q
        return None

    def lit(self, e, names, want=frozenset()):
        if isinstance(e, K) or int(e) <= 1:
            return e
        q = self.place(int(e), set(names), set(want))
        if q is None:
            self.residue[(int(e), tuple(sorted(map(str, names))))] += 1
            return e
        self.derived[q.name] += 1
        return q

    # expressions: (domains, rewritten expression)
    def ex(self, e, env, want=frozenset()):
        """(domains of e, e with its literals derived); want is the domains of where e's value goes
        (the cell or table it is stored in, the squash it feeds), which a literal factor may rescale to."""
        if isinstance(e, bool) or isinstance(e, int):
            return set(), e
        k = e[0]
        if k == 'v':
            return env.get(e[1], set()), e
        if k == 'g':
            self.cell(self.p.globals[e[1]])
            return {('cell', int(self.p.globals[e[1]]))} | self.gd.get(e[1], set()), e
        if k == 'ld':
            _, i = self.ex(e[3], env)
            self.cell(e[2])
            return {('cell', int(e[2]))} | self.td.get(e[1], set()), (k, e[1], e[2], i)
        if k == 'rom':
            _, i = self.ex(e[3], env)
            w = self.p.roms[e[1]][0]
            self.cell(w)
            return {('cell', int(w))}, (k, e[1], e[2], i)
        if k == 'in':
            _, i = self.ex(e[1], env)
            return {('cell', 1)}, (k, i)
        if k == 'hash':
            _, i = self.ex(e[1], env)
            self.cell(4)
            return {('cell', 4)}, (k, i)
        if k == 'squash':
            return {'stretch'}, (k, self.operands([e[1]], {'stretch'}, env, {'stretch'})[1][0])
        if k == 'not':
            _, a = self.ex(e[1], env)
            return set(), (k, a)
        if k == 'sel':
            _, c = self.ex(e[1], env)
            both, (a, b) = self.operands(e[2:], set(), env, want)
            return both, (k, c, a, b)
        if k in ('mod', 'div') and type(e[2]) is int and e[2] == int(RADIX):
            # a quotient or remainder by 2 takes a word's binary digits apart: the radix
            self.derived[RADIX.name] += 1
            e = (k, e[1], RADIX)
        arith = k in ('add', 'sub', 'mod', 'mul', 'div')
        both, (a, b) = self.operands(e[1:], set(), env, want if arith else frozenset())
        return (both if arith else set()), (k, a, b)

    def operands(self, es, extra, env, want):
        """Operands that share a domain: each literal among them is placed in the domains of the rest."""
        got = [None if (isinstance(x, int) and not isinstance(x, bool)) else self.ex(x, env, want) for x in es]
        both = set(extra)
        for g in got:
            if g is not None:
                both |= g[0]
        out = [self.lit(x, both, want) if g is None else g[1] for x, g in zip(es, got)]
        return both, out

    def st(self, ss, env):
        env, out = dict(env), []
        for s in ss:
            k = s[0]
            if k == 'let':
                d, v = self.ex(s[2], env) if not isinstance(s[2], int) else (set(), s[2])
                env[s[1]] = d
                out.append((k, s[1], v))
            elif k == 'set':
                gd = {('cell', int(self.p.globals[s[1]]))} | self.gd.get(s[1], set())
                self.cell(self.p.globals[s[1]])
                if isinstance(s[2], int):
                    out.append((k, s[1], self.lit(s[2], gd)))
                else:
                    d, v = self.ex(s[2], env, gd)
                    self.ng.setdefault(s[1], set()).update(d - {('cell', int(self.p.globals[s[1]]))})
                    out.append((k, s[1], v))
            elif k == 'store':
                c = ('cell', int(s[2]))
                self.cell(s[2])
                _, i = self.ex(s[3], env) if not isinstance(s[3], int) else (set(), s[3])
                if isinstance(s[4], int):
                    v = self.lit(s[4], {c} | self.td.get(s[1], set()))
                else:
                    d, v = self.ex(s[4], env, {c} | self.td.get(s[1], set()))
                    self.nt.setdefault(s[1], set()).update(d - {c})
                out.append((k, s[1], s[2], i, v) + tuple(s[5:]))
            elif k == 'if':
                out.append((k, self.ex(s[1], env)[1], self.st(s[2], env), self.st(s[3], env)))
            elif k == 'loop':
                out.append((k, self.ex(s[1], env)[1] if not isinstance(s[1], int) else s[1], s[2], self.st(s[3], env)))
            elif k == 'loopw':
                out.append((k, self.ex(s[1], env)[1] if not isinstance(s[1], int) else s[1],
                            self.ex(s[2], env)[1], s[3], self.st(s[4], env)))
            elif k == 'call':
                out.append((k, s[1], [self.ex(a, env)[1] if not isinstance(a, int) else a for a in s[2]]) + tuple(s[3:]))
            elif k == 'gcopy':
                out.append((k, s[1], self.ex(s[2], env)[1] if not isinstance(s[2], int) else s[2]) + tuple(s[3:]))
            else:
                out.append(s)
        return out

    def run(self):
        self.td, self.gd = dict(self.seed_t), dict(self.seed_g)
        for _ in range(16):
            self.nt, self.ng = {}, {}
            self.residue.clear()
            self.derived.clear()
            funcs = {f: (pb[0], self.st(pb[1], {})) for f, pb in self.p.funcs.items()}
            td = {t: self.nt.get(t, set()) | self.seed_t.get(t, set()) for t in set(self.nt) | set(self.seed_t)}
            gd = {g: self.ng.get(g, set()) | self.seed_g.get(g, set()) for g in set(self.ng) | set(self.seed_g)}
            if td == self.td and gd == self.gd:
                return funcs
            self.td, self.gd = td, gd
        raise RuntimeError('domains: no fixpoint')


def derive(prog, domains, tables, globals_):
    """Rewrite prog's functions in place; return the pass (its derived and residue counts)."""
    p = Pass(prog, domains, tables, globals_)
    prog.funcs = p.run()
    return p


class Widths:
    """The globals' widths, typed by their declared domains: a global's width is the least rung of the
    machine's word ladder (the widest cell, wordBytes, halved by the radix down to a byte) whose cell holds
    its domain's extent. A global whose stated width is that rung takes the rung, so its width is derived
    from what it holds; the IR machine then holds every value set to the domain (Machine.set). A global
    whose width is not the least rung of its domain, or that has no declared domain, keeps its numeral and
    is reported."""

    def __init__(self, word):
        rungs, w, n = [], word, 0
        while int(w) > 1:
            rungs.append(w)
            n += 1
            w = K(('wordHalfBytes', 'wordQuarterBytes', 'wordEighthBytes')[n - 1], ('div', w, RADIX)) if int(w) > 2 else 1
        self.rungs = sorted(rungs, key=int)
        self.derived, self.residue = {}, {}

    def least(self, extent):
        for r in self.rungs:
            if int(BYTE) ** int(r) >= int(extent):
                return r
        return None

    def type(self, prog, declared):
        prog.bounds = getattr(prog, 'bounds', {})
        for g, w in list(prog.globals.items()):
            if int(w) <= 1 or isinstance(w, K):
                continue
            d = declared.get(g)
            r = None if d is None else self.least(d.extent)
            if r is not None and int(r) == int(w):
                prog.globals[g] = r
                prog.bounds[g] = d.extent
                self.derived[g] = (r.name, d.name)
            else:
                self.residue[g] = (int(w), None if d is None else d.name, None if r is None else int(r))
        return self


def report(p, top=40):
    """The pass's outcome: the quantities it derived (uses each) and the literals it left, by value,
    with the domains in scope at each (the declarations the residue asks for)."""
    lines = ['derived %d literal uses as %d quantities' % (sum(p.derived.values()), len(p.derived))]
    by = collections.defaultdict(collections.Counter)
    for (v, names), n in p.residue.items():
        by[v][names] += n
    lines.append('left %d literal uses of %d values' % (sum(p.residue.values()), len(by)))
    for v, cs in sorted(by.items(), key=lambda kv: -sum(kv[1].values()))[:top]:
        scope = ', '.join('%s x%d' % ('/'.join(n) or '(none)', c) for n, c in cs.most_common(3))
        lines.append('  %12d  %5d  %s' % (v, sum(cs.values()), scope))
    return '\n'.join(lines)


if __name__ == '__main__':
    import importlib
    import sys
    for name in sys.argv[1:] or ['ncm_model']:
        print(name + ':\n' + report(importlib.import_module(name).DERIVED))
