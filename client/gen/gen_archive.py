# Generates Uorc/Codec/Archive.lex.tex: UORC archives whose program is the ulz decoder kernel (writing aid).
# The kernel's wire encoding is GraphEmit's encoding of the graph that graph/ulzgraph.py builds; only the
# payload's length and bytes vary, so the archive is kernelPrefix ++ uleb(length) ++ payload ++ kernelSuffix.
import sys
import os
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from state import definer
import ulzgraph, wire

G, e = ulzgraph.build(b'\xaa\xbb')
body = wire.graph_bytes(G, e)
cut = body.index(bytes([3, 2, 0xaa, 0xbb]))
assert body.count(bytes([3, 2, 0xaa, 0xbb])) == 1
PREFIX, SUFFIX = body[:cut + 1], body[cut + 4:]
DOM = wire.DOM.hex()
DEC, ENC, SHA = 'Uorc.Codec.LzDecode', 'Uorc.Codec.LzEncode', 'Uorc.Integrity.Sha256'
defs = []
D = definer(defs, 'Archive')
D('kernelPrefix', [], 'Bytes', '0x' + PREFIX.hex())
D('kernelSuffix', [], 'Bytes', '0x' + SUFFIX.hex())
D('cat', ['a : Bytes', 'b : Bytes'], 'Bytes', '(prim append a b : Bytes)')
D('uleb', ['fuel : Nat', 'n : Nat'], 'Bytes',
  f"(match fuel\n    | Nat.zero => 0x\n    | Nat.succ k => (if (n <? 128) then ({DEC}::singleton n) else (cat ({DEC}::singleton ((prim remainder n 128 0 : Nat) + 128)) (uleb k (prim quotient n 128 0 : Nat)))))", 'fuel')
D('number', ['n : Nat'], 'Bytes', '(uleb 10 n)')
# the frame preimage: magic, format 2, profile 1, self-contained, body kind, one object, its length, the body
D('preimage', ['kind : Bytes', 'n : Nat', 'body : Bytes'], 'Bytes',
  "(cat (cat (cat (cat (cat 0x554f5243020100 kind) (number 1)) (number n)) (number (prim length body : Nat))) body)")
# UORC-SHA256-CHAIN-v1 over the preimage: one frame per full 64 bytes, then the terminal frame
D('chainLoop', ['fuel : Nat', 'data : Bytes', 'at : Nat', 'c : Bytes'], '(Option Bytes)',
  f"(match fuel\n    | Nat.zero => (ctor Option.none [Bytes])\n    | Nat.succ k => (let rest : Nat := (prim subtract (prim length data : Nat) at : Nat); "
  f"(if (64 <=? rest) then (match (prim slice data at 64 : (Option Bytes))\n      | Option.none => (ctor Option.none [Bytes])\n      | Option.some chunk => (match ({SHA}::hash (cat (cat (cat 0x{DOM} 0x01) c) chunk))\n        | Result.error failure => (ctor Option.none [Bytes])\n        | Result.ok h => (chainLoop k data (at + 64) h))) "
  f"else (match (prim slice data at rest : (Option Bytes))\n      | Option.none => (ctor Option.none [Bytes])\n      | Option.some tail => (match ({SHA}::hash (cat (cat (cat (cat 0x{DOM} 0x02) c) ({DEC}::singleton rest)) tail))\n        | Result.error failure => (ctor Option.none [Bytes])\n        | Result.ok h => (ctor Option.some [Bytes] h))))))", 'fuel')
D('seal', ['pre : Bytes'], '(Option Bytes)',
  f"(match ({SHA}::hash (cat 0x{DOM} 0x00))\n    | Result.error failure => (ctor Option.none [Bytes])\n    | Result.ok c => (match (chainLoop ((prim quotient (prim length pre : Nat) 64 0 : Nat) + 1) pre 0 c)\n      | Option.none => (ctor Option.none [Bytes])\n      | Option.some d => (ctor Option.some [Bytes] (cat pre d))))")
D('programArchive', ['n : Nat', 'payload : Bytes'], '(Option Bytes)',
  "(seal (preimage 0x01 n (cat (cat (cat (kernelPrefix) (number (prim length payload : Nat))) payload) (kernelSuffix))))")
D('rawArchive', ['input : Bytes'], '(Option Bytes)', "(seal (preimage 0x00 (prim length input : Nat) input))")
D('archive', ['input : Bytes'], '(Option Bytes)', f"(programArchive (prim length input : Nat) ({ENC}::encode input))")
D('readUleb', ['fuel : Nat', 'data : Bytes', 'at : Nat', 'weight : Nat', 'acc : Nat'], '(Option (Nat * Nat))',
  f"(match fuel\n    | Nat.zero => (ctor Option.none [(Nat * Nat)])\n    | Nat.succ k => (match (prim index data at : (Option UInt8))\n      | Option.none => (ctor Option.none [(Nat * Nat)])\n      | Option.some octet => (let b : Nat := (prim to_nat octet : Nat); "
  f"(if (b <? 128) then (ctor Option.some [(Nat * Nat)] ((acc + (weight * b)), (at + 1))) else (readUleb k data (at + 1) (128 * weight) (acc + (weight * (prim subtract b 128 : Nat))))))))", 'fuel')
# The payload of an archive whose program is the ulz kernel; the caller rebuilds the archive to confirm it.
D('payloadAt', ['data : Bytes', 'at : Nat'], '(Option Bytes)',
  f"(match (readUleb 10 data at 1 0)\n    | Option.none => (ctor Option.none [Bytes])\n    | Option.some r => (prim slice data (second r) (first r) : (Option Bytes)))")
D('lengthAt', ['data : Bytes'], '(Option (Nat * Nat))',
  "(match (readUleb 10 data 9 1 0)\n    | Option.none => (ctor Option.none [(Nat * Nat)])\n    | Option.some n => (match (readUleb 10 data (second n) 1 0)\n      | Option.none => (ctor Option.none [(Nat * Nat)])\n      | Option.some b => (ctor Option.some [(Nat * Nat)] ((first n), ((second b) + (prim length (kernelPrefix) : Nat))))))")
D('kernelPayload', ['data : Bytes'], '(Option (Nat * Bytes))',
  "(match (lengthAt data)\n    | Option.none => (ctor Option.none [(Nat * Bytes)])\n    | Option.some np => (match (payloadAt data (second np))\n      | Option.none => (ctor Option.none [(Nat * Bytes)])\n      | Option.some payload => (match (programArchive (first np) payload)\n        | Option.none => (ctor Option.none [(Nat * Bytes)])\n        | Option.some rebuilt => (if (prim equal rebuilt data : Bool) then (ctor Option.some [(Nat * Bytes)] ((first np), payload)) else (ctor Option.none [(Nat * Bytes)])))))")
# The kernel graph's meaning on a payload the encoder wrote is what the certified decoder computes; a payload
# the encoder would not write (re-encoding differs) is left to the reference path.
D('fastDecode', ['data : Bytes'], '(Option Bytes)',
  f"(match (kernelPayload data)\n    | Option.none => (ctor Option.none [Bytes])\n    | Option.some np => (let payload : Bytes := (second np); (let out : Bytes := ({DEC}::decode payload); "
  f"(if (((prim length out : Nat) =? (first np)) && (prim equal ({ENC}::encode out) payload : Bool)) then (ctor Option.some [Bytes] out) else (ctor Option.none [Bytes])))))")
src = ("\\begin{lexlean}{Uorc.Codec.Archive}\n\\importmodule{Uorc.Codec.LzDecode}\n\\importmodule{Uorc.Codec.LzEncode}\n\\importmodule{Uorc.Integrity.Sha256}\n\\title{Parameters}\n\\begin{semanticmodule}\n"
       + "\n".join(defs) + "\\end{semanticmodule}\n\\end{lexlean}\n")
open(os.path.join(HERE, '..', 'src', 'Uorc', 'Codec', 'Archive.lex.tex'), 'w').write(src)
print('prefix', len(PREFIX), 'suffix', len(SUFFIX))
