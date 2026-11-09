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
D('depthFor', ['fuel : Nat', 'n : Nat', 'depth : Nat'], 'Nat', "(match fuel\n    | Nat.zero => depth\n    | Nat.succ m => (if (n <=? (pow2 depth)) then depth else (depthFor m n (depth + 1))))", 'fuel')
_n = upd('d', range=f"({P('d','range')} * 256)", code=f"(({P('d','code')} * 256) + {RB(P('d','input'), P('d','pos'))})", pos=f"({P('d','pos')} + 1)")
D('normalize', ['d : Dec'], 'Dec', f"(if ({P('d','range')} <? 16777216) then {_n} else d)")
D('prob', ['d : Dec', 'at : Nat'], 'Nat', f"({T}::get 2 {P('d','mem')} at)")
body = (f"(let v : Nat := (prob d at); (let bound : Nat := ((prim quotient {P('d','range')} 2048 0 : Nat) * v); "
        f"(if ({P('d','code')} <? bound) then (0, (normalize {upd('d', range='bound', mem=f'({T}::set 2 {P(chr(100),chr(109)+chr(101)+chr(109))} at (v + (prim quotient (prim subtract 2048 v : Nat) 32 0 : Nat)))')})) "
        f"else (1, (normalize {upd('d', range=f'(prim subtract {P(chr(100),chr(114)+chr(97)+chr(110)+chr(103)+chr(101))} bound : Nat)', code=f'(prim subtract {P(chr(100),chr(99)+chr(111)+chr(100)+chr(101))} bound : Nat)', mem=f'({T}::set 2 {P(chr(100),chr(109)+chr(101)+chr(109))} at (prim subtract v (prim quotient v 32 0 : Nat) : Nat))')})))))")
D('decodeBit', ['d : Dec', 'at : Nat'], '(Nat * Dec)', body)
body = (f"(match fuel\n    | Nat.zero => (acc, d)\n    | Nat.succ m => (let half : Nat := (prim quotient {P('d','range')} 2 0 : Nat); "
        f"(if ({P('d','code')} <? half) then (directLoop m (normalize {upd('d', range='half')}) (2 * acc)) "
        f"else (directLoop m (normalize {upd('d', range='half', code=f'(prim subtract {P(chr(100),chr(99)+chr(111)+chr(100)+chr(101))} half : Nat)')}) ((2 * acc) + 1)))))")
D('directLoop', ['fuel : Nat', 'd : Dec', 'acc : Nat'], '(Nat * Dec)', body, 'fuel')
D('treeLoop', ['fuel : Nat', 'd : Dec', 'base : Nat', 'm : Nat'], '(Nat * Dec)',
  "(match fuel\n    | Nat.zero => (m, d)\n    | Nat.succ k => (let r : (Nat * Dec) := (decodeBit d (base + m)); (let bit : Nat := (first r); (treeLoop k (second r) base ((2 * m) + bit)))))", 'fuel')
D('tree', ['d : Dec', 'base : Nat', 'bits : Nat'], '(Nat * Dec)', "(let r : (Nat * Dec) := (treeLoop bits d base 1); (let v : Nat := (first r); ((prim subtract v (pow2 bits) : Nat), (second r))))")
D('reverseLoop', ['fuel : Nat', 'd : Dec', 'base : Nat', 'm : Nat', 'weight : Nat', 'acc : Nat'], '(Nat * Dec)',
  "(match fuel\n    | Nat.zero => (acc, d)\n    | Nat.succ k => (let r : (Nat * Dec) := (decodeBit d (base + m)); (let bit : Nat := (first r); (reverseLoop k (second r) base ((2 * m) + bit) (2 * weight) (acc + (weight * bit))))))", 'fuel')
D('matchedLoop', ['fuel : Nat', 'd : Dec', 'base : Nat', 'matchByte : Nat', 'sym : Nat'], '(Nat * Dec)',
  "(match fuel\n    | Nat.zero => (sym, d)\n    | Nat.succ k => (let mbit : Nat := (prim remainder (prim quotient matchByte (pow2 k) 0 : Nat) 2 0 : Nat); (let r : (Nat * Dec) := (decodeBit d ((base + ((1 + mbit) * 256)) + sym)); (let bit : Nat := (first r); (let next : Nat := ((2 * sym) + bit); (if (bit =? mbit) then (matchedLoop k (second r) base matchByte next) else (let rest : (Nat * Dec) := (treeLoop k (second r) base next); rest)))))))", 'fuel')
LC = ("(let c : (Nat * Dec) := (decodeBit d base); (let cb : Nat := (first c); (if (cb =? 0) then (tree (second c) (base + 2) 3) else (let c2 : (Nat * Dec) := (decodeBit (second c) (base + 1)); (let cb2 : Nat := (first c2); "
      "(if (cb2 =? 0) then (let t : (Nat * Dec) := (tree (second c2) (base + 18) 3); (let tv : Nat := (first t); ((8 + tv), (second t)))) else (let t : (Nat * Dec) := (tree (second c2) (base + 34) 8); (let tv : Nat := (first t); ((16 + tv), (second t))))))))))")
D('lengthCode', ['d : Dec', 'base : Nat'], '(Nat * Dec)', LC)
SLOT = 6216; SPEC = 6472; ALIGN = 6587; LENL = 6603; LENR = 7149; LIT = 72
DS = (f"(let ls : Nat := (if (lenCode <? 3) then lenCode else 3); (let s : (Nat * Dec) := (tree d ({SLOT} + (ls * 64)) 6); (let slot : Nat := (first s); "
      "(if (slot <? 4) then s else (let fb : Nat := (prim subtract (prim quotient slot 2 0 : Nat) 1 : Nat); (let base : Nat := ((2 + (prim remainder slot 2 0 : Nat)) * (pow2 fb)); "
      f"(if (slot <? 14) then (let r : (Nat * Dec) := (reverseLoop fb (second s) (prim subtract ({SPEC} + base) slot : Nat) 1 1 0); (let rv : Nat := (first r); ((base + rv), (second r)))) "
      f"else (let hi : (Nat * Dec) := (directLoop (prim subtract fb 4 : Nat) (second s) 0); (let hv : Nat := (first hi); (let lo : (Nat * Dec) := (reverseLoop 4 (second hi) {ALIGN} 1 1 0); (let lv : Nat := (first lo); (((base + (16 * hv)) + lv), (second lo)))))))))))))")
D('distance', ['d : Dec', 'lenCode : Nat'], '(Nat * Dec)', DS)
D('afterLiteral', ['s : Nat'], 'Nat', "(if (s <? 4) then 0 else (if (s <? 10) then (prim subtract s 3 : Nat) else (prim subtract s 6 : Nat)))")
D('afterMatch', ['s : Nat'], 'Nat', "(if (s <? 7) then 7 else 10)")
D('afterRep', ['s : Nat'], 'Nat', "(if (s <? 7) then 8 else 11)")
D('afterShort', ['s : Nat'], 'Nat', "(if (s <? 7) then 9 else 11)")
D('outAt', ['d : Dec', 'at : Nat'], 'Nat', f"({T}::get 1 {P('d','mem')} (16384 + at))")
D('emit', ['d : Dec', 'value : Nat'], 'Dec', upd('d', mem=f"({T}::set 1 {P('d','mem')} (16384 + {P('d','size')}) value)", size=f"({P('d','size')} + 1)"))
# the byte one rep0 back, read before the state is consumed
BACK = lambda v: f"(outAt {v} (prim subtract {P(v,'size')} ({P(v,'rep0')} + 1) : Nat))"
D('copyLoop', ['fuel : Nat', 'd : Dec'], 'Dec',
  f"(match fuel\n    | Nat.zero => d\n    | Nat.succ k => (if ({P('d','total')} <=? {P('d','size')}) then d else (let b : Nat := {BACK('d')}; (copyLoop k (emit d b)))))", 'fuel')
lit = (f"(let s0 : Nat := {P('d','state')}; (let size0 : Nat := {P('d','size')}; (let prev : Nat := (if (size0 =? 0) then 0 else (outAt d (prim subtract size0 1 : Nat))); "
       f"(let base : Nat := ({LIT} + (768 * (prim quotient prev 32 0 : Nat))); "
       f"(let mb : Nat := (if ({P('d','rep0')} <? size0) then {BACK('d')} else 0); "
       f"(let r : (Nat * Dec) := (if (s0 <? 7) then (treeLoop 8 d base 1) else (matchedLoop 8 d base mb 1)); "
       f"(let sym : Nat := (first r); (let e : Dec := (emit (second r) (prim subtract sym 256 : Nat)); {upd('e', state='(afterLiteral s0)')}))))))))")
D('literal', ['d : Dec'], 'Dec', lit)
mt = (f"(let l : (Nat * Dec) := (lengthCode d {LENL}); (let len : Nat := (first l); (let dist : (Nat * Dec) := (distance (second l) len); (let dv : Nat := (first dist); (let e : Dec := (second dist); "
      f"(copyLoop (len + 2) {upd('e', state=f'(afterMatch {P(chr(101),chr(115)+chr(116)+chr(97)+chr(116)+chr(101))})', rep0='dv', rep1=P('e','rep0'), rep2=P('e','rep1'), rep3=P('e','rep2'))}))))))")
D('newMatch', ['d : Dec'], 'Dec', mt)
D('repMatch', ['d : Dec'], 'Dec', f"(let l : (Nat * Dec) := (lengthCode d {LENR}); (let len : Nat := (first l); (let e : Dec := (second l); (copyLoop (len + 2) {upd('e', state=f'(afterRep {P(chr(101),chr(115)+chr(116)+chr(97)+chr(116)+chr(101))})')}))))")
rp = (f"(let s : Nat := {P('d','state')}; (let g0 : (Nat * Dec) := (decodeBit d (24 + s)); (let g0b : Nat := (first g0); (let a : Dec := (second g0); "
      f"(if (g0b =? 0) then (let long : (Nat * Dec) := (decodeBit a (60 + s)); (let lb : Nat := (first long); (let b : Dec := (second long); (if (lb =? 0) then (let back : Nat := {BACK('b')}; (let e : Dec := (emit b back); {upd('e', state='(afterShort s)')})) else (repMatch b))))) "
      f"else (let g1 : (Nat * Dec) := (decodeBit a (36 + s)); (let g1b : Nat := (first g1); (let b : Dec := (second g1); (if (g1b =? 0) then (repMatch {upd('b', rep0=P('b','rep1'), rep1=P('b','rep0'))}) "
      f"else (let g2 : (Nat * Dec) := (decodeBit b (48 + s)); (let g2b : Nat := (first g2); (let c : Dec := (second g2); (if (g2b =? 0) then (repMatch {upd('c', rep0=P('c','rep2'), rep1=P('c','rep0'), rep2=P('c','rep1'))}) else (repMatch {upd('c', rep0=P('c','rep3'), rep1=P('c','rep0'), rep2=P('c','rep1'), rep3=P('c','rep2'))}))))))))))))))")
D('rep', ['d : Dec'], 'Dec', rp)
st = (f"(let s : Nat := {P('d','state')}; (let m : (Nat * Dec) := (decodeBit d s); (let mb : Nat := (first m); "
      "(if (mb =? 0) then (literal (second m)) else (let r : (Nat * Dec) := (decodeBit (second m) (12 + s)); (let rb : Nat := (first r); (if (rb =? 0) then (newMatch (second r)) else (rep (second r)))))))))")
D('step', ['d : Dec'], 'Dec', st)
# runs steps until the output is complete; the state moves through and is never shared
D('decodeLoop', ['fuel : Nat', 'd : Dec'], 'Dec', f"(match fuel\n    | Nat.zero => d\n    | Nat.succ k => (let t : Nat := {P('d','total')}; (let s : Nat := {P('d','size')}; (if (t <=? s) then d else (decodeLoop k (step d))))))", 'fuel')
# little-endian, read from the top byte down so that no weight passes 2^56
D('readLength', ['data : Bytes', 'fuel : Nat', 'at : Nat', 'acc : Nat'], 'Nat',
  "(match fuel\n    | Nat.zero => acc\n    | Nat.succ k => (readLength data k at ((256 * acc) + (byteAt data (at + k)))))", 'fuel')
# the output starts as `history` (h bytes); matches may reach into it and only the bytes after it are returned
startW = ("(let h : Nat := (prim length history : Nat); (let total : Nat := (readLength input 8 0 0); (let code : Nat := ((256 * ((256 * ((256 * (byteAt input 9)) + (byteAt input 10))) + (byteAt input 11))) + (byteAt input 12)); "
          + L.make({'mem': f"(prim append (prim append ({T}::fill 2 13 1024) history : Bytes) ({T}::fill 1 (depthFor 64 total 0) 0) : Bytes)"},
                   dict(input='input', pos='13', range='4294967295', code='code', size='h', total='(h + total)', state='0', rep0='0', rep1='0', rep2='0', rep3='0')) + ")))")
D('startWith', ['history : Bytes', 'input : Bytes'], 'Dec', startW)
decW = (f"(let h : Nat := (prim length history : Nat); (let d : Dec := (startWith history input); (let steps : Nat := ((prim subtract {P('d','total')} h : Nat) + 1); "
        f"(let f : Dec := (decodeLoop steps d); (let size : Nat := {P('f','size')}; (match (prim slice {P('f','mem')} (16384 + h) (prim subtract size h : Nat) : (Option Bytes))\n    | Option.none => 0x\n    | Option.some b => b))))))")
D('decodeWith', ['history : Bytes', 'input : Bytes'], 'Bytes', decW)
D('decode', ['input : Bytes'], 'Bytes', "(decodeWith 0x input)")
src = ("\\begin{lexlean}{Uorc.Codec.LzDecode}\n\\importmodule{Uorc.Codec.Table}\n\\title{Parameters}\n\\begin{semanticmodule}\n"
       + L.structure() + "\n" + "\n".join(defs) + "\\end{semanticmodule}\n\\end{lexlean}\n")
open(os.path.join(HERE, '..', 'src', 'Uorc', 'Codec', 'LzDecode.lex.tex'), 'w').write(src)
