# Generates Uorc/Codec/Table.lex.tex: fixed-width unsigned entries in a byte string, written in place.
import sys
import os
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from state import definer
terms = [f"(if ((prim remainder (prim quotient r {1<<k} 0 : Nat) 2 0 : Nat) =? 1) then ({1<<k} : UInt8) else (0 : UInt8))" for k in range(8)]
e = terms[0]
for t in terms[1:]:
    e = f"(prim bit_or {e} {t} : UInt8)"
defs = []
D = definer(defs, 'Table')
D('byteOf', ['v : Nat'], 'UInt8', f"(let r : Nat := (prim remainder v 256 0 : Nat); {e})")
D('readByte', ['t : Bytes', 'at : Nat'], 'Nat', "(match (prim index t at : (Option UInt8))\n    | Option.none => 0\n    | Option.some b => (prim to_nat b : Nat))")
D('writeByte', ['t : Bytes', 'at : Nat', 'v : Nat'], 'Bytes', "(match (prim update t at (byteOf v) : (Option Bytes))\n    | Option.none => 0x\n    | Option.some u => u)")
D('getLoop', ['fuel : Nat', 't : Bytes', 'at : Nat', 'acc : Nat'], 'Nat', "(match fuel\n    | Nat.zero => acc\n    | Nat.succ k => (let b : Nat := (readByte t at); (getLoop k t (at + 1) ((256 * acc) + b))))", 'fuel')
D('get', ['width : Nat', 't : Bytes', 'key : Nat'], 'Nat', "(getLoop width t (width * key) 0)")
D('setLoop', ['fuel : Nat', 't : Bytes', 'at : Nat', 'v : Nat'], 'Bytes', "(match fuel\n    | Nat.zero => t\n    | Nat.succ k => (setLoop k (writeByte t at (prim remainder v 256 0 : Nat)) (prim subtract at 1 : Nat) (prim quotient v 256 0 : Nat)))", 'fuel')
D('set', ['width : Nat', 't : Bytes', 'key : Nat', 'v : Nat'], 'Bytes', "(setLoop width t (prim subtract ((width * key) + width) 1 : Nat) v)")
D('double', ['fuel : Nat', 'b : Bytes'], 'Bytes', "(match fuel\n    | Nat.zero => b\n    | Nat.succ k => (double k (prim append b b : Bytes)))", 'fuel')
D('zeros', ['width : Nat'], 'Bytes', "(match (prim slice 0x00000000 0 width : (Option Bytes))\n    | Option.none => 0x\n    | Option.some z => z)")
D('fill', ['width : Nat', 'depth : Nat', 'value : Nat'], 'Bytes', "(double depth (set width (zeros width) 0 value))")
src = "\\begin{lexlean}{Uorc.Codec.Table}\n\\title{Parameters}\n\\begin{semanticmodule}\n" + "\n".join(defs) + "\\end{semanticmodule}\n\\end{lexlean}\n"
open(os.path.join(HERE, '..', 'src', 'Uorc', 'Codec', 'Table.lex.tex'), 'w').write(src)
