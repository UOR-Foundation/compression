# The domains of the hand-written model's quantities, and the pass that states its figures from them.
#
# client/gen/domains.py types the coder's IR; this pass types the hand-written LexLean of the model's
# Search and Core modules the same way. A number in the model is not free: an overflow guard compares a
# natural with the top of the word the model is lowered to, a trie is walked once per bit of its key, a
# bit count becomes bytes eight places at a time, a place-value digit is taken in its base. Each such
# domain is declared once, by its extent, in the wire module Uorc.Wire.Domains (a format identity:
# octet, word, radix), and every other quantity of the domain is derived there from the extent.
#
# The pass reads each definition body as a tree, infers the domain of the operand a literal acts with
# (the other side of a comparison, sum or difference; the parameter it is passed to; the dividend of a
# place-value quotient or remainder; the field it is added to), and states the literal as the quantity
# of that domain it equals. A literal the pass cannot place keeps its numeral and is reported with its
# context, so the figures it leaves are the next declarations to make.
#
#   python3 client/gen/srcdomains.py            rewrite src/Uorc/{Search,Core} in place, report the rest
#   python3 client/gen/srcdomains.py --check    exit 1 if a rewrite would change a file (the pass is at
#                                               its fixpoint on the committed sources)
import collections
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
MODULE = 'Uorc.Wire.Domains'
SCOPE = ['src/Uorc/Search', 'src/Uorc/Core']


def q(name):
    return '(%s::%s)' % (MODULE, name)


# The declared domains: extents first (numerals of the wire module), then quantities derived from them.
# Each entry: name, LexLean body, value. The order is the module's order of definition.
BYTE_BITS = 8    # an octet
WORD_BYTES = 8   # the declared word: the export lowers a natural to a word of eight octets


def declarations():
    """The extents (numerals: a format identity each) and every quantity derived from them, in order."""
    out = [
        ('radix', '2', 2),             # a bit: the place value of binary
        ('byteBits', str(BYTE_BITS), BYTE_BITS),          # an octet
    ]
    prev = 'radix'
    for k in range(2, BYTE_BITS + 1):            # the octet's bit places; the last is its extent
        out.append(('bitPlace%d' % k, '(prim multiply (%s) (radix) : Nat)' % prev, 2 ** k))
        prev = 'bitPlace%d' % k
    out += [
        ('byteBase', '(%s)' % prev, 256),   # the octet's values: radix to the byteBits
        ('byteLast', '(prim subtract (byteBase) 1 : Nat)', 255),
        ('byteHalf', '(prim quotient (byteBase) (radix) 0 : Nat)', 128),
        ('byteBitLast', '(prim subtract (byteBits) 1 : Nat)', BYTE_BITS - 1),
        ('wordBytes', str(WORD_BYTES), WORD_BYTES),
        ('wordBits', '(prim multiply (wordBytes) (byteBits) : Nat)', WORD_BYTES * 8),
    ]
    prev = 'byteBase'
    for k in range(2, WORD_BYTES):   # a word's byte places above the first
        out.append(('bytePlace%d' % k, '(prim multiply (%s) (byteBase) : Nat)' % prev, 256 ** k))
        prev = 'bytePlace%d' % k
    # the word's top, without its extent (which no word holds): its half twice, less one
    half = 256 ** (WORD_BYTES - 1) * 128
    out += [
        ('wordHalf', '(prim multiply (%s) (byteHalf) : Nat)' % prev, half),
        ('wordLast', '((wordHalf) + (prim subtract (wordHalf) 1 : Nat))', 2 * half - 1),
        # a binary tree over the octet alphabet: leaves are the octet values, inner nodes follow them,
        # so a node is named below radix times the alphabet, in one bit more than an octet
        ('nodeBase', '(prim multiply (radix) (byteBase) : Nat)', 512),
        ('nodeLast', '(prim subtract (nodeBase) 1 : Nat)', 511),
        ('nodeBits', '((byteBits) + 1)', 9),
        # a node is written in whole octets, and an inner node as its radix children
        ('nodeBytes', '(prim quotient (prim subtract ((nodeBits) + (byteBits)) 1 : Nat) (byteBits) 0 : Nat)', 2),
        ('nodeRecordBytes', '(prim multiply (radix) (nodeBytes) : Nat)', 4),
    ]
    # the seal: an archive is sealed by a SHA-256 digest (UORC-SHA256-CHAIN-v1), of 32 octets
    out.append(('digestBytes', '32', 32))
    return out


DECLARATIONS = declarations()
VALUE = {n: v for n, _, v in DECLARATIONS}

# The quantities each domain offers a literal acting in it.
DOMAINS = {
    'word': ['wordLast', 'byteBase'] + ['bytePlace%d' % k for k in range(2, WORD_BYTES)],  # a natural of the
    #                                  model is lowered to the declared word: its top and byte places
    'key': ['wordBits', 'nodeBits'],   # a trie keyed by a word (or a tree node) is walked once per key bit
    'node': ['byteBase', 'nodeBase', 'nodeLast'],  # a tree node: the first inner node, and past the last
    'nodes': ['nodeRecordBytes'],      # a count of inner nodes: the octets each is written in
    'digest': ['digestBytes'],         # a seal: the octets of its digest
    'bits': ['byteBits', 'wordBits', 'byteBitLast'],  # a count of bits: whole bytes and words of it
    'byte': ['byteBase', 'byteLast'],  # an octet value
    'place': ['radix', 'byteBase'],    # the base of a place-value digit, in a binary definition
    'digit': ['byteBase'],             # the base a byte digit is taken in
}

# Parameters declared to hold a domain: function (last name segment) -> {index: domain}.
TRIE_WALKS = ['writeTrie', 'readTrie', 'natRead', 'natWrite', 'bucketRead', 'bucketWrite',
              'typeWriteTrie', 'typeReadTrie', 'blockWriteTrie', 'blockReadTrie', 'scheduleWriteTrie',
              'scheduleReadTrie', 'bodyWriteTrie', 'bodyReadTrie', 'wordToNatAux', 'wordOfNatAux',
              'natToU64Bits']
PARAMS = {f: {0: 'key'} for f in TRIE_WALKS}
PARAMS['fitsAddition'] = {0: 'word'}

# Fields declared to hold a domain.
FIELDS = {
    'slotVisits': 'key',     # trie levels visited: a slot lookup walks every bit of the slot key
    'nextNode': 'node', 'root': 'node',
    'bitCount': 'bits', 'filledBits': 'bits', 'encodedBits': 'bits', 'f_filledBits': 'bits',
    'integrity': 'digest',   # an archive's seal
}

# Definitions that take or build a natural in binary places (bits of a word, a binary tree's node
# numbering, a ladder of power-of-two bins): there a factor or divisor of two is the radix.
BINARY = {'wordToNatAux', 'wordOfNatAux', 'natToU64Bits', 'powerTwo', 'indexBitsAux', 'Search.PackedFit.step',
          'Search.HuffmanFit.perform', 'Search.Normalize.buildStep', 'chooseCapacity', 'validBins',
          'Core.ByteBuilder.step'}

# Binders declared to hold a domain, by the definition that binds them.
DIGESTS = ('digest', 'digestBytes', 'expectedId', 'actualId', 'leftBasisId', 'rightBasisId')
BINDERS = {
    'Search.HuffmanFit.perform': {n: 'node' for n in ('first', 'second', 'index', 'best', 'node', 'parent')},
    # the digits of a big-endian octet string
    'Search.Enumerate.step': {'digit': 'byte'}, 'Search.Enumerate.buildStep': {'digit': 'byte'},
    'Search.RopeWords.step': {'digit': 'byte'},
    'Search.InputModels.step': {'nextState': 'byte'},   # a state model's states are named by octets
}
for _d in ('Search.ArchiveBuild.step', 'Search.ArchiveTrial.afterBoundaries', 'Search.ArchiveTrial.dispatch',
           'Search.BasisIdentity.perform', 'Search.HashRope.step', 'Search.Identity.step'):
    BINDERS[_d] = {n: 'digest' for n in DIGESTS}

COMPARE = {'<?', '<=?', '=?', '<', '<=', '>', '>=', '=', '!=?'}
ARITH = {'+', '-'}

TOKEN = re.compile(r'"(?:\\.|[^"\\])*"|[A-Za-z_][\w.\']*(?:::[\w.\']+)*|\d+|:=|=>|<=\?|<\?|=\?|&&|\|\||[(){}\[\],|:+\-*<>=!?/\\^]')


class Node:
    __slots__ = ('open', 'kids', 'parent')

    def __init__(self, open_, parent):
        self.open, self.kids, self.parent = open_, [], parent


def parse(text, base):
    """Tokens of text (offset base) into nested groups; leaves are (token, start, end)."""
    root = Node(None, None)
    cur = root
    for m in TOKEN.finditer(text):
        t = m.group(0)
        if t in '([{':
            if t == '{' and cur.kids and tok(cur.kids[-1]) == 'var':
                cur.kids.pop()                 # var{name: x} is the binder x
                t = 'var'
            n = Node(t, cur)
            cur.kids.append(n)
            cur = n
        elif t in ')]}':
            if cur.open == 'var':
                names = [k for k in cur.kids if isinstance(k, tuple)]
                cur.kids = names[2:3] if len(names) >= 3 and names[0][0] == 'name' else names
            cur = cur.parent or root
        else:
            cur.kids.append((t, base + m.start(), base + m.end()))
    return root


def tok(k):
    return k[0] if isinstance(k, tuple) else None


def short(name):
    return name.split('::')[-1].split('.')[-1]


def domain_of(kids, defn_params):
    """The domain of an operand (a group's kids, or one leaf)."""
    if not kids:
        return None
    head = tok(kids[0])
    if head == 'project' and len(kids) >= 3:
        f = tok(kids[-1])
        return FIELDS.get(f, 'word')
    if head == 'prim' and len(kids) >= 4 and tok(kids[1]) == 'subtract':
        a = domain_of([kids[2]], defn_params)
        if a == 'node' and tok(kids[3]) is None:
            return 'nodes'             # nodes past the first inner one: a count of inner nodes
        if a and a != 'word':
            return a
    if head == 'prim' and len(kids) >= 2:
        op = tok(kids[1])
        if op == 'length':
            return 'digest' if domain_of([kids[2]], defn_params) == 'digest' else 'word'
        if op == 'remainder' and len(kids) >= 4 and domain_of([kids[2]], defn_params) == 'bits':
            return 'bits'                  # the bits past whole octets
        if op in ('subtract', 'multiply', 'quotient', 'remainder'):
            return 'word'
        return None
    if head and short(head) == 'length':
        return 'word'
    if len(kids) == 1:
        t = tok(kids[0])
        if t is not None and re.fullmatch(r'[A-Za-z_][\w\']*', t):
            ty = defn_params.get(t)
            if ty and ty.startswith('@'):
                return ty[1:]
            if t in FIELDS:
                return FIELDS[t]
            if ty == 'Nat':
                return 'word'
            if ty and ty.startswith('@'):
                return ty[1:]
        if isinstance(kids[0], Node):
            return domain_of(kids[0].kids, defn_params)
    # an infix sum or difference keeps its operands' domain
    for i, k in enumerate(kids):
        if tok(k) in ARITH and 0 < i < len(kids) - 1:
            for side in (kids[:i], kids[i + 1:]):
                d = domain_of(side if len(side) > 1 else ([side[0]] if side else []), defn_params)
                if d:
                    return d
    return None


def place(v, domain, compare=False):
    for name in DOMAINS.get(domain, []):
        if domain == 'word' and name != 'wordLast' and not compare:
            continue                   # a byte place bounds a word: it is the extent a comparison tests
        if VALUE[name] == v:
            return name
    return None


def contexts(node, defn_params, out, binary=False):
    kids = node.kids
    head = tok(kids[0]) if kids else None
    for i, k in enumerate(kids):
        if isinstance(k, Node):
            contexts(k, defn_params, out, binary)
            continue
        t = k[0]
        if not t.isdigit() or int(t) <= 1:
            continue
        # a typed literal `(n : T)` other than Nat is not a natural of the word
        if node.open == '(' and len(kids) == 3 and i == 0 and tok(kids[1]) == ':' and tok(kids[2]) != 'Nat':
            continue
        v = int(t)
        doms = []
        compare = False
        if head == 'prim' and len(kids) > 3:
            op = tok(kids[1])
            if op in ('quotient', 'remainder') and i == 3:
                dd = domain_of([kids[2]], defn_params)
                doms += ['bits'] if dd == 'bits' else []
                doms += ['place' if binary else 'digit']
            elif op == 'multiply' and i in (2, 3):
                other = kids[3 if i == 2 else 2]
                dd = domain_of([other], defn_params)
                doms += [dd] if dd in ('bits', 'nodes') else []
                doms += ['place'] if binary else []
            elif op == 'subtract' and i in (2, 3):
                dd = domain_of([kids[3 if i == 2 else 2]], defn_params)
                if dd:
                    doms.append(dd)
        elif head and i >= 1 and short(head) in PARAMS and node.open == '(':
            d = PARAMS[short(head)].get(i - 1)
            if d:
                doms.append(d)
        else:
            if i + 1 < len(kids) and tok(kids[i + 1]) in COMPARE | ARITH and i + 2 < len(kids):
                d = domain_of([kids[i + 2]], defn_params)
                compare = tok(kids[i + 1]) in COMPARE
                if d:
                    doms.append(d)
            if i >= 2 and tok(kids[i - 1]) in COMPARE | ARITH:
                d = domain_of([kids[i - 2]], defn_params)
                compare = compare or tok(kids[i - 1]) in COMPARE
                if d:
                    doms.append(d)
        name = None
        for d in doms:
            name = place(v, d, compare)
            if name:
                break
        out.append((k, name, doms))


def params_of(decl):
    m = re.search(r'parameters: \[(.*?)\],\n', decl, re.S)
    out = {x: 'Nat' for x in re.findall(r'let (\w+) : Nat :=', decl)}
    if m:
        for p in re.finditer(r'(\w+) : ([\w.:]+)', m.group(1)):
            out[p.group(1)] = p.group(2)
    return out


def rewrite(path, residue, derived):
    text = path.read_text()
    edits = []
    pos = 0
    for m in re.finditer(r'definition\{', text):
        start = m.end()
        end = text.find('\n}', start)
        decl = text[start:end]
        name = re.search(r'name: (\S+),', decl)
        name = name.group(1) if name else '?'
        b = decl.find('body:')
        if b < 0:
            continue
        body = decl[b + 5:]
        tree = parse(body, start + b + 5)
        out = []
        qual = '%s.%s.%s' % (path.parent.name, path.name.split('.')[0], name)
        ps = params_of(decl)
        ps.update({b: '@' + d for b, d in BINDERS.get(qual, {}).items()})
        contexts(tree, ps, out, name in BINARY or qual in BINARY)
        for (t, s, e), qn, doms in out:
            if qn:
                edits.append((s, e, q(qn)))
                derived[qn] += 1
            else:
                residue[(int(t), tuple(doms))].append('%s::%s' % (path.stem.split('.')[0], name))
    if not edits:
        return text, text
    new = text
    for s, e, r in sorted(edits, reverse=True):
        new = new[:s] + r + new[e:]
    if '\\importmodule{%s}' % MODULE not in new:
        new = re.sub(r'(\\begin\{lexlean\}\{[^}]*\}\n)', r'\1\\importmodule{%s}\n' % MODULE, new, count=1)
    return text, new


def declared(text):
    """The (name, body) of each definition of a module, as `lexlean fmt` leaves or finds them."""
    out = []
    for d in text.split('definition{')[1:]:
        d = ' '.join(d.split('\n}')[0].split())
        out.append((re.search(r'name: (\S+?),', d).group(1), re.search(r'body: (.*?), (?:axioms|executable)', d).group(1)))
    return out


def module_text():
    out = ['\\begin{lexlean}{%s}' % MODULE, '\\title{Parameters}', '\\begin{semanticmodule}']
    for n, body, _ in DECLARATIONS:
        out.append('definition{\n  name: %s,\n  parameters: [],\n  result: Nat,\n  body: %s,\n  executable: true,\n  axioms: []\n}\n' % (n, body))
    out += ['\\end{semanticmodule}', '\\end{lexlean}', '']
    return '\n'.join(out)


def closure():
    """The modules of Search and Core the shipped client is exported from (Uorc.Client's imports)."""
    roots = [ROOT / 'client/src', ROOT / 'src']
    seen, todo = set(), ['Uorc.Client']
    while todo:
        m = todo.pop()
        if m in seen:
            continue
        seen.add(m)
        for r in roots:
            f = r / (m.replace('.', '/') + '.lex.tex')
            if f.exists():
                todo += re.findall(r'\\importmodule\{([^}]+)\}', f.read_text())
                break
    return seen


def main():
    check = '--check' in sys.argv
    residue = collections.defaultdict(list)
    derived = collections.Counter()
    changed = []
    names = closure()
    files = sorted(p for d in SCOPE for p in (ROOT / d).glob('*.lex.tex')
                   if 'Uorc.%s.%s' % (p.parent.name, p.name.split('.')[0]) in names)
    for p in files:
        old, new = rewrite(p, residue, derived)
        if new != old:
            changed.append(p)
            if not check:
                p.write_text(new)
    dom = ROOT / 'src/Uorc/Wire/Domains.lex.tex'
    if not check:
        if not dom.exists() or declared(dom.read_text()) != declared(module_text()):
            dom.write_text(module_text())   # then `lexlean fmt` lays it out
    elif declared((ROOT / 'src/Uorc/Wire/Domains.lex.tex').read_text()) != declared(module_text()):
        changed.append(ROOT / 'src/Uorc/Wire/Domains.lex.tex')
    print('derived:', dict(derived), sum(derived.values()))
    left = collections.Counter({k: len(v) for k, v in residue.items()})
    print('left: %d literals' % sum(left.values()))
    for (v, doms), n in left.most_common(40):
        print('  %6d  %-22s %s  e.g. %s' % (n, v, ','.join(doms) or '-', residue[(v, doms)][0]))
    if check and changed:
        print('not at fixpoint:', *[str(p.relative_to(ROOT)) for p in changed])
        sys.exit(1)


if __name__ == '__main__':
    main()
