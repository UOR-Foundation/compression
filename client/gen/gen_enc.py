# Generates Uorc/Codec/LzEncode.lex.tex (LexLean source). The generator is a writing aid; the .lex.tex is the source.
# The encoder state is (mem, EncS): one byte table and a record of scalars (see state.py).
# mem holds the probabilities (bytes [0, 16384)), the 3-byte hash table (65536 four-byte entries from byte
# 16384: the latest position of each hash), the 4-byte hash heads (h4size four-byte entries from byte 278528),
# the chain through them (four-byte entries from word chainAt), and then the output from byte outBase.
# h4size is 2^hb with hb the input's bit length clamped to [16, 22], so small inputs keep small tables.
import sys
import os
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from state import Layout, definer

T = 'Uorc.Codec.Table'; DEC = 'Uorc.Codec.LzDecode'
L = Layout('Enc', ['mem'],
           ['data:Bytes', 'n:Nat', 'low:Nat', 'rng:Nat', 'cache:Nat', 'csize:Nat', 'outSize:Nat',
            'state:Nat', 'rep0:Nat', 'rep1:Nat', 'rep2:Nat', 'rep3:Nat', 'at:Nat', 'outBase:Nat',
            'h4div:Nat', 'h4size:Nat', 'chainAt:Nat'], {})
P, upd = L.P, L.upd
defs = []
D = definer(defs, 'LzEncode')
B = lambda data, a: f"({T}::readByte {data} {a})"
pow2 = lambda k: f"({DEC}::pow2 {k})"
SLOT = 6216; SPEC = 6472; ALIGN = 6587; LENL = 6603; LENR = 7149; LIT = 72
q = lambda v, f: P(v, f)

D('emitByte', ['e : Enc', 'value : Nat'], 'Enc', upd('e', mem=f"({T}::set 1 {P('e','mem')} ({P('e','outBase')} + {P('e','outSize')}) value)", outSize=f"({P('e','outSize')} + 1)"))
D('emitRun', ['fuel : Nat', 'e : Enc', 'value : Nat'], 'Enc', "(match fuel\n    | Nat.zero => e\n    | Nat.succ k => (emitRun k (emitByte e value) value))", 'fuel')
sh = (f"(let low : Nat := {P('e','low')}; (let cache : Nat := {P('e','cache')}; (let csize : Nat := {P('e','csize')}; "
      f"(if ((low <? 4278190080) || (4294967296 <=? low)) then (let carry : Nat := (prim quotient low 4294967296 0 : Nat); "
      f"(let a : Enc := (emitByte e (prim remainder (cache + carry) 256 0 : Nat)); (let b : Enc := (emitRun (prim subtract csize 1 : Nat) a (prim remainder (255 + carry) 256 0 : Nat)); "
      f"{upd('b', cache='(prim remainder (prim quotient low 16777216 0 : Nat) 256 0 : Nat)', csize='1', low='((prim remainder low 16777216 0 : Nat) * 256)')}))) "
      f"else {upd('e', csize='(csize + 1)', low='((prim remainder low 16777216 0 : Nat) * 256)')}))))")
D('shiftLow', ['e : Enc'], 'Enc', sh)
D('norm', ['e : Enc'], 'Enc', f"(if ({P('e','rng')} <? 16777216) then (shiftLow {upd('e', rng='(%s * 256)' % P('e','rng'))}) else e)")
eb = (f"(let v : Nat := ({T}::get 2 {P('e','mem')} at); (let bound : Nat := ((prim quotient {P('e','rng')} 2048 0 : Nat) * v); "
      f"(if (bit =? 0) then (norm {upd('e', rng='bound', mem='(%s::set 2 %s at (v + (prim quotient (prim subtract 2048 v : Nat) 32 0 : Nat)))' % (T, P('e','mem')))}) "
      f"else (norm {upd('e', low='(%s + bound)' % P('e','low'), rng='(prim subtract %s bound : Nat)' % P('e','rng'), mem='(%s::set 2 %s at (prim subtract v (prim quotient v 32 0 : Nat) : Nat))' % (T, P('e','mem')))}))))")
D('encodeBit', ['e : Enc', 'at : Nat', 'bit : Nat'], 'Enc', eb)
D('bitOf', ['v : Nat', 'k : Nat'], 'Nat', f"(prim remainder (prim quotient v {pow2('k')} 0 : Nat) 2 0 : Nat)")
D('directEnc', ['fuel : Nat', 'e : Enc', 'v : Nat'], 'Enc',
  f"(match fuel\n    | Nat.zero => e\n    | Nat.succ k => (let half : Nat := (prim quotient {P('e','rng')} 2 0 : Nat); (directEnc k (norm (if ((bitOf v k) =? 1) then {upd('e', rng='half', low='(%s + half)' % P('e','low'))} else {upd('e', rng='half')})) v)))", 'fuel')
D('treeEnc', ['fuel : Nat', 'e : Enc', 'base : Nat', 'm : Nat', 'v : Nat'], 'Enc',
  "(match fuel\n    | Nat.zero => e\n    | Nat.succ k => (let b : Nat := (bitOf v k); (treeEnc k (encodeBit e (base + m) b) base ((2 * m) + b) v)))", 'fuel')
D('revEnc', ['fuel : Nat', 'e : Enc', 'base : Nat', 'm : Nat', 'v : Nat'], 'Enc',
  "(match fuel\n    | Nat.zero => e\n    | Nat.succ k => (let b : Nat := (prim remainder v 2 0 : Nat); (revEnc k (encodeBit e (base + m) b) base ((2 * m) + b) (prim quotient v 2 0 : Nat))))", 'fuel')
D('litMatched', ['fuel : Nat', 'e : Enc', 'base : Nat', 'value : Nat', 'matchByte : Nat', 'sym : Nat'], 'Enc',
  "(match fuel\n    | Nat.zero => e\n    | Nat.succ k => (let mbit : Nat := (bitOf matchByte k); (let b : Nat := (bitOf value k); (let next : Enc := (encodeBit e ((base + ((1 + mbit) * 256)) + sym) b); (if (b =? mbit) then (litMatched k next base value matchByte ((2 * sym) + b)) else (let rest : Enc := (treeEnc k next base ((2 * sym) + b) value); rest))))))", 'fuel')
D('lenEnc', ['e : Enc', 'base : Nat', 'v : Nat'], 'Enc',
  "(if (v <? 8) then (treeEnc 3 (encodeBit e base 0) (base + 2) 1 v) else (if (v <? 16) then (treeEnc 3 (encodeBit (encodeBit e base 1) (base + 1) 0) (base + 18) 1 (prim subtract v 8 : Nat)) else (treeEnc 8 (encodeBit (encodeBit e base 1) (base + 1) 1) (base + 34) 1 (prim subtract v 16 : Nat))))")
D('log2Loop', ['fuel : Nat', 'v : Nat', 'acc : Nat'], 'Nat', "(match fuel\n    | Nat.zero => acc\n    | Nat.succ k => (if (v <? 2) then acc else (log2Loop k (prim quotient v 2 0 : Nat) (acc + 1))))", 'fuel')
D('slotOf', ['dist : Nat'], 'Nat', "(if (dist <? 4) then dist else (let top : Nat := (log2Loop 64 dist 0); ((2 * top) + (bitOf dist (prim subtract top 1 : Nat)))))")
de = (f"(let ls : Nat := (if (lenCode <? 3) then lenCode else 3); (let slot : Nat := (slotOf dist); (let a : Enc := (treeEnc 6 e ({SLOT} + (ls * 64)) 1 slot); "
      f"(if (slot <? 4) then a else (let fb : Nat := (prim subtract (prim quotient slot 2 0 : Nat) 1 : Nat); (let base : Nat := ((2 + (prim remainder slot 2 0 : Nat)) * {pow2('fb')}); (let reduced : Nat := (prim subtract dist base : Nat); "
      f"(if (slot <? 14) then (revEnc fb a (prim subtract ({SPEC} + base) slot : Nat) 1 reduced) else (revEnc 4 (directEnc (prim subtract fb 4 : Nat) a (prim quotient reduced 16 0 : Nat)) {ALIGN} 1 (prim remainder reduced 16 0 : Nat))))))))))")
D('distEnc', ['e : Enc', 'dist : Nat', 'lenCode : Nat'], 'Enc', de)
# match finding reads the input and the tables without taking the state
D('hashAt', ['data : Bytes', 'i : Nat'], 'Nat', f"(let x : Nat := ((({B('data','i')} * 65536) + ({B('data','(i + 1)')} * 256)) + {B('data','(i + 2)')}); (prim remainder (prim quotient (x * 2654435761) 65536 0 : Nat) 65536 0 : Nat))")
D('hash4At', ['data : Bytes', 'i : Nat', 'hdiv : Nat', 'hsz : Nat'], 'Nat',
  f"(let x : Nat := ((({B('data','i')} + (256 * {B('data','(i + 1)')})) + (65536 * {B('data','(i + 2)')})) + (16777216 * {B('data','(i + 3)')})); (prim remainder (prim quotient (2654435761 * x) hdiv 0 : Nat) hsz 0 : Nat))")
ins = (f"(let n : Nat := {P('e','n')}; (let data : Bytes := {P('e','data')}; "
       f"(let f : Enc := (if ((i + 3) <? n) then (let k : Nat := (69632 + (hash4At data i {P('e','h4div')} {P('e','h4size')})); (let hv : Nat := ({T}::get 4 {P('e','mem')} k); "
       + upd('e', mem=f"({T}::set 4 ({T}::set 4 {P('e','mem')} ({P('e','chainAt')} + i) hv) k (i + 1))") + ")) else e); "
       f"(if ((i + 2) <? n) then {upd('f', mem=f'({T}::set 4 {P(chr(102),chr(109)+chr(101)+chr(109))} (4096 + (hashAt data i)) (i + 1))')} else f))))")
D('insert', ['e : Enc', 'i : Nat'], 'Enc', ins)
D('insertRange', ['fuel : Nat', 'e : Enc', 'i : Nat'], 'Enc', "(match fuel\n    | Nat.zero => e\n    | Nat.succ k => (insertRange k (insert e i) (i + 1)))", 'fuel')
D('matchLen', ['fuel : Nat', 'data : Bytes', 'a : Nat', 'b : Nat', 'l : Nat'], 'Nat', f"(match fuel\n    | Nat.zero => l\n    | Nat.succ k => (if ({B('data','(a + l)')} =? {B('data','(b + l)')}) then (matchLen k data a b (l + 1)) else l))", 'fuel')
fl = (f"(match fuel\n    | Nat.zero => (best, dist)\n    | Nat.succ k => (if (candidate =? 0) then (best, dist) else (let j : Nat := (prim subtract candidate 1 : Nat); (let next : Nat := ({T}::get 4 chain (cb + j)); "
      f"(if ({B('data','(j + best)')} =? {B('data','(i + best)')}) then (let l : Nat := (matchLen lim data j i 0); (let nb : Nat := (if (best <? l) then l else best); (let nd : Nat := (if (best <? l) then (prim subtract i (j + 1) : Nat) else dist); (if (64 <=? l) then (nb, nd) else (findLoop k data chain cb i lim next nb nd))))) "
      f"else (findLoop k data chain cb i lim next best dist))))))")
D('findLoop', ['fuel : Nat', 'data : Bytes', 'chain : Bytes', 'cb : Nat', 'i : Nat', 'lim : Nat', 'candidate : Nat', 'best : Nat', 'dist : Nat'], '(Nat * Nat)', fl, 'fuel')
D('limitAt', ['n : Nat', 'i : Nat'], 'Nat', "(let left : Nat := (prim subtract n i : Nat); (if (left <? 273) then left else 273))")
# the latest position with the same 3-byte hash gives the short matches; the 4-byte chain, the long ones
fd = (f"(let n : Nat := {P('e','n')}; (if (n <=? (i + 2)) then (0, 0) else (let data : Bytes := {P('e','data')}; (let mem : Bytes := {P('e','mem')}; (let lim : Nat := (limitAt n i); "
      f"(let c3 : Nat := ({T}::get 4 mem (4096 + (hashAt data i))); "
      f"(let s : (Nat * Nat) := (if (c3 =? 0) then (0, 0) else (let j : Nat := (prim subtract c3 1 : Nat); (let l : Nat := (matchLen lim data j i 0); (if (2 <=? l) then (l, (prim subtract i (j + 1) : Nat)) else (0, 0))))); "
      f"(if ((n <=? (i + 3)) || (64 <=? (first s))) then s else (let h : Nat := ({T}::get 4 mem (69632 + (hash4At data i {P('e','h4div')} {P('e','h4size')}))); (findLoop 48 data mem {P('e','chainAt')} i lim h (first s) (second s)))))))))))")
D('find', ['e : Enc', 'i : Nat'], '(Nat * Nat)', fd)
D('repLen', ['data : Bytes', 'i : Nat', 'lim : Nat', 'r : Nat'], 'Nat', "(if (i <? (r + 1)) then 0 else (matchLen lim data (prim subtract i (r + 1) : Nat) i 0))")
br = (f"(let data : Bytes := {P('e','data')}; (let l0 : Nat := (repLen data i lim {P('e','rep0')}); (let l1 : Nat := (repLen data i lim {P('e','rep1')}); (let l2 : Nat := (repLen data i lim {P('e','rep2')}); (let l3 : Nat := (repLen data i lim {P('e','rep3')}); "
      "(let a : (Nat * Nat) := (if (l0 <? l1) then (l1, 1) else (l0, 0)); (let b : (Nat * Nat) := (if ((first a) <? l2) then (l2, 2) else a); (if ((first b) <? l3) then (l3, 3) else b))))))))")
D('bestRep', ['e : Enc', 'i : Nat', 'lim : Nat'], '(Nat * Nat)', br)
D('afterLiteral', ['s : Nat'], 'Nat', f"({DEC}::afterLiteral s)")
rep = (f"(let s : Nat := {P('e','state')}; (let at0 : Nat := {P('e','at')}; (let a : Enc := (encodeBit (encodeBit e s 1) (12 + s) 1); "
       f"(let b : Enc := (if (ri =? 0) then (encodeBit (encodeBit a (24 + s) 0) (60 + s) 1) else (let g : Enc := (encodeBit a (24 + s) 1); (if (ri =? 1) then (encodeBit g (36 + s) 0) else (encodeBit (encodeBit g (36 + s) 1) (48 + s) (prim subtract ri 2 : Nat))))); "
       f"(let c : Enc := (lenEnc b {LENR} (prim subtract len 2 : Nat)); "
       f"(let r : Enc := (if (ri =? 0) then c else (if (ri =? 1) then {upd('c', rep0=P('c','rep1'), rep1=P('c','rep0'))} else (if (ri =? 2) then {upd('c', rep0=P('c','rep2'), rep1=P('c','rep0'), rep2=P('c','rep1'))} else {upd('c', rep0=P('c','rep3'), rep1=P('c','rep0'), rep2=P('c','rep1'), rep3=P('c','rep2'))}))); "
       f"(let f : Enc := (insertRange (prim subtract len 1 : Nat) r (at0 + 1)); {upd('f', state=f'({DEC}::afterRep s)', at='(at0 + len)')})))))))")
D('emitRep', ['e : Enc', 'ri : Nat', 'len : Nat'], 'Enc', rep)
mt = (f"(let s : Nat := {P('e','state')}; (let at0 : Nat := {P('e','at')}; (let a : Enc := (encodeBit (encodeBit e s 1) (12 + s) 0); (let b : Enc := (distEnc (lenEnc a {LENL} (prim subtract len 2 : Nat)) dist (prim subtract len 2 : Nat)); "
      f"(let f : Enc := (insertRange (prim subtract len 1 : Nat) b (at0 + 1)); {upd('f', state=f'({DEC}::afterMatch s)', at='(at0 + len)', rep0='dist', rep1=P('f','rep0'), rep2=P('f','rep1'), rep3=P('f','rep2'))})))))")
D('emitMatch', ['e : Enc', 'len : Nat', 'dist : Nat'], 'Enc', mt)
lt = (f"(let s : Nat := {P('e','state')}; (let i : Nat := {P('e','at')}; (let data : Bytes := {P('e','data')}; (let r0 : Nat := {P('e','rep0')}; "
      f"(let prev : Nat := (if (i =? 0) then 0 else {B('data','(prim subtract i 1 : Nat)')}); (let base : Nat := ({LIT} + (768 * (prim quotient prev 32 0 : Nat))); "
      f"(let cur : Nat := {B('data','i')}; (let mb : Nat := (if (i <? (r0 + 1)) then 0 else {B('data','(prim subtract i (r0 + 1) : Nat)')}); "
      f"(let a : Enc := (encodeBit e s 0); (let b : Enc := (if (s <? 7) then (treeEnc 8 a base 1 cur) else (litMatched 8 a base cur mb 1)); "
      f"{upd('b', state='(afterLiteral s)', at='(i + 1)')}))))))))))")
D('emitLiteral', ['e : Enc'], 'Enc', lt)
stp = (f"(let i : Nat := {P('e','at')}; (let n : Nat := {P('e','n')}; (let lim : Nat := (limitAt n i); (let r : (Nat * Nat) := (bestRep e i lim); (let rl : Nat := (first r); (let ri : Nat := (second r); "
       "(let f : (Nat * Nat) := (find e i); (let fl : Nat := (first f); (let fd : Nat := (second f); (let ml1 : Nat := (if ((fl =? 2) && (128 <=? fd)) then 0 else fl); (let e1 : Enc := (insert e i); "
       "(let ml : Nat := (if (((2 <=? ml1) && ((rl + 1) <? ml1)) && ((i + 1) <? n)) then (let g : (Nat * Nat) := (find e1 (i + 1)); (let gl : Nat := (first g); (let gd : Nat := (second g); (if (((ml1 + 1) <? gl) || ((gl =? (ml1 + 1)) && (gd <? (4 * fd)))) then 0 else ml1)))) else ml1); "
       "(if ((2 <=? rl) && (ml <=? (rl + 1))) then (emitRep e1 ri rl) else (if (2 <=? ml) then (emitMatch e1 ml fd) else (emitLiteral e1)))))))))))))))")
D('step', ['e : Enc'], 'Enc', stp)
D('encodeLoop', ['fuel : Nat', 'e : Enc'], 'Enc', f"(match fuel\n    | Nat.zero => e\n    | Nat.succ k => (let i : Nat := {P('e','at')}; (let n : Nat := {P('e','n')}; (if (n <=? i) then e else (encodeLoop k (step e))))))", 'fuel')
D('header', ['fuel : Nat', 'v : Nat'], 'Bytes', f"(match fuel\n    | Nat.zero => 0x\n    | Nat.succ k => (prim append ({DEC}::singleton (prim remainder v 256 0 : Nat)) (header k (prim quotient v 256 0 : Nat)) : Bytes))", 'fuel')
start = (f"(let n : Nat := (prim length data : Nat); (let od : Nat := ({DEC}::depthFor 64 ((2 * n) + 64) 0); (let cd : Nat := ({DEC}::depthFor 64 (n + 1) 0); "
         f"(let hb : Nat := (if (cd <? 16) then 16 else (if (22 <? cd) then 22 else cd)); (let h4size : Nat := {pow2('hb')}; "
         + L.make({'mem': f"(prim append (prim append (prim append (prim append ({T}::fill 2 13 1024) ({T}::fill 4 16 0) : Bytes) ({T}::fill 4 hb 0) : Bytes) ({T}::fill 4 cd 0) : Bytes) ({T}::fill 1 od 0) : Bytes)"},
                  dict(data='data', n='n', low='0', rng='4294967295', cache='0', csize='1', outSize='0', state='0', rep0='0', rep1='0', rep2='0', rep3='0', at='0',
                       outBase=f'((278528 + (4 * h4size)) + (4 * {pow2(chr(99)+chr(100))}))', h4div=pow2('(prim subtract 32 hb : Nat)'), h4size='h4size', chainAt='(69632 + h4size)')) + ")))))")
D('start', ['data : Bytes'], 'Enc', start)
# encode `input` against the window `history`: the match finder starts holding every history position,
# the coder starts at the first input byte, and only the input is emitted
D('startWith', ['history : Bytes', 'input : Bytes'], 'Enc',
  f"(let e : Enc := (start (prim append history input : Bytes)); (let h : Nat := (prim length history : Nat); (insertRange h {upd('e', at='h')} 0)))")
encW = (f"(let n : Nat := (prim length input : Nat); (let e : Enc := (startWith history input); "
        f"(let f : Enc := (shiftLow (shiftLow (shiftLow (shiftLow (shiftLow (encodeLoop (n + 1) e)))))); (let size : Nat := {P('f','outSize')}; "
        f"(prim append (header 8 n) (match (prim slice {P('f','mem')} {P('f','outBase')} size : (Option Bytes))\n    | Option.none => 0x\n    | Option.some b => b) : Bytes)))))")
D('encodeWith', ['history : Bytes', 'input : Bytes'], 'Bytes', encW)
D('encode', ['data : Bytes'], 'Bytes', "(encodeWith 0x data)")
src = ("\\begin{lexlean}{Uorc.Codec.LzEncode}\n\\importmodule{Uorc.Codec.Table}\n\\importmodule{Uorc.Codec.LzDecode}\n\\title{Parameters}\n\\begin{semanticmodule}\n"
       + L.structure() + "\n" + "\n".join(defs) + "\\end{semanticmodule}\n\\end{lexlean}\n")
open(os.path.join(HERE, '..', 'src', 'Uorc', 'Codec', 'LzEncode.lex.tex'), 'w').write(src)
