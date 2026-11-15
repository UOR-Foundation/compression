# The slots of the model's kernel graph blocks, and the pass that states their references from them.
#
# A kernel graph block names its values by slot: its parameters first, then one slot for each
# instruction, in order. An instruction refers to an earlier value by its distance back from the slot it
# defines (Uorc.Core.Graph::slotType: index = slots - distance). A distance is therefore not a free
# number: it is the difference of two slot indices, and a slot index is the one before it plus one.
#
# The pass reads each block of Search and Core whose instruction list is written out (directly, or
# through the parts a large list is split into), simulates its slots, and states the block as that
# layout: every slot index is bound once (the first is zero, each next is the one before it plus one),
# and every slot reference is the difference of the referring instruction's slot and the slot it names.
# A part of a split list takes the slot indices it refers to as parameters. Block references (the
# distance of a called block among those before it) are left as they are: they are the next layout.
#
#   python3 client/gen/srcslots.py            rewrite in place, report what is left
#   python3 client/gen/srcslots.py --check    exit 1 unless the sources are at the pass's fixpoint
import collections
import re
import sys

import srcdomains as S

INSTR = 'Uorc.Core.Graph::Instruction.'
JOIN = 'joinBoundedList0'

# Fields of each instruction that name a block (a distance among prior blocks), not a slot.
BLOCK_FIELDS = {'apply': {0}, 'tabulate': {1}, 'fold': {2}, 'scan': {2, 4}}
# Fields that are not references at all (constants).
CONSTANTS = {'naturalConstant', 'byteConstant', 'predicateConstant', 'sequenceConstant',
             'sequenceRopeConstant'}


def name(i):
    """The name of slot i: slotA, slotB, ... slotZ, slotAA, ..."""
    s, i = '', i + 1
    while i:
        i, r = divmod(i - 1, 26)
        s = chr(ord('A') + r) + s
    return 'slot' + s


class Ref:
    """A slot reference in an instruction: the leaf token, the slot it names (once simulated)."""

    def __init__(self, leaf):
        self.leaf, self.target = leaf, None


def instructions(node):
    """The instruction ctors of a cons-chain, in order, each as (opcode, [Ref or None per Nat leaf]),
    or None if the list is not written out."""
    out = []
    cur = node
    while True:
        if not isinstance(cur, S.Node):
            return None
        k = cur.kids
        if k and S.tok(k[0]) == 'nil':
            return out
        if not (k and S.tok(k[0]) == 'cons' and len(k) == 3 and isinstance(k[1], S.Node) and isinstance(k[2], S.Node)):
            return None
        ins = k[1].kids
        if not (ins and S.tok(ins[0]) == 'ctor' and (S.tok(ins[1]) or '').startswith(INSTR)):
            return None
        op = S.tok(ins[1])[len(INSTR):]
        refs = []
        if op not in CONSTANTS:
            for fi, f in enumerate(ins[2:]):
                block = fi in BLOCK_FIELDS.get(op, ())
                if isinstance(f, tuple):
                    if not f[0].isdigit():
                        return None            # a computed reference: the block is built, not written
                    refs.append((fi, block, [f]))
                else:
                    leaves = []
                    sub = f
                    while sub.kids and S.tok(sub.kids[0]) == 'cons':
                        x = sub.kids[1]
                        if not (isinstance(x, tuple) and x[0].isdigit()):
                            return None
                        leaves.append(x)
                        sub = sub.kids[2]
                    if not (sub.kids and S.tok(sub.kids[0]) == 'nil'):
                        return None
                    refs.append((fi, block, leaves))
        out.append((op, refs))
        cur = k[2]


def count_types(node):
    """The number of entries of a written-out (List ValueType)."""
    if not isinstance(node, S.Node):
        return None
    n, cur = 0, node
    while cur.kids and S.tok(cur.kids[0]) == 'cons':
        n += 1
        cur = cur.kids[2]
        if not isinstance(cur, S.Node):
            return None
    return n if cur.kids and S.tok(cur.kids[0]) == 'nil' else None


def field(rec, fname):
    """The value node of `fname := value` in a record's brace group."""
    k = rec.kids
    for i in range(len(k) - 2):
        if S.tok(k[i]) == fname and S.tok(k[i + 1]) == ':=':
            return k[i + 2]
    return None


class Defs:
    def __init__(self, text):
        self.text = text
        self.defs = {}
        for m in re.finditer(r'definition\{', text):
            s = m.end()
            e = text.find('\n}', s)
            d = text[s:e]
            n = re.search(r'name: (\S+),', d).group(1)
            b = d.find('body:') + 5
            self.defs[n] = (s, e, d, s + b)


def parts_of(node, defs):
    """The part definitions a joinBoundedList0 tree concatenates, in order, or None."""
    k = node.kids
    if k and S.tok(k[0]) == JOIN and len(k) == 3:
        a, b = parts_of(k[1], defs), parts_of(k[2], defs)
        return None if a is None or b is None else a + b
    if len(k) == 1 and S.tok(k[0]) in defs:
        return [S.tok(k[0])]
    return None


def main():
    check = '--check' in sys.argv
    names = S.closure()
    files = sorted(p for d in S.SCOPE for p in (S.ROOT / d).glob('*.lex.tex')
                   if 'Uorc.%s.%s' % (p.parent.name, p.name.split('.')[0]) in names)
    stated = collections.Counter()
    changed = []
    for p in files:
        text = p.read_text()
        defs = Defs(text)
        edits = []          # (start, end, replacement)
        part_params = {}    # part name -> (start slot name, [external slot names])
        for dn, (s, e, d, b) in defs.defs.items():
            tree = S.parse(text[b:e], b)
            for rec in blocks(tree):
                params = count_types(field(rec, 'parameters')) if field(rec, 'parameters') else None
                ins_node = field(rec, 'instructions')
                if params is None or not isinstance(ins_node, S.Node):
                    continue
                written = instructions(ins_node)
                if written is not None:
                    state(text, edits, ins_node, [(None, written)], params, stated)
                    continue
                parts = parts_of(ins_node, defs.defs)
                if not parts:
                    continue
                seq = []
                for pn in parts:
                    ps, pe, pd, pb = defs.defs[pn]
                    if 'parameters: [],' not in pd:
                        seq = None
                        break
                    pt = S.parse(text[pb:pe], pb)
                    w = instructions(pt.kids[0]) if pt.kids and isinstance(pt.kids[0], S.Node) else None
                    if w is None:
                        seq = None
                        break
                    seq.append((pn, w, pt.kids[0], ps, pd))
                if seq:
                    state_parts(text, edits, ins_node, seq, params, stated, dn,
                                min(defs.defs[pn][0] for pn in parts))
        if edits:
            new = text
            for a, z, r in sorted(edits, reverse=True):
                new = new[:a] + r + new[z:]
            if new != text:
                changed.append(p)
                if not check:
                    p.write_text(new)
    print('slot references stated:', sum(stated.values()), dict(stated))
    if check and changed:
        print('not at fixpoint:', *[str(c.relative_to(S.ROOT)) for c in changed])
        sys.exit(1)


def blocks(node):
    """Every `record Uorc.Core.Graph::Block {...}` brace group under node."""
    k = node.kids
    for i, x in enumerate(k):
        if isinstance(x, S.Node):
            if x.open == '{' and i >= 2 and S.tok(k[i - 1]) == 'Uorc.Core.Graph::Block' and S.tok(k[i - 2]) == 'record':
                yield x
            yield from blocks(x)


def group_span(text, node):
    """The span of a group node in text, its brackets included."""
    return node.start, node.end


def simulate(seq, params):
    """Slot of each instruction and the slot each reference names. seq: [(part, written)]."""
    slot = params
    out = []
    for part, written in seq:
        for op, refs in written:
            rs = []
            for fi, block, leaves in refs:
                for leaf in leaves:
                    d = int(leaf[0])
                    rs.append((leaf, block, None if block else slot - d))
            out.append((part, slot, op, rs))
            slot += 1
    return out, slot


def chain(names_upto):
    """`let slotA : Nat := 0; (let slotB : Nat := (slotA + 1); ...` as (prefix, closing)."""
    pre, close = [], 0
    for i in range(names_upto):
        val = '0' if i == 0 else '(%s + 1)' % name(i - 1)
        pre.append('(let %s : Nat := %s; ' % (name(i), val))
        close += 1
    return ''.join(pre), ')' * close


def ref(at, target):
    return '(prim subtract %s %s : Nat)' % (name(at), name(target))


def state(text, edits, ins_node, seq, params, stated):
    sim, end = simulate(seq, params)
    if any(t is not None and t < 0 for _, _, _, rs in sim for _, _, t in rs):
        return
    for _, at, op, rs in sim:
        for leaf, block, t in rs:
            if t is not None:
                edits.append((leaf[1], leaf[2], ref(at, t)))
                stated[op] += 1
    a, z = group_span(text, ins_node)
    need = max([at for _, at, _, rs in sim if any(t is not None for _, _, t in rs)], default=-1) + 1
    if need:
        pre, close = chain(need)
        edits.append((a, a, pre))
        edits.append((z, z, close))


def state_parts(text, edits, ins_node, seq, params, stated, block, first_part_start):
    """A block whose list is split into parts: its slot indices are definitions of the module (a
    chain of lets as deep as a large block would pass the depth a module's data form holds), and each
    part, still parameterless, refers to them."""
    sim, end = simulate([(pn, w) for pn, w, _, _, _ in seq], params)
    if any(t is not None and t < 0 for _, _, _, rs in sim for _, _, t in rs):
        return
    slot = lambda i: '(%s%s)' % (block, name(i)[0].upper() + name(i)[1:])
    for part, at, op, rs in sim:
        for leaf, blk, t in rs:
            if t is not None:
                edits.append((leaf[1], leaf[2], '(prim subtract %s %s : Nat)' % (slot(at), slot(t))))
                stated[op] += 1
    need = max([at for _, at, _, rs in sim if any(t is not None for _, _, t in rs)], default=-1) + 1
    out = []
    for i in range(need):
        body = '0' if i == 0 else '(%s + 1)' % slot(i - 1)
        out.append('definition{\n  name: %s,\n  parameters: [],\n  result: Nat,\n  body: %s,\n  executable: true,\n  axioms: []\n}\n\n'
                   % (slot(i)[1:-1], body))
    at = text.rfind('definition{', 0, first_part_start)
    edits.append((at, at, ''.join(out)))


if __name__ == '__main__':
    main()
