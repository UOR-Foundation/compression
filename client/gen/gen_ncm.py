# Generates Uorc/Codec/NcmDecode.lex.tex and NcmEncode.lex.tex from ncm_model (writing aid).
import sys
import os
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import irlex, ncm_model
from state import definer

TB = 'Uorc.Codec.Table'
OUTDIR = os.path.join(HERE, '..', 'src', 'Uorc', 'Codec')


def sizing(D):
    D('depthFor', ['fuel : Nat', 'n : Nat', 'depth : Nat'], 'Nat',
      "(match fuel\n    | Nat.zero => depth\n    | Nat.succ m => (if (n <=? (pow2 depth)) then depth else (depthFor m n (depth + 1))))", 'fuel')
    D('tbLoop', ['fuel : Nat', 'n : Nat', 'tb : Nat'], 'Nat',
      "(match fuel\n    | Nat.zero => tb\n    | Nat.succ m => (if ((pow2 tb) <? (prim multiply 1024 n : Nat)) then (tbLoop m n (tb + 1)) else tb))", 'fuel')
    D('tbFor', ['n : Nat'], 'Nat', "(tbLoop 8 n 16)")
    D('mbFor', ['tb : Nat'], 'Nat', "(if (22 <? tb) then 22 else (prim subtract tb 2 : Nat))")
    # the context table's bits: the ones the profile word records in bits 20-25 (TABLE_SHIFT), else the length's own;
    # the match tables' bits: the ones it records in bits 26-30 (MATCH_SHIFT), else the default context table's
    D('tbOf', ['n : Nat', 'profile : Nat'], 'Nat',
      f"(let t : Nat := (prim remainder (prim quotient profile {ncm_model.TABLE_UNIT} 0 : Nat) 64 0 : Nat); (if (0 <? t) then t else (tbFor n)))")
    D('mbOf', ['n : Nat', 'profile : Nat'], 'Nat',
      f"(let m : Nat := (prim remainder (prim quotient profile {ncm_model.MATCH_UNIT} 0 : Nat) 32 0 : Nat); (if (0 <? m) then m else (mbFor (tbFor n))))")
    D('zeros', ['depth : Nat'], 'Bytes', f"({TB}::fill 1 depth 0)")


def tables(L, n, extra, history='0x', profile='profile'):
    # Every table's depth names n: the certificate kernel evaluates whatever is closed, and a closed state
    # would have it run the model's whole setup (thousands of table writes) at check time.
    t = {'glob': f'(zeros (8 + (prim multiply 0 {n} : Nat)))', 'mem': f'(zeros (22 + (prim multiply 0 {n} : Nat)))', 'T': f'({TB}::fill 3 (tbOf {n} {profile}) 0)', 'MT': f'({TB}::fill 4 (mbOf {n} {profile}) 0)',
         'H': f'(prim append {history} (zeros (depthFor 40 {n} 0)) : Bytes)', 'A2': f'({TB}::fill 2 (22 + (prim multiply 0 {n} : Nat)) 0)',
         # the short match table, and the short match and repeat models' 64 and 128 slots
         'M2T': f'({TB}::fill 4 (mbOf {n} {profile}) 0)', 'M2S': f'({TB}::fill 3 (6 + (prim multiply 0 {n} : Nat)) 0)',
         'RS': f'({TB}::fill 3 (7 + (prim multiply 0 {n} : Nat)) 0)'}
    t.update(extra)
    return t


def scalars(L, inp):
    return {s.split(':')[0]: (inp if s.startswith('input:') else '0') for s in L.scalars}


# decoder
BIG = ['T', 'H', 'MT', 'A2', 'M2T', 'M2S', 'RS']
V03 = str(ncm_model.V03)
dec = irlex.Lex(ncm_model.P, 'NcmDecode', ['decodeProfile', 'decodeWarm'], BIG)
sizing(dec.D)
n = ("(let n : Nat := (((prim multiply 16777216 (getAt 1 input 0) : Nat) + (prim multiply 65536 (getAt 1 input 1) : Nat)) + "
     "((prim multiply 256 (getAt 1 input 2) : Nat) + (getAt 1 input 3))); ")
# against a window: H starts holding the history (h bytes), and only the n coded bytes after it are returned
body = (n + "(let h : Nat := (prim length history : Nat); "
        f"(let s0 : {dec.St} := {dec.L.make(tables(dec.L, 'n', {}, 'history'), scalars(dec.L, 'input'))}; "
        f"(let s1 : {dec.St} := (decodeProfile s0 h profile); (match (prim slice {dec.L.P('s1', 'H')} h n : (Option Bytes))\n    | Option.none => 0x\n    | Option.some b => b)))))")
# the decode under a profile word (ncm_model: bits 0-16 contexts, then the match features); V03 is the v0.3 coder
dec.D('runProfile', ['history : Bytes', 'input : Bytes', 'profile : Nat'], 'Bytes', f"(if ((prim length input : Nat) <? 4) then 0x else {body})")
dec.D('runWith', ['history : Bytes', 'input : Bytes'], 'Bytes', f"(runProfile history input {V03})")
dec.D('run', ['input : Bytes'], 'Bytes', "(runWith 0x input)")
# from a kappa-named model state: H starts holding the state (w bytes), which the model codes first; the tables are
# sized for w + n bytes and only the n bytes after the state are returned
wbody = (n + "(let w : Nat := (prim length state : Nat); "
         f"(let s0 : {dec.St} := {dec.L.make(tables(dec.L, '(w + n)', {}, 'state'), scalars(dec.L, 'input'))}; "
         f"(let s1 : {dec.St} := (decodeWarm s0 w profile); (match (prim slice {dec.L.P('s1', 'H')} w n : (Option Bytes))\n    | Option.none => 0x\n    | Option.some b => b)))))")
dec.D('runWarm', ['state : Bytes', 'input : Bytes', 'profile : Nat'], 'Bytes', f"(if ((prim length input : Nat) <? 4) then 0x else {wbody})")
open(os.path.join(OUTDIR, 'NcmDecode.lex.tex'), 'w').write(dec.source([TB]))

# encoder
enc = irlex.Lex(ncm_model.P, 'NcmEncode', ['encodeProfile', 'encodeLearn', 'encodeWarm'], BIG + ['OUT'])
sizing(enc.D)
enc.D('start', ['history : Bytes', 'input : Bytes', 'n : Nat', 'cap : Nat', 'profile : Nat'], enc.St,
      enc.L.make(tables(enc.L, 'n', {'OUT': '(zeros (depthFor 40 cap 0))'}, 'history'), scalars(enc.L, 'input')))
enc.D('finish', ['s1 : ' + enc.St, 'n : Nat'], 'Bytes',
      f"(if (0 <? {enc.gread('s1', 'ovf')}) then 0x else (let head : Bytes := (putAt 4 (zeros 2) 0 n); "
      f"(match (prim slice {enc.L.P('s1', 'OUT')} 0 {enc.gread('s1', 'op')} : (Option Bytes))\n    | Option.none => 0x\n"
      f"    | Option.some b => (prim append head b : Bytes))))")
body = ("(let n : Nat := (prim length input : Nat); (let cap : Nat := ((prim multiply 2 n : Nat) + 64); "
        "(finish (encodeProfile (start history input n cap profile) n cap (prim length history : Nat) profile) n)))")
# encode `input` against the window `history` under a profile word: the match model starts holding every
# history position
enc.D('runProfile', ['history : Bytes', 'input : Bytes', 'profile : Nat'], 'Bytes', f"(if ((prim length input : Nat) <? 4294967296) then {body} else 0x)")
enc.D('runWith', ['history : Bytes', 'input : Bytes'], 'Bytes', f"(runProfile history input {V03})")
# the profile the encoder learns coding `input` under `profile`: without the contexts that saved nothing
learn = ("(let n : Nat := (prim length input : Nat); (let cap : Nat := ((prim multiply 2 n : Nat) + 64); "
         f"(let s1 : {enc.St} := (encodeLearn (start history input n cap profile) n cap (prim length history : Nat) profile); "
         f"(if (0 <? {enc.gread('s1', 'ovf')}) then profile else {enc.gread('s1', 'lmask')}))))")
enc.D('learnProfile', ['history : Bytes', 'input : Bytes', 'profile : Nat'], 'Nat', f"(if ((prim length input : Nat) <? 4294967296) then {learn} else profile)")
# encode `input` as the continuation of the kappa-named model state `state`, which the whole model codes first
wenc = ("(let n : Nat := (prim length input : Nat); (let cap : Nat := ((prim multiply 2 n : Nat) + 64); (let w : Nat := (prim length state : Nat); "
        "(finish (encodeWarm (start state input (w + n) cap profile) n cap w profile) n))))")
enc.D('runWarm', ['state : Bytes', 'input : Bytes', 'profile : Nat'], 'Bytes', f"(if ((prim length input : Nat) <? 4294967296) then {wenc} else 0x)")
enc.D('run', ['input : Bytes'], 'Bytes', "(runWith 0x input)")
open(os.path.join(OUTDIR, 'NcmEncode.lex.tex'), 'w').write(enc.source([TB]))
print('ok')
