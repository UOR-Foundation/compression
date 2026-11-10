# Generates Uorc/Codec/X86.lex.tex from x86_model (a writing aid; the .lex.tex is the source).
import sys
import os
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import irlex, x86_model

TB = 'Uorc.Codec.Table'
OUTDIR = os.path.join(HERE, '..', 'src', 'Uorc', 'Codec')

x = irlex.Lex(x86_model.P, 'X86', ['filter'], ['OUT'])
D = x.D
D('depthFor', ['fuel : Nat', 'n : Nat', 'depth : Nat'], 'Nat',
  "(match fuel\n    | Nat.zero => depth\n    | Nat.succ m => (if (n <=? (pow2 depth)) then depth else (depthFor m n (depth + 1))))", 'fuel')
D('zeros', ['depth : Nat'], 'Bytes', f"({TB}::fill 1 depth 0)")
# Every table's depth names n: the certificate kernel evaluates whatever is closed.
tables = {'glob': '(zeros (8 + (prim multiply 0 n : Nat)))', 'mem': '(zeros (prim multiply 0 n : Nat))',
          'OUT': '(zeros (depthFor 40 n 0))'}
D('start', ['input : Bytes', 'n : Nat'], x.St, x.L.make(tables, {'input': 'input'}))
D('state', ['d : Nat', 'base : Nat', 'input : Bytes'], x.St,
  "(let n : Nat := (prim length input : Nat); (filter (start input n) n d base))")
# the filter (d = 0) or its inverse (d = 1) of `input` placed at `base` in the stream
D('run', ['d : Nat', 'base : Nat', 'input : Bytes'], 'Bytes',
  f"(let s1 : {x.St} := (state d base input); (match (prim slice {x.L.P('s1', 'OUT')} 0 (prim length input : Nat) : (Option Bytes))\n    | Option.none => 0x\n"
  "    | Option.some b => b))")
# the displacements the filter rewrites in `input`
D('hits', ['input : Bytes'], 'Nat', f"(let s1 : {x.St} := (state 0 0 input); {x.gread('s1', 'hits')})")
open(os.path.join(OUTDIR, 'X86.lex.tex'), 'w').write(x.source([TB]))
print('ok')
