# Generates Uorc/Codec/LzDecode.lex.tex (LexLean source). The generator is a writing aid; the .lex.tex is the source.
# The decoder state is (mem, DecS): one byte table and a record of scalars (see state.py).
# mem holds the probabilities (8192 two-byte entries, bytes [0, 16384)) and then the output.
import sys
import os
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from state import Layout, definer

ALPHA = '0x' + bytes(range(256)).hex()
T = 'Uorc.Codec.Table'
L = Layout('Dec', ['mem'],
           ['input:Bytes', 'pos:Nat', 'range:Nat', 'code:Nat', 'size:Nat', 'total:Nat',
            'state:Nat', 'rep0:Nat', 'rep1:Nat', 'rep2:Nat', 'rep3:Nat'], {})
P, upd = L.P, L.upd
defs = []
D = definer(defs, 'LzDecode')
RB = lambda data, at: f"({T}::readByte {data} {at})"

D('alphabet', [], 'Bytes', ALPHA)
D('byteAt', ['data : Bytes', 'at : Nat'], 'Nat', RB('data', 'at'))
D('singleton', ['value : Nat'], 'Bytes', "(match (prim slice (alphabet) value 1 : (Option Bytes))\n    | Option.none => 0x\n    | Option.some piece => piece)")
D('pow2', ['k : Nat'], 'Nat', "(match k\n    | Nat.zero => 1\n    | Nat.succ m => (2 * (pow2 m)))", 'k')
# The decoder's layout, named once. Each probability block follows the one
# before it (its base is the previous base plus that block's size); the base
# counts are the format's own (states, context bits, tree depths).
def K(name, body):
    D(name, [], 'Nat', str(body))
    return f"({name})"  # a reference to a quantity is parenthesised
STATES = K('states', 12)
IS_REP = K('isRepBase', STATES)
IS_REP_G0 = K('isRepG0Base', f"({IS_REP} + {STATES})")
IS_REP_G1 = K('isRepG1Base', f"({IS_REP_G0} + {STATES})")
IS_REP_G2 = K('isRepG2Base', f"({IS_REP_G1} + {STATES})")
IS_REP0_LONG = K('isRep0LongBase', f"({IS_REP_G2} + {STATES})")
LIT = K('literalBase', f"({IS_REP0_LONG} + {STATES})")
RADIX = K('radix', 2)
BYTE_BITS = K('byteBits', 8)
BYTE_BASE = K('byteBase', f"(pow2 {BYTE_BITS})")
LC = K('literalContextBits', 3)
# a literal's coder: the plain tree and the two matched trees (match bit 0, 1)
LIT_CODER = K('literalCoder', f"(({RADIX} + 1) * {BYTE_BASE})")
LIT_SHIFT = K('literalContextShift', f"(pow2 (prim subtract {BYTE_BITS} {LC} : Nat))")
SLOT = K('slotBase', f"({LIT} + ({LIT_CODER} * (pow2 {LC})))")
LEN_STATES = K('lengthStates', 4)
SLOT_BITS = K('slotBits', 6)
SLOT_TREE = K('slotTree', f"(pow2 {SLOT_BITS})")
SPEC = K('specBase', f"({SLOT} + ({LEN_STATES} * {SLOT_TREE}))")
START_SLOT = K('startSlot', 4)
END_SLOT = K('endSlot', 14)
ALIGN_BITS = K('alignBits', 4)
ALIGN_TREE = K('alignTree', f"(pow2 {ALIGN_BITS})")
# the reverse trees of slots startSlot..endSlot index from 1 up to the full distances less endSlot
ALIGN = K('alignBase', f"({SPEC} + (prim subtract ((pow2 (prim quotient {END_SLOT} {RADIX} 0 : Nat)) + 1) {END_SLOT} : Nat))")
LENL = K('lengthBase', f"({ALIGN} + {ALIGN_TREE})")
LOW_BITS = K('lowBits', 3)
HIGH_BITS = K('highBits', 8)
LOW = K('lowSymbols', f"(pow2 {LOW_BITS})")
# each length tree's block holds twice its symbols (index m from 1 doubles each level)
LOW_AT = K('lowAt', RADIX)
MID_AT = K('midAt', f"({LOW_AT} + ({RADIX} * {LOW}))")
HIGH_AT = K('highAt', f"({MID_AT} + ({RADIX} * {LOW}))")
LENR = K('repLengthBase', f"({LENL} + ({HIGH_AT} + ({RADIX} * (pow2 {HIGH_BITS}))))")
MIN_MATCH = K('minMatch', 2)
PROB_BITS = K('probabilityBits', 11)
PROB_RANGE = K('probabilityRange', f"(pow2 {PROB_BITS})")

PROB_COUNT_BITS = K('probabilityCountBits', 13)
PROB_CELL = K('probabilityCell', RADIX)
OUT_AT = K('outputBase', f"({PROB_CELL} * (pow2 {PROB_COUNT_BITS}))")
LENGTH_BYTES = K('lengthBytes', BYTE_BITS)
# the header: the output length, the coder's zero byte, then its code word
START_POS = K('startPosition', 13)
CODE_AT = K('codeAt', f"({LENGTH_BYTES} + 1)")
CODE_BYTES = K('codeBytes', f"(prim subtract {START_POS} {CODE_AT} : Nat)")
CODE_BITS = f"({CODE_BYTES} * {BYTE_BITS})"
TOP = K('rangeTop', f"(pow2 (prim subtract {CODE_BITS} {BYTE_BITS} : Nat))")
MOVE = K('moveScale', 32)

D('depthFor', ['fuel : Nat', 'n : Nat', 'depth : Nat'], 'Nat', "(match fuel\n    | Nat.zero => depth\n    | Nat.succ m => (if (n <=? (pow2 depth)) then depth else (depthFor m n (depth + 1))))", 'fuel')
_n = upd('d', range=f"({P('d','range')} * {BYTE_BASE})", code=f"(({P('d','code')} * {BYTE_BASE}) + {RB(P('d','input'), P('d','pos'))})", pos=f"({P('d','pos')} + 1)")
D('normalize', ['d : Dec'], 'Dec', f"(if ({P('d','range')} <? {TOP}) then {_n} else d)")
D('prob', ['d : Dec', 'at : Nat'], 'Nat', f"({T}::get {PROB_CELL} {P('d','mem')} at)")
body = (f"(let v : Nat := (prob d at); (let bound : Nat := ((prim quotient {P('d','range')} {PROB_RANGE} 0 : Nat) * v); "
        f"(if ({P('d','code')} <? bound) then (0, (normalize {upd('d', range='bound', mem=f'({T}::set {PROB_CELL} {P(chr(100),chr(109)+chr(101)+chr(109))} at (v + (prim quotient (prim subtract {PROB_RANGE} v : Nat) {MOVE} 0 : Nat)))')})) "
        f"else (1, (normalize {upd('d', range=f'(prim subtract {P(chr(100),chr(114)+chr(97)+chr(110)+chr(103)+chr(101))} bound : Nat)', code=f'(prim subtract {P(chr(100),chr(99)+chr(111)+chr(100)+chr(101))} bound : Nat)', mem=f'({T}::set {PROB_CELL} {P(chr(100),chr(109)+chr(101)+chr(109))} at (prim subtract v (prim quotient v {MOVE} 0 : Nat) : Nat))')})))))")
D('decodeBit', ['d : Dec', 'at : Nat'], '(Nat * Dec)', body)
body = (f"(match fuel\n    | Nat.zero => (acc, d)\n    | Nat.succ m => (let half : Nat := (prim quotient {P('d','range')} {RADIX} 0 : Nat); "
        f"(if ({P('d','code')} <? half) then (directLoop m (normalize {upd('d', range='half')}) ({RADIX} * acc)) "
        f"else (directLoop m (normalize {upd('d', range='half', code=f'(prim subtract {P(chr(100),chr(99)+chr(111)+chr(100)+chr(101))} half : Nat)')}) (({RADIX} * acc) + 1)))))")
D('directLoop', ['fuel : Nat', 'd : Dec', 'acc : Nat'], '(Nat * Dec)', body, 'fuel')
D('treeLoop', ['fuel : Nat', 'd : Dec', 'base : Nat', 'm : Nat'], '(Nat * Dec)',
  "(match fuel\n    | Nat.zero => (m, d)\n    | Nat.succ k => (let r : (Nat * Dec) := (decodeBit d (base + m)); (let bit : Nat := (first r); (treeLoop k (second r) base ((2 * m) + bit)))))", 'fuel')
D('tree', ['d : Dec', 'base : Nat', 'bits : Nat'], '(Nat * Dec)', "(let r : (Nat * Dec) := (treeLoop bits d base 1); (let v : Nat := (first r); ((prim subtract v (pow2 bits) : Nat), (second r))))")
D('reverseLoop', ['fuel : Nat', 'd : Dec', 'base : Nat', 'm : Nat', 'weight : Nat', 'acc : Nat'], '(Nat * Dec)',
  "(match fuel\n    | Nat.zero => (acc, d)\n    | Nat.succ k => (let r : (Nat * Dec) := (decodeBit d (base + m)); (let bit : Nat := (first r); (reverseLoop k (second r) base ((2 * m) + bit) (2 * weight) (acc + (weight * bit))))))", 'fuel')
D('matchedLoop', ['fuel : Nat', 'd : Dec', 'base : Nat', 'matchByte : Nat', 'sym : Nat'], '(Nat * Dec)',
  f"(match fuel\n    | Nat.zero => (sym, d)\n    | Nat.succ k => (let mbit : Nat := (prim remainder (prim quotient matchByte (pow2 k) 0 : Nat) {RADIX} 0 : Nat); (let r : (Nat * Dec) := (decodeBit d ((base + ((1 + mbit) * {BYTE_BASE})) + sym)); (let bit : Nat := (first r); (let next : Nat := (({RADIX} * sym) + bit); (if (bit =? mbit) then (matchedLoop k (second r) base matchByte next) else (let rest : (Nat * Dec) := (treeLoop k (second r) base next); rest)))))))", 'fuel')
LC_ = (f"(let c : (Nat * Dec) := (decodeBit d base); (let cb : Nat := (first c); (if (cb =? 0) then (tree (second c) (base + {LOW_AT}) {LOW_BITS}) else (let c2 : (Nat * Dec) := (decodeBit (second c) (base + 1)); (let cb2 : Nat := (first c2); "
      f"(if (cb2 =? 0) then (let t : (Nat * Dec) := (tree (second c2) (base + {MID_AT}) {LOW_BITS}); (let tv : Nat := (first t); (({LOW} + tv), (second t)))) else (let t : (Nat * Dec) := (tree (second c2) (base + {HIGH_AT}) {HIGH_BITS}); (let tv : Nat := (first t); ((({RADIX} * {LOW}) + tv), (second t))))))))))")
D('lengthCode', ['d : Dec', 'base : Nat'], '(Nat * Dec)', LC_)
LS_LAST = f"(prim subtract {LEN_STATES} 1 : Nat)"
DS = (f"(let ls : Nat := (if (lenCode <? {LS_LAST}) then lenCode else {LS_LAST}); (let s : (Nat * Dec) := (tree d ({SLOT} + (ls * {SLOT_TREE})) {SLOT_BITS}); (let slot : Nat := (first s); "
      f"(if (slot <? {START_SLOT}) then s else (let fb : Nat := (prim subtract (prim quotient slot {RADIX} 0 : Nat) 1 : Nat); (let base : Nat := (({RADIX} + (prim remainder slot {RADIX} 0 : Nat)) * (pow2 fb)); "
      f"(if (slot <? {END_SLOT}) then (let r : (Nat * Dec) := (reverseLoop fb (second s) (prim subtract ({SPEC} + base) slot : Nat) 1 1 0); (let rv : Nat := (first r); ((base + rv), (second r)))) "
      f"else (let hi : (Nat * Dec) := (directLoop (prim subtract fb {ALIGN_BITS} : Nat) (second s) 0); (let hv : Nat := (first hi); (let lo : (Nat * Dec) := (reverseLoop {ALIGN_BITS} (second hi) {ALIGN} 1 1 0); (let lv : Nat := (first lo); (((base + ({ALIGN_TREE} * hv)) + lv), (second lo)))))))))))))")
D('distance', ['d : Dec', 'lenCode : Nat'], '(Nat * Dec)', DS)
D('afterLiteral', ['s : Nat'], 'Nat', "(if (s <? 4) then 0 else (if (s <? 10) then (prim subtract s 3 : Nat) else (prim subtract s 6 : Nat)))")
D('afterMatch', ['s : Nat'], 'Nat', "(if (s <? 7) then 7 else 10)")
D('afterRep', ['s : Nat'], 'Nat', "(if (s <? 7) then 8 else 11)")
D('afterShort', ['s : Nat'], 'Nat', "(if (s <? 7) then 9 else 11)")
D('outAt', ['d : Dec', 'at : Nat'], 'Nat', f"({T}::get 1 {P('d','mem')} ({OUT_AT} + at))")
D('emit', ['d : Dec', 'value : Nat'], 'Dec', upd('d', mem=f"({T}::set 1 {P('d','mem')} ({OUT_AT} + {P('d','size')}) value)", size=f"({P('d','size')} + 1)"))
# the byte one rep0 back, read before the state is consumed
BACK = lambda v: f"(outAt {v} (prim subtract {P(v,'size')} ({P(v,'rep0')} + 1) : Nat))"
D('copyLoop', ['fuel : Nat', 'd : Dec'], 'Dec',
  f"(match fuel\n    | Nat.zero => d\n    | Nat.succ k => (if ({P('d','total')} <=? {P('d','size')}) then d else (let b : Nat := {BACK('d')}; (copyLoop k (emit d b)))))", 'fuel')
lit = (f"(let s0 : Nat := {P('d','state')}; (let size0 : Nat := {P('d','size')}; (let prev : Nat := (if (size0 =? 0) then 0 else (outAt d (prim subtract size0 1 : Nat))); "
       f"(let base : Nat := ({LIT} + ({LIT_CODER} * (prim quotient prev {LIT_SHIFT} 0 : Nat))); "
       f"(let mb : Nat := (if ({P('d','rep0')} <? size0) then {BACK('d')} else 0); "
       f"(let r : (Nat * Dec) := (if (s0 <? 7) then (treeLoop {BYTE_BITS} d base 1) else (matchedLoop {BYTE_BITS} d base mb 1)); "
       f"(let sym : Nat := (first r); (let e : Dec := (emit (second r) (prim subtract sym {BYTE_BASE} : Nat)); {upd('e', state='(afterLiteral s0)')}))))))))")
D('literal', ['d : Dec'], 'Dec', lit)
mt = (f"(let l : (Nat * Dec) := (lengthCode d {LENL}); (let len : Nat := (first l); (let dist : (Nat * Dec) := (distance (second l) len); (let dv : Nat := (first dist); (let e : Dec := (second dist); "
      f"(copyLoop (len + {MIN_MATCH}) {upd('e', state=f'(afterMatch {P(chr(101),chr(115)+chr(116)+chr(97)+chr(116)+chr(101))})', rep0='dv', rep1=P('e','rep0'), rep2=P('e','rep1'), rep3=P('e','rep2'))}))))))")
D('newMatch', ['d : Dec'], 'Dec', mt)
D('repMatch', ['d : Dec'], 'Dec', f"(let l : (Nat * Dec) := (lengthCode d {LENR}); (let len : Nat := (first l); (let e : Dec := (second l); (copyLoop (len + {MIN_MATCH}) {upd('e', state=f'(afterRep {P(chr(101),chr(115)+chr(116)+chr(97)+chr(116)+chr(101))})')}))))")
rp = (f"(let s : Nat := {P('d','state')}; (let g0 : (Nat * Dec) := (decodeBit d ({IS_REP_G0} + s)); (let g0b : Nat := (first g0); (let a : Dec := (second g0); "
      f"(if (g0b =? 0) then (let long : (Nat * Dec) := (decodeBit a ({IS_REP0_LONG} + s)); (let lb : Nat := (first long); (let b : Dec := (second long); (if (lb =? 0) then (let back : Nat := {BACK('b')}; (let e : Dec := (emit b back); {upd('e', state='(afterShort s)')})) else (repMatch b))))) "
      f"else (let g1 : (Nat * Dec) := (decodeBit a ({IS_REP_G1} + s)); (let g1b : Nat := (first g1); (let b : Dec := (second g1); (if (g1b =? 0) then (repMatch {upd('b', rep0=P('b','rep1'), rep1=P('b','rep0'))}) "
      f"else (let g2 : (Nat * Dec) := (decodeBit b ({IS_REP_G2} + s)); (let g2b : Nat := (first g2); (let c : Dec := (second g2); (if (g2b =? 0) then (repMatch {upd('c', rep0=P('c','rep2'), rep1=P('c','rep0'), rep2=P('c','rep1'))}) else (repMatch {upd('c', rep0=P('c','rep3'), rep1=P('c','rep0'), rep2=P('c','rep1'), rep3=P('c','rep2'))}))))))))))))))")
D('rep', ['d : Dec'], 'Dec', rp)
st = (f"(let s : Nat := {P('d','state')}; (let m : (Nat * Dec) := (decodeBit d s); (let mb : Nat := (first m); "
      f"(if (mb =? 0) then (literal (second m)) else (let r : (Nat * Dec) := (decodeBit (second m) ({IS_REP} + s)); (let rb : Nat := (first r); (if (rb =? 0) then (newMatch (second r)) else (rep (second r)))))))))")
D('step', ['d : Dec'], 'Dec', st)
# runs steps until the output is complete; the state moves through and is never shared
D('decodeLoop', ['fuel : Nat', 'd : Dec'], 'Dec', f"(match fuel\n    | Nat.zero => d\n    | Nat.succ k => (let t : Nat := {P('d','total')}; (let s : Nat := {P('d','size')}; (if (t <=? s) then d else (decodeLoop k (step d))))))", 'fuel')
# little-endian, read from the top byte down so that no weight passes 2^56
D('readLength', ['data : Bytes', 'fuel : Nat', 'at : Nat', 'acc : Nat'], 'Nat',
  f"(match fuel\n    | Nat.zero => acc\n    | Nat.succ k => (readLength data k at (({BYTE_BASE} * acc) + (byteAt data (at + k)))))", 'fuel')
# big-endian, the code word after the length and the coder's zero byte
D('readBig', ['data : Bytes', 'fuel : Nat', 'at : Nat', 'acc : Nat'], 'Nat',
  f"(match fuel\n    | Nat.zero => acc\n    | Nat.succ k => (readBig data k (at + 1) (({BYTE_BASE} * acc) + (byteAt data at))))", 'fuel')
# the output starts as `history` (h bytes); matches may reach into it and only the bytes after it are returned
startW = (f"(let h : Nat := (prim length history : Nat); (let total : Nat := (readLength input {LENGTH_BYTES} 0 0); (let code : Nat := (readBig input {CODE_BYTES} {CODE_AT} 0); "
          + L.make({'mem': f"(prim append (prim append ({T}::fill {PROB_CELL} {PROB_COUNT_BITS} (prim quotient {PROB_RANGE} {RADIX} 0 : Nat)) history : Bytes) ({T}::fill 1 (depthFor ({LENGTH_BYTES} * {BYTE_BITS}) total 0) 0) : Bytes)"},
                   dict(input='input', pos=START_POS, range=f"(prim subtract (pow2 {CODE_BITS}) 1 : Nat)", code='code', size='h', total='(h + total)', state='0', rep0='0', rep1='0', rep2='0', rep3='0')) + ")))")
D('startWith', ['history : Bytes', 'input : Bytes'], 'Dec', startW)
decW = (f"(let h : Nat := (prim length history : Nat); (let d : Dec := (startWith history input); (let steps : Nat := ((prim subtract {P('d','total')} h : Nat) + 1); "
        f"(let f : Dec := (decodeLoop steps d); (let size : Nat := {P('f','size')}; (match (prim slice {P('f','mem')} ({OUT_AT} + h) (prim subtract size h : Nat) : (Option Bytes))\n    | Option.none => 0x\n    | Option.some b => b))))))")
D('decodeWith', ['history : Bytes', 'input : Bytes'], 'Bytes', decW)
D('decode', ['input : Bytes'], 'Bytes', "(decodeWith 0x input)")
src = ("\\begin{lexlean}{Uorc.Codec.LzDecode}\n\\importmodule{Uorc.Codec.Table}\n\\title{Parameters}\n\\begin{semanticmodule}\n"
       + L.structure() + "\n" + "\n".join(defs) + "\\end{semanticmodule}\n\\end{lexlean}\n")
open(os.path.join(HERE, '..', 'src', 'Uorc', 'Codec', 'LzDecode.lex.tex'), 'w').write(src)
