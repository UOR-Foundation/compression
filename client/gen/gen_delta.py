# Generates Uorc/Codec/Delta.lex.tex from delta_model (a writing aid; the .lex.tex is the source).
import sys
import os
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import irlex, delta_model

TB = 'Uorc.Codec.Table'
OUTDIR = os.path.join(HERE, '..', 'src', 'Uorc', 'Codec')

x = irlex.Lex(delta_model.P, 'Delta', ['filter', 'measure'], ['OUT'])
D = x.D
D('depthFor', ['fuel : Nat', 'n : Nat', 'depth : Nat'], 'Nat',
  "(match fuel\n    | Nat.zero => depth\n    | Nat.succ m => (if (n <=? (pow2 depth)) then depth else (depthFor m n (depth + 1))))", 'fuel')
D('zeros', ['depth : Nat'], 'Bytes', f"({TB}::fill 1 depth 0)")
# Every table's depth names n: the certificate kernel evaluates whatever is closed. The output table's
# depth search runs for as many rounds as the input has bytes (a depth past them is never needed).
tables = {'glob': '(zeros (8 + (prim multiply 0 n : Nat)))', 'mem': '(zeros (prim multiply 0 n : Nat))',
          'OUT': '(zeros (depthFor (n + 1) n 0))'}
D('start', ['input : Bytes', 'n : Nat'], x.St, x.L.make(tables, {'input': 'input'}))
# the filter (d = 0) or its inverse (d = 1) of `input`, samples `w` bytes wide in `c` channels
D('run', ['d : Nat', 'w : Nat', 'c : Nat', 'input : Bytes'], 'Bytes',
  f"(let n : Nat := (prim length input : Nat); (let s1 : {x.St} := (filter (start input n) n d w c); (match (prim slice {x.L.P('s1', 'OUT')} 0 n : (Option Bytes))\n    | Option.none => 0x\n"
  "    | Option.some b => b)))")
# the magnitude bits of `input`'s samples (raw 1) or of their differences (raw 0)
D('bits', ['w : Nat', 'c : Nat', 'raw : Nat', 'input : Bytes'], 'Nat',
  f"(let n : Nat := (prim length input : Nat); (let s1 : {x.St} := (measure (start input n) n w c raw); {x.gread('s1', 'bits')}))")
open(os.path.join(OUTDIR, 'Delta.lex.tex'), 'w').write(x.source([TB]))
print('ok')
