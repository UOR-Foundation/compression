# State layout shared by the codec generators (a writing aid; the .lex.tex files are the source).
# A state is a right-nested pair of byte tables ending in a record of scalars:
#   (t0, (t1, (... (tk, Scalars))))
# so that rust-std moves each table out of the state when the state is consumed,
# and the in-place `prim update` never finds a second owner.
import re

class Layout:
    def __init__(self, name, tables, scalars, widths):
        self.name, self.tables, self.scalars, self.widths = name, tables, scalars, widths
        self.srec = name + 'S'
        self.counter = 0

    def real_type(self, i):
        # type of the state suffix starting at table i
        t = self.srec
        for _ in reversed(self.tables[i:]):
            t = f"(Bytes * {t})"
        return t

    def rest_type(self, i):
        # a placeholder token, replaced by finish() after the multiplication rewrite
        return self.name if i == 0 else (self.srec if i == len(self.tables) else f"{self.name}Rest{i}")

    def full_type(self):
        return self.real_type(0)

    def _suffix(self, v, i):
        e = v
        for _ in range(i):
            e = f"(second {e})"
        return e

    def sc(self, v):
        return self._suffix(v, len(self.tables))

    def P(self, v, f):
        if f in self.tables:
            return f"(first {self._suffix(v, self.tables.index(f))})"
        assert f in [s.split(':')[0] for s in self.scalars], f
        return f"(project {self.sc(v)} {f})"

    def upd(self, v, **kw):
        self.counter += 1
        n = self.counter
        tnames = {t: f"{t}{n}" for t in self.tables}
        scn = f"sc{n}"
        def subst(expr):
            for t in self.tables:
                expr = expr.replace(self.P(v, t), tnames[t])
            for s in self.scalars:
                f = s.split(':')[0]
                expr = expr.replace(self.P(v, f), f"(project {scn} {f})")
            if re.search(r'(?<![\w.])%s(?![\w])' % re.escape(v), expr):
                raise ValueError(f"upd({v}): a value still reads the state: {expr}")
            return expr
        kw = {k: subst(e) for k, e in kw.items()}
        opens = []
        rest = v
        for i, t in enumerate(self.tables):
            opens.append(f"(let {tnames[t]} : Bytes := (first {rest}); ")
            nxt = f"r{n}_{i}"
            opens.append(f"(let {nxt} : {self.rest_type(i + 1)} := (second {rest}); ")
            rest = nxt
        opens.append(f"(let {scn} : {self.srec} := {rest}; ")
        news = []
        for k, e in kw.items():
            if k in self.tables:
                news.append(f"(let n{tnames[k]} : Bytes := {e}; ")
        fields = []
        for s in self.scalars:
            f = s.split(':')[0]
            fields.append(f"{f} := {kw[f] if f in kw else f'(project {scn} {f})'}")
        result = f"(record {self.srec} {{" + ", ".join(fields) + "})"
        for t in reversed(self.tables):
            val = f"n{tnames[t]}" if t in kw else tnames[t]
            result = f"({val}, {result})"
        return "".join(opens) + "".join(news) + result + ")" * (len(opens) + len(news))

    def make(self, tables, scalars):
        result = "(record %s {" % self.srec + ", ".join(f"{s.split(':')[0]} := {scalars[s.split(':')[0]]}" for s in self.scalars) + "})"
        for t in reversed(self.tables):
            result = f"({tables[t]}, {result})"
        return result

    def structure(self):
        return ("structure{\n  name: %s,\n  type_parameters: [],\n  parameters: [],\n  fields: [\n" % self.srec
                + ",\n".join("    {name: %s, type: %s}" % tuple(f.split(':', 1)) for f in self.scalars) + "\n  ]\n}\n")

    def finish(self, text):
        # the longest names first: Rest1 is a prefix of Rest10
        for i in reversed(range(1, len(self.tables))):
            text = text.replace(f"{self.name}Rest{i}", self.real_type(i))
        return re.sub(r'(?<![\w.:])%s(?![\w])' % self.name, self.full_type(), text)


import json, os
_AXF = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'axioms.json')
_AX = json.load(open(_AXF)) if os.path.exists(_AXF) else {}

def definer(defs, module=None):
    def D(name, params, result, body, rec=None):
        ax = _AX.get(f"{module}.{name}", ['Quot.sound', 'propext'])
        AX = "  executable: true,\n  axioms: [%s]" % ", ".join(ax)
        s = "definition{\n  name: %s,\n  parameters: [%s],\n  result: %s,\n  body: %s,\n%s" % (name, ", ".join(params), result, body, AX)
        if rec:
            s += ",\n  recursive_argument: %s" % rec
        defs.append(s + "\n}\n")
    return D
