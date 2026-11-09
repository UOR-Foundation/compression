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
    D('zeros', ['depth : Nat'], 'Bytes', f"({TB}::fill 1 depth 0)")


def tables(L, n, extra):
    # the mem depth names n so the certificate kernel does not build the 2 MiB table as a closed term
    t = {'mem': f'(zeros (21 + (prim multiply 0 {n} : Nat)))', 'T': f'({TB}::fill 3 (tbFor {n}) 0)', 'MT': f'({TB}::fill 4 (mbFor (tbFor {n})) 0)',
         'H': f'(zeros (depthFor 40 {n} 0))'}
    t.update(extra)
    return t


def scalars(L, inp):
    return {s.split(':')[0]: (inp if s.startswith('input:') else '0') for s in L.scalars}


# decoder
dec = irlex.Lex(ncm_model.P, 'NcmDecode', ['decode'], ['T', 'H', 'MT'])
sizing(dec.D)
n = ("(let n : Nat := (((prim multiply 16777216 (getAt 1 input 0) : Nat) + (prim multiply 65536 (getAt 1 input 1) : Nat)) + "
     "((prim multiply 256 (getAt 1 input 2) : Nat) + (getAt 1 input 3))); ")
body = (n + f"(let s0 : {dec.St} := {dec.L.make(tables(dec.L, 'n', {}), scalars(dec.L, 'input'))}; "
        f"(let s1 : {dec.St} := (decode s0); (match (prim slice {dec.L.P('s1', 'H')} 0 n : (Option Bytes))\n    | Option.none => 0x\n    | Option.some b => b))))")
dec.D('run', ['input : Bytes'], 'Bytes', f"(if ((prim length input : Nat) <? 4) then 0x else {body})")
open(os.path.join(OUTDIR, 'NcmDecode.lex.tex'), 'w').write(dec.source([TB]))

# encoder
enc = irlex.Lex(ncm_model.P, 'NcmEncode', ['encode'], ['T', 'H', 'MT', 'OUT'])
sizing(enc.D)
cap = "(prim multiply 2 n : Nat) + 64"
body = ("(let n : Nat := (prim length input : Nat); (let cap : Nat := (" + cap + "); "
        f"(let s0 : {enc.St} := {enc.L.make(tables(enc.L, 'n', {'OUT': '(zeros (depthFor 40 cap 0))'}), scalars(enc.L, 'input'))}; "
        f"(let s1 : {enc.St} := (encode s0 n cap); "
        f"(if (0 <? {enc.L.P('s1', 'ovf')}) then 0x else (let head : Bytes := (putAt 4 (zeros 2) 0 n); "
        f"(match (prim slice {enc.L.P('s1', 'OUT')} 0 {enc.L.P('s1', 'op')} : (Option Bytes))\n    | Option.none => 0x\n"
        f"    | Option.some b => (prim append head b : Bytes))))))))")
enc.D('run', ['input : Bytes'], 'Bytes', f"(if ((prim length input : Nat) <? 4294967296) then {body} else 0x)")
open(os.path.join(OUTDIR, 'NcmEncode.lex.tex'), 'w').write(enc.source([TB]))
print('ok')
