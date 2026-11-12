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
# A raw archive's body is its one object; the rebuilt archive confirms the frame and the trailer.
D('rawPayload', ['data : Bytes'], '(Option Bytes)',
  "(match (readUleb 10 data 9 1 0)\n    | Option.none => (ctor Option.none [Bytes])\n    | Option.some n => (match (readUleb 10 data (second n) 1 0)\n      | Option.none => (ctor Option.none [Bytes])\n      | Option.some b => (match (prim slice data (second b) (first n) : (Option Bytes))\n        | Option.none => (ctor Option.none [Bytes])\n        | Option.some payload => (match (rawArchive payload)\n          | Option.none => (ctor Option.none [Bytes])\n          | Option.some rebuilt => (if (prim equal rebuilt data : Bool) then (ctor Option.some [Bytes] payload) else (ctor Option.none [Bytes]))))))")
# The kernel graph's meaning on a payload the encoder wrote is what the certified decoder computes; a payload
# the encoder would not write (re-encoding differs) is left to the reference path.
D('kernelDecode', ['data : Bytes'], '(Option Bytes)',
  f"(match (kernelPayload data)\n    | Option.none => (ctor Option.none [Bytes])\n    | Option.some np => (let payload : Bytes := (second np); (let out : Bytes := ({DEC}::decode payload); "
  f"(if (((prim length out : Nat) =? (first np)) && (prim equal ({ENC}::encode out) payload : Bool)) then (ctor Option.some [Bytes] out) else (ctor Option.none [Bytes])))))")

# ---- the standard kernel basis (gen_basis.py): linked archives name the ulz + ncm libraries by identity ----
import gen_basis as GB
import wire
NDEC, NENC = 'Uorc.Codec.NcmDecode', 'Uorc.Codec.NcmEncode'
NONE_B = "(ctor Option.none [Bytes])"
LPRE = GB.TEMPLATES['ulz'][0]
assert GB.TEMPLATES['ncm'][0] == LPRE
D('basisIdentity', [], 'Bytes', '0x' + GB.STD_ID.hex())
D('basisLibrary', [], 'Bytes', '0x' + GB.LIBB.hex())
D('libraryBlocks', [], 'Nat', str(GB.L))
# Library blocks are only ever appended, so an earlier release's library is a prefix of this one: each era
# names its library as that prefix rather than carrying its own copy.
D('libraryPrefix', ['n : Nat'], 'Bytes', "(match (prim slice (basisLibrary) 0 n : (Option Bytes))\n    | Option.none => (basisLibrary)\n    | Option.some prefix => prefix)")
D('linkedPrefix', [], 'Bytes', '0x' + LPRE.hex())
D('ulzSuffix', [], 'Bytes', '0x' + GB.TEMPLATES['ulz'][1].hex())
D('ncmSuffix', [], 'Bytes', '0x' + GB.TEMPLATES['ncm'][1].hex())
# the x86-filtered ncm entry (gen_basis CODECS ending in x): its prefix has the same length
XPRE = GB.TEMPLATES['ncmx'][0]
assert len(XPRE) == len(LPRE)
D('x86Prefix', [], 'Bytes', '0x' + XPRE.hex())
D('ncmxSuffix', [], 'Bytes', '0x' + GB.TEMPLATES['ncmx'][1].hex())
# a conditional archive (scope 1, program body) over the standard basis
D('linkedPreimage', ['n : Nat', 'body : Bytes'], 'Bytes',
  "(cat (cat (cat (cat (cat 0x554f524302010101 (number 1)) (number n)) (number (prim length body : Nat))) (basisIdentity)) body)")
D('linkedArchiveAs', ['prefix : Bytes', 'suffix : Bytes', 'n : Nat', 'payload : Bytes'], '(Option Bytes)',
  "(seal (linkedPreimage n (cat (cat (cat prefix (number (prim length payload : Nat))) payload) suffix)))")
D('linkedArchive', ['suffix : Bytes', 'n : Nat', 'payload : Bytes'], '(Option Bytes)',
  "(linkedArchiveAs (linkedPrefix) suffix n payload)")
# The v0.2 basis (gen_basis V02_*): its library is the first v02Blocks blocks of this one. v0.2 archives
# name it, so they keep their meaning: linked against it, decoded by the same decoders.
assert GB.V02_LIB.startswith(wire.uleb(GB.G.v02_blocks))
V02_BLOCKS = GB.V02_LIB[len(wire.uleb(GB.G.v02_blocks)):]
assert GB.LIBB.startswith(V02_BLOCKS)
assert GB._hex('linked-prefix.hex') == LPRE
D('v02Identity', [], 'Bytes', '0x' + GB.V02_ID.hex())
D('v02Blocks', [], 'Nat', str(GB.G.v02_blocks))
D('v02Library', [], 'Bytes', '(libraryPrefix %d)' % len(V02_BLOCKS))
D('v02UlzSuffix', [], 'Bytes', '0x' + GB._hex('linked-ulz-suffix.hex').hex())
D('v02NcmSuffix', [], 'Bytes', '0x' + GB._hex('linked-ncm-suffix.hex').hex())
D('v02Archive', ['suffix : Bytes', 'n : Nat', 'payload : Bytes'], '(Option Bytes)',
  "(seal (cat (cat (cat (cat (cat (cat 0x554f524302010101 (number 1)) (number n)) (number (prim length (cat (cat (cat (linkedPrefix) (number (prim length payload : Nat))) payload) suffix) : Nat))) (v02Identity)) (cat (cat (linkedPrefix) (number (prim length payload : Nat))) payload)) suffix))")
D('byteAt', ['data : Bytes', 'at : Nat'], 'Nat',
  "(match (prim index data at : (Option UInt8))\n    | Option.none => 256\n    | Option.some octet => (prim to_nat octet : Nat))")
# (object length, offset of the basis identity) of a one-object archive whose body runs to its seal
D('linkedHeader', ['data : Bytes'], '(Option (Nat * Nat))',
  "(if ((byteAt data 8) =? 1) then (match (readUleb 10 data 9 1 0)\n    | Option.none => (ctor Option.none [(Nat * Nat)])\n    | Option.some n => (match (readUleb 10 data (second n) 1 0)\n      | Option.none => (ctor Option.none [(Nat * Nat)])\n      | Option.some b => (if ((((second b) + 64) + (first b)) =? (prim length data : Nat)) then (ctor Option.some [(Nat * Nat)] ((first n), (second b))) else (ctor Option.none [(Nat * Nat)])))) else (ctor Option.none [(Nat * Nat)]))")
D('linkedUlz', ['data : Bytes', 'n : Nat', 'payload : Bytes'], '(Option Bytes)',
  f"(match (linkedArchive (ulzSuffix) n payload)\n    | Option.none => (ctor Option.none [Bytes])\n    | Option.some rebuilt => (if (prim equal rebuilt data : Bool) then (let out : Bytes := ({DEC}::decode payload); "
  f"(if (((prim length out : Nat) =? n) && (prim equal ({ENC}::encode out) payload : Bool)) then (ctor Option.some [Bytes] out) else (ctor Option.none [Bytes]))) else (ctor Option.none [Bytes])))")
D('linkedNcm', ['data : Bytes', 'n : Nat', 'payload : Bytes'], '(Option Bytes)',
  f"(match (linkedArchive (ncmSuffix) n payload)\n    | Option.none => (ctor Option.none [Bytes])\n    | Option.some rebuilt => (if (prim equal rebuilt data : Bool) then (let out : Bytes := ({NDEC}::run payload); "
  f"(if (((prim length out : Nat) =? n) && (prim equal ({NENC}::run out) payload : Bool)) then (ctor Option.some [Bytes] out) else (ctor Option.none [Bytes]))) else (ctor Option.none [Bytes])))")
# the x86-filtered linked archive: the decoder's output, unfiltered
D('linkedNcmx', ['data : Bytes', 'n : Nat', 'payload : Bytes'], '(Option Bytes)',
  f"(match (linkedArchiveAs (x86Prefix) (ncmxSuffix) n payload)\n    | Option.none => (ctor Option.none [Bytes])\n    | Option.some rebuilt => (if (prim equal rebuilt data : Bool) then (let out : Bytes := (Uorc.Codec.X86::run 1 0 ({NDEC}::run payload)); "
  f"(if (((prim length out : Nat) =? n) && (prim equal ({NENC}::run (Uorc.Codec.X86::run 0 0 out)) payload : Bool)) then (ctor Option.some [Bytes] out) else (ctor Option.none [Bytes]))) else (ctor Option.none [Bytes])))")
# a v0.2 linked archive (ulz or ncm over the v0.2 basis)
D('linkedV02', ['data : Bytes', 'n : Nat', 'payload : Bytes'], '(Option Bytes)',
  f"(match (v02Archive (v02UlzSuffix) n payload)\n    | Option.none => {NONE_B}\n    | Option.some ulz => (if (prim equal ulz data : Bool) then (let out : Bytes := ({DEC}::decode payload); "
  f"(if (((prim length out : Nat) =? n) && (prim equal ({ENC}::encode out) payload : Bool)) then (ctor Option.some [Bytes] out) else {NONE_B})) else (match (v02Archive (v02NcmSuffix) n payload)\n      | Option.none => {NONE_B}\n      | Option.some ncm => (if (prim equal ncm data : Bool) then (let out : Bytes := ({NDEC}::run payload); "
  f"(if (((prim length out : Nat) =? n) && (prim equal ({NENC}::run out) payload : Bool)) then (ctor Option.some [Bytes] out) else {NONE_B})) else {NONE_B}))))")
# The v0.3 basis (gen_basis V03_*): its library is the first v03Blocks blocks of this one. v0.3 archives name
# it, so they keep their meaning: the same prefixes, suffixes that reach the run blocks at the v0.3 distance.
assert GB.V03_LIB.startswith(wire.uleb(GB.G.v03_blocks))
V03_BLOCKS = GB.V03_LIB[len(wire.uleb(GB.G.v03_blocks)):]
assert GB.LIBB.startswith(V03_BLOCKS)
D('v03Identity', [], 'Bytes', '0x' + GB.V03_ID.hex())
D('v03Blocks', [], 'Nat', str(GB.G.v03_blocks))
D('v03Library', [], 'Bytes', '(libraryPrefix %d)' % len(V03_BLOCKS))
for c, name in (('ulz', 'v03UlzSuffix'), ('ncm', 'v03NcmSuffix'), ('ncmx', 'v03NcmxSuffix')):
    D(name, [], 'Bytes', '0x' + GB.V03_TEMPLATES[c][1].hex())
D('v03Archive', ['prefix : Bytes', 'suffix : Bytes', 'n : Nat', 'payload : Bytes'], '(Option Bytes)',
  "(let body : Bytes := (cat (cat (cat prefix (number (prim length payload : Nat))) payload) suffix); "
  "(seal (cat (cat (cat (cat (cat 0x554f524302010101 (number 1)) (number n)) (number (prim length body : Nat))) (v03Identity)) body)))")
D('linkedV03', ['data : Bytes', 'n : Nat', 'payload : Bytes'], '(Option Bytes)',
  f"(match (v03Archive (linkedPrefix) (v03UlzSuffix) n payload)\n    | Option.none => {NONE_B}\n    | Option.some ulz => (if (prim equal ulz data : Bool) then (let out : Bytes := ({DEC}::decode payload); "
  f"(if (((prim length out : Nat) =? n) && (prim equal ({ENC}::encode out) payload : Bool)) then (ctor Option.some [Bytes] out) else {NONE_B})) else (match (v03Archive (linkedPrefix) (v03NcmSuffix) n payload)\n      | Option.none => {NONE_B}\n      | Option.some ncm => (if (prim equal ncm data : Bool) then (let out : Bytes := ({NDEC}::run payload); "
  f"(if (((prim length out : Nat) =? n) && (prim equal ({NENC}::run out) payload : Bool)) then (ctor Option.some [Bytes] out) else {NONE_B})) else (match (v03Archive (x86Prefix) (v03NcmxSuffix) n payload)\n        | Option.none => {NONE_B}\n        | Option.some ncmx => (if (prim equal ncmx data : Bool) then (let out : Bytes := (Uorc.Codec.X86::run 1 0 ({NDEC}::run payload)); "
  f"(if (((prim length out : Nat) =? n) && (prim equal ({NENC}::run (Uorc.Codec.X86::run 0 0 out)) payload : Bool)) then (ctor Option.some [Bytes] out) else {NONE_B})) else {NONE_B}))))))")
# The v0.4 basis's identity and library (its frames are read below, with the profiled ones)
assert GB.V04_LIB.startswith(wire.uleb(GB.G.v04_blocks))
V04_BLOCKS = GB.V04_LIB[len(wire.uleb(GB.G.v04_blocks)):]
assert GB.LIBB.startswith(V04_BLOCKS)
D('v04Identity', [], 'Bytes', '0x' + GB.V04_ID.hex())
D('v04Blocks', [], 'Nat', str(GB.G.v04_blocks))
D('v04Library', [], 'Bytes', '(libraryPrefix %d)' % len(V04_BLOCKS))
# Profiled entries (gen_basis PCODECS): prefix ++ uleb(len) ++ payload ++ 00 ++ uleb(profile) ++ tail, over k = 0..2
# window frames, x = 1 over the x86 filter. The profile word is the coder's (ncm_model; bits 0-16 contexts, 17-19
# the match features).
for x, xs in ((0, ''), (1, 'x')):
    for k in (0, 1, 2):
        c = 'ncmp' + (str(k) if k else '') + xs
        D(f'profile{k}{xs.upper()}Prefix', [], 'Bytes', '0x' + GB.PTEMPLATES[c][0].hex())
        D(f'profile{k}{xs.upper()}Tail', [], 'Bytes', '0x' + GB.PTEMPLATES[c][1].hex())
assert len({len(p) for p, _ in GB.PTEMPLATES.values()}) == 1


# A window entry over k >= 1 frames is the one-frame entry with three more nodes per frame (nat r, basis, and from
# the second frame on the concatenation with the history so far); the call to the coder then reaches the payload
# and the profile word 3 nodes further back per frame. So any window depth is one entry form, the depth read from
# it (native archive::window_profile_template).
def deep_parts(x):
    pre, tail = GB.PTEMPLATES['ncmp1' + ('x' if x else '')]
    d = 8 if x else 5
    rest = tail[6:]
    at = rest.index(bytes([d, 1, d - 1]))
    assert rest.count(bytes([d, 1, d - 1])) == 1 and tail[:6] == bytes.fromhex('030000011e01')
    return pre[3], rest[:at], rest[at + 3:]


def deep_template(x, k):
    one, head, end = deep_parts(x)
    d = 8 if x else 5
    more = 3 * (k - 1)
    frames = bytes.fromhex('00011e01') + b''.join(b'\x00' + wire.uleb(r) + bytes.fromhex('1e011b0301') for r in range(2, k + 1))
    return (bytes([1, 0, 3]) + wire.uleb(one + more) + bytes([3]),
            bytes.fromhex('0300') + frames + head + wire.uleb(d + more) + b'\x01' + wire.uleb(d - 1 + more) + end)


for x, xs in ((0, ''), (1, 'x')):
    for k in (1, 2):
        assert deep_template(x, k) == GB.PTEMPLATES['ncmp%d%s' % (k, xs)], (x, k)
    one, head, end = deep_parts(x)
    D(f'deep{xs.upper()}Head', [], 'Bytes', '0x' + head.hex())
    D(f'deep{xs.upper()}End', [], 'Bytes', '0x' + end.hex())
    D(f'deep{xs.upper()}Nodes', [], 'Nat', str(one))
D('windowFrames', ['k : Nat'], 'Bytes',
  "(match k\n    | Nat.zero => 0x\n    | Nat.succ m => (if (m =? 0) then 0x00011e01 else (cat (windowFrames m) (cat (cat 0x00 (number k)) 0x1e011b0301))))", 'k')
D('deepPrefix', ['x : Nat', 'k : Nat'], 'Bytes',
  "(cat (cat 0x010003 (number ((if (x =? 0) then (deepNodes) else (deepXNodes)) + (prim multiply 3 (prim subtract k 1 : Nat) : Nat)))) 0x03)")
D('deepTail', ['x : Nat', 'k : Nat'], 'Bytes',
  "(let more : Nat := (prim multiply 3 (prim subtract k 1 : Nat) : Nat); (let d : Nat := ((if (x =? 0) then 5 else 8) + more); "
  "(cat (cat (cat (cat (cat (cat 0x0300 (windowFrames k)) (if (x =? 0) then (deepHead) else (deepXHead))) (number d)) 0x01) (number (prim subtract d 1 : Nat))) (if (x =? 0) then (deepEnd) else (deepXEnd)))))")
D('profilePrefix', ['x : Nat', 'k : Nat'], 'Bytes',
  "(if (k =? 0) then (if (x =? 0) then (profile0Prefix) else (profile0XPrefix)) else (deepPrefix x k))")
D('profileTail', ['x : Nat', 'k : Nat'], 'Bytes',
  "(if (k =? 0) then (if (x =? 0) then (profile0Tail) else (profile0XTail)) else (deepTail x k))")
D('profiledBody', ['x : Nat', 'k : Nat', 'profile : Nat', 'payload : Bytes'], 'Bytes',
  "(cat (cat (cat (cat (profilePrefix x k) (number (prim length payload : Nat))) payload) (cat 0x00 (number profile))) (profileTail x k))")
# a profile word names a subset of the coder's 20 features
# and the tables' bits when the frame records them: bits 20-25 the context table (0, or 5 to 32:
# ncm::TABLE_MIN..TABLE_MAX), bits 26-30 the match tables (0, or any)
D('profileWord', ['profile : Nat'], 'Bool',
  f"(let t : Nat := (prim remainder (prim quotient profile {1 << 20} 0 : Nat) 64 0 : Nat); ((profile <? {1 << 31}) && ((t =? 0) || ((5 <=? t) && (t <=? 32)))))")
# a frame's word: a profile word in bits 0-31, bits 32-36 reserved, and the frame's streams (ncm::streams):
# streams - 1 in bits 37-52 and the log2 of the epoch in bits 53-57, one stream carrying no epoch
S_AT, E_AT = 1 << 37, 1 << 53
D('frameWord', ['word : Nat'], 'Bool',
  f"(let e : Nat := (prim quotient word {E_AT} 0 : Nat); (let k : Nat := (prim remainder (prim quotient word {S_AT} 0 : Nat) 65536 0 : Nat); "
  f"((((e <? 32) && ((prim remainder (prim quotient word {1 << 32} 0 : Nat) 32 0 : Nat) =? 0)) && (profileWord (prim remainder word {1 << 32} 0 : Nat))) && ((0 <? k) || (e =? 0)))))")
# a v0.4 frame's word: v0.4's ncm blocks read no table field
D('v04ProfileWord', ['profile : Nat'], 'Bool', f"(profile <? {1 << 20})")
# a frame of one stream is coded by the frame's coder, a frame of more by the streams coder
D('ncmDecoder', ['history : Bytes', 'payload : Bytes', 'word : Nat'], 'Bytes',
  f"(if (word <? {S_AT}) then ({NDEC}::runProfile history payload word) else ({NDEC}::runStreams history payload word))")
D('ncmEncoder', ['history : Bytes', 'input : Bytes', 'word : Nat'], 'Bytes',
  f"(if (word <? {S_AT}) then ({NENC}::runProfile history input word) else ({NENC}::runStreams history input word))")
D('profiledDecoder', ['x : Nat', 'history : Bytes', 'payload : Bytes', 'profile : Nat'], 'Bytes',
  f"(if (x =? 0) then (ncmDecoder history payload profile) else "
  f"(Uorc.Codec.X86::run 1 (prim length history : Nat) (ncmDecoder (Uorc.Codec.X86::run 0 0 history) payload profile)))")
D('profiledEncoder', ['x : Nat', 'history : Bytes', 'input : Bytes', 'profile : Nat'], 'Bytes',
  f"(if (x =? 0) then (ncmEncoder history input profile) else "
  f"(let h : Bytes := (Uorc.Codec.X86::run 0 0 history); (let f : Bytes := (Uorc.Codec.X86::run 0 (prim length history : Nat) input); (ncmEncoder h f profile))))")

# A linked archive this client writes: the rebuilt archive confirms the basis identity, the template and the
# seal, and the payload must be the one the encoder writes for its output.
D('linkedDecode', ['data : Bytes'], '(Option Bytes)',
  "(match (linkedHeader data)\n    | Option.none => (ctor Option.none [Bytes])\n    | Option.some h => (match (payloadAt data (((second h) + 32) + (prim length (linkedPrefix) : Nat)))\n      | Option.none => (ctor Option.none [Bytes])\n      | Option.some payload => (match (linkedUlz data (first h) payload)\n        | Option.some out => (ctor Option.some [Bytes] out)\n        | Option.none => (match (linkedNcm data (first h) payload)\n          | Option.some out => (ctor Option.some [Bytes] out)\n          | Option.none => (match (linkedNcmx data (first h) payload)\n            | Option.some out => (ctor Option.some [Bytes] out)\n            | Option.none => (match (linkedV03 data (first h) payload)\n              | Option.some out => (ctor Option.some [Bytes] out)\n              | Option.none => (linkedV02 data (first h) payload)))))))")
D('fastDecode', ['data : Bytes'], '(Option Bytes)',
  "(if ((byteAt data 6) =? 1) then (linkedDecode data) else (if ((byteAt data 7) =? 0) then (rawPayload data) else (kernelDecode data)))")
# Linking: a sealed conditional archive over the standard basis whose body is `uleb(k) blocks.. e` (e < 128)
# means the self-contained archive whose body is `uleb(L + k) library blocks.. uleb(L + e)`.
D('linkedBodyAs', ['count : Nat', 'library : Bytes', 'body : Bytes'], '(Option Bytes)',
  "(match (readUleb 10 body 0 1 0)\n    | Option.none => (ctor Option.none [Bytes])\n    | Option.some k => (let last : Nat := (prim subtract (prim length body : Nat) 1 : Nat); (let e : Nat := (byteAt body last); "
  "(if (((second k) <=? last) && (e <? 128)) then (match (prim slice body (second k) (prim subtract last (second k) : Nat) : (Option Bytes))\n      | Option.none => (ctor Option.none [Bytes])\n      | Option.some blocks => (ctor Option.some [Bytes] (cat (cat (cat (number (count + (first k))) library) blocks) (number (count + e))))) else (ctor Option.none [Bytes])))))")
D('linkedBody', ['body : Bytes'], '(Option Bytes)', "(linkedBodyAs (libraryBlocks) (basisLibrary) body)")
MAT = ("(if (((byteAt data 6) =? 1) && ((byteAt data 7) =? 1)) then (match (linkedHeader data)\n    | Option.none => (ctor Option.none [Bytes])\n    | Option.some h => (let at : Nat := ((second h) + 32); (let end : Nat := (prim subtract (prim length data : Nat) 32 : Nat); (match (prim slice data (second h) 32 : (Option Bytes))\n      | Option.none => (ctor Option.none [Bytes])\n      | Option.some identity => (match (prim slice data at (prim subtract end at : Nat) : (Option Bytes))\n        | Option.none => (ctor Option.none [Bytes])\n        | Option.some body => (match (prim slice data 0 end : (Option Bytes))\n          | Option.none => (ctor Option.none [Bytes])\n          | Option.some pre => (match (seal pre)\n            | Option.none => (ctor Option.none [Bytes])\n            | Option.some resealed => (if (prim equal resealed data : Bool) then (match (if (prim equal identity (basisIdentity) : Bool) then (linkedBody body) else (if (prim equal identity (v04Identity) : Bool) then (linkedBodyAs (v04Blocks) (v04Library) body) else (if (prim equal identity (v03Identity) : Bool) then (linkedBodyAs (v03Blocks) (v03Library) body) else (if (prim equal identity (v02Identity) : Bool) then (linkedBodyAs (v02Blocks) (v02Library) body) else (ctor Option.none [Bytes])))))\n              | Option.none => (ctor Option.none [Bytes])\n              | Option.some whole => (seal (preimage 0x01 (first h) whole))) else (ctor Option.none [Bytes]))))))))) else (ctor Option.none [Bytes]))")
D('materialize', ['data : Bytes'], '(Option Bytes)', MAT)
# compress candidates
D('ulzLinkedArchive', ['input : Bytes'], '(Option Bytes)',
  f"(let payload : Bytes := ({ENC}::encode input); (if (prim equal ({DEC}::decode payload) input : Bool) then (linkedArchive (ulzSuffix) (prim length input : Nat) payload) else (ctor Option.none [Bytes])))")
D('ncmxLinkedArchive', ['input : Bytes'], '(Option Bytes)',
  f"(let f : Bytes := (Uorc.Codec.X86::run 0 0 input); (let payload : Bytes := ({NENC}::run f); (if ((0 <? (prim length payload : Nat)) && (prim equal ({NDEC}::run payload) f : Bool)) then (linkedArchiveAs (x86Prefix) (ncmxSuffix) (prim length input : Nat) payload) else (ctor Option.none [Bytes]))))")
# whether a frame is x86 code: one rewritten call or jump per 256 bytes or more (measured: denser frames
# gain 1-8% under the filter, sparser ones lose up to 3%)
D('x86Dense', ['input : Bytes'], 'Bool',
  f"(let h : Nat := (Uorc.Codec.X86::hits input); ((0 <? h) && ((prim length input : Nat) <=? (prim multiply 256 h : Nat))))")
D('ncmLinkedArchive', ['input : Bytes'], '(Option Bytes)',
  f"(let payload : Bytes := ({NENC}::run input); (if ((0 <? (prim length payload : Nat)) && (prim equal ({NDEC}::run payload) input : Bool)) then (linkedArchive (ncmSuffix) (prim length input : Nat) payload) else (ctor Option.none [Bytes])))")
# codec 0 ulz (earlier archives only), 1 ncm, 2 ncm over the x86 filter; the client writes 1 or 2
D('linkedCandidate', ['codec : Nat', 'input : Bytes'], '(Option Bytes)',
  "(if (codec =? 2) then (ncmxLinkedArchive input) else (ncmLinkedArchive input))")

# ---- window frames: a frame is a linked archive whose basis is [library, the K preceding frames' outputs] ----
# Its identity is the UORC-BASIS-V1 chain over those roots, and its entry decodes the payload with the window
# (roots 1..K, oldest first) as the decoder's history, so matches reach back across frame boundaries.
NONE_B = "(ctor Option.none [Bytes])"
D('basisTag', [], 'Bytes', '0x' + GB.TAG.hex())
D('libraryRoot', [], 'Bytes', "(cat (number (libraryBlocks)) (basisLibrary))")
for k in (1, 2):
    assert GB.TEMPLATES[f'ulz{k}'][0] == GB.TEMPLATES[f'ncm{k}'][0]
    D(f'window{k}Prefix', [], 'Bytes', '0x' + GB.TEMPLATES[f'ulz{k}'][0].hex())
    D(f'window{k}Suffix', [], 'Bytes', '0x' + GB.TEMPLATES[f'ulz{k}'][1].hex())
    D(f'window{k}NcmSuffix', [], 'Bytes', '0x' + GB.TEMPLATES[f'ncm{k}'][1].hex())
    D(f'window{k}XPrefix', [], 'Bytes', '0x' + GB.TEMPLATES[f'ncm{k}x'][0].hex())
    D(f'window{k}XNcmSuffix', [], 'Bytes', '0x' + GB.TEMPLATES[f'ncm{k}x'][1].hex())
D('windowCount', ['window : (List Bytes)'], 'Nat',
  "(match window\n    | List.nil => 0\n    | List.cons r rest => (1 + (windowCount rest)))", 'window')
D('windowRoots', ['window : (List Bytes)'], 'Bytes',
  "(match window\n    | List.nil => 0x\n    | List.cons r rest => (cat (cat (number (prim length r : Nat)) r) (windowRoots rest)))", 'window')
D('windowHistory', ['window : (List Bytes)'], 'Bytes',
  "(match window\n    | List.nil => 0x\n    | List.cons r rest => (cat r (windowHistory rest)))", 'window')
D('digest', ['pre : Bytes'], '(Option Bytes)',
  f"(match ({SHA}::hash (cat 0x{DOM} 0x00))\n    | Result.error failure => {NONE_B}\n    | Result.ok c => (chainLoop ((prim quotient (prim length pre : Nat) 64 0 : Nat) + 1) pre 0 c))")
D('windowIdentity', ['window : (List Bytes)'], '(Option Bytes)',
  "(digest (cat (cat (cat (cat (basisTag) (number (1 + (windowCount window)))) (number (prim length (libraryRoot) : Nat))) (libraryRoot)) (windowRoots window)))")
# codec 0 ulz, 1 ncm, 2 ncm over the x86 filter
D('windowPrefix', ['codec : Nat', 'k : Nat'], 'Bytes',
  "(if (codec <? 2) then (if (k =? 1) then (window1Prefix) else (if (k =? 2) then (window2Prefix) else 0x)) else (if (k =? 1) then (window1XPrefix) else (if (k =? 2) then (window2XPrefix) else 0x)))")
D('windowSuffix', ['codec : Nat', 'k : Nat'], 'Bytes',
  "(if (codec =? 0) then (if (k =? 1) then (window1Suffix) else (if (k =? 2) then (window2Suffix) else 0x)) else (if (codec =? 1) then (if (k =? 1) then (window1NcmSuffix) else (if (k =? 2) then (window2NcmSuffix) else 0x)) "
  "else (if (k =? 1) then (window1XNcmSuffix) else (if (k =? 2) then (window2XNcmSuffix) else 0x))))")
D('windowPreimage', ['identity : Bytes', 'n : Nat', 'body : Bytes'], 'Bytes',
  "(cat (cat (cat (cat (cat 0x554f524302010101 (number 1)) (number n)) (number (prim length body : Nat))) identity) body)")
D('windowArchive', ['codec : Nat', 'k : Nat', 'identity : Bytes', 'n : Nat', 'payload : Bytes'], '(Option Bytes)',
  f"(if ((1 <=? k) && (k <=? 2)) then (seal (windowPreimage identity n (cat (cat (cat (windowPrefix codec k) (number (prim length payload : Nat))) payload) (windowSuffix codec k)))) else {NONE_B})")
# the codec's decoder and encoder against a window history
# the filtered codecs code the filtered frame placed after the filtered history
D('windowDecoder', ['codec : Nat', 'history : Bytes', 'payload : Bytes'], 'Bytes',
  f"(if (codec =? 0) then ({DEC}::decodeWith history payload) else (if (codec =? 1) then ({NDEC}::runWith history payload) else "
  f"(let h : Bytes := (Uorc.Codec.X86::run 0 0 history); (Uorc.Codec.X86::run 1 (prim length history : Nat) ({NDEC}::runWith h payload)))))")
D('windowEncoder', ['codec : Nat', 'history : Bytes', 'input : Bytes'], 'Bytes',
  f"(if (codec =? 0) then ({ENC}::encodeWith history input) else (if (codec =? 1) then ({NENC}::runWith history input) else "
  f"(let h : Bytes := (Uorc.Codec.X86::run 0 0 history); (let f : Bytes := (Uorc.Codec.X86::run 0 (prim length history : Nat) input); ({NENC}::runWith h f)))))")
D('windowCandidate', ['codec : Nat', 'window : (List Bytes)', 'input : Bytes'], '(Option Bytes)',
  f"(match window\n    | List.nil => (linkedCandidate codec input)\n    | List.cons oldest rest => (match (windowIdentity window)\n      | Option.none => {NONE_B}\n      | Option.some identity => (let history : Bytes := (windowHistory window); (let payload : Bytes := (windowEncoder codec history input); "
  f"(if ((0 <? (prim length payload : Nat)) && (prim equal (windowDecoder codec history payload) input : Bool)) then (windowArchive codec (windowCount window) identity (prim length input : Nat) payload) else {NONE_B})))))")
# v0.2 window frames: ulz over [v0.2 library, frames]
D('v02Root', [], 'Bytes', "(cat (number (v02Blocks)) (v02Library))")
D('v02WindowIdentity', ['window : (List Bytes)'], '(Option Bytes)',
  "(digest (cat (cat (cat (cat (basisTag) (number (1 + (windowCount window)))) (number (prim length (v02Root) : Nat))) (v02Root)) (windowRoots window)))")
for k in (1, 2):
    D(f'v02Window{k}Prefix', [], 'Bytes', '0x' + GB._hex(f'linked-ulz{k}-prefix.hex').hex())
    D(f'v02Window{k}Suffix', [], 'Bytes', '0x' + GB._hex(f'linked-ulz{k}-suffix.hex').hex())
D('v02WindowPrefix', ['k : Nat'], 'Bytes', "(if (k =? 1) then (v02Window1Prefix) else (if (k =? 2) then (v02Window2Prefix) else 0x))")
D('v02WindowSuffix', ['k : Nat'], 'Bytes', "(if (k =? 1) then (v02Window1Suffix) else (if (k =? 2) then (v02Window2Suffix) else 0x))")
D('v02WindowArchive', ['k : Nat', 'identity : Bytes', 'n : Nat', 'payload : Bytes'], '(Option Bytes)',
  f"(if ((1 <=? k) && (k <=? 2)) then (seal (windowPreimage identity n (cat (cat (cat (v02WindowPrefix k) (number (prim length payload : Nat))) payload) (v02WindowSuffix k)))) else {NONE_B})")
WDA = (f"(let k : Nat := (windowCount window); (match (linkedHeader data)\n    | Option.none => {NONE_B}\n    | Option.some h => (match (prim slice data (second h) 32 : (Option Bytes))\n      | Option.none => {NONE_B}\n      | Option.some identity => (match (payloadAt data (((second h) + 32) + (prim length (windowPrefix codec k) : Nat)))\n        | Option.none => {NONE_B}\n        | Option.some payload => (match (windowArchive codec k identity (first h) payload)\n          | Option.none => {NONE_B}\n          | Option.some rebuilt => (match (windowIdentity window)\n            | Option.none => {NONE_B}\n            | Option.some expected => (if ((prim equal rebuilt data : Bool) && (prim equal identity expected : Bool)) then (let history : Bytes := (windowHistory window); (let out : Bytes := (windowDecoder codec history payload); "
  f"(if (((prim length out : Nat) =? (first h)) && (prim equal (windowEncoder codec history out) payload : Bool)) then (ctor Option.some [Bytes] out) else {NONE_B}))) else {NONE_B})))))))")
D('windowDecodeAs', ['codec : Nat', 'data : Bytes', 'window : (List Bytes)'], '(Option Bytes)', WDA)
WV02 = WDA
for a, b in [('(windowPrefix codec k)', '(v02WindowPrefix k)'), ('(windowArchive codec k identity (first h) payload)', '(v02WindowArchive k identity (first h) payload)'),
             ('(windowIdentity window)', '(v02WindowIdentity window)'), ('(windowDecoder codec history payload)', f'({DEC}::decodeWith history payload)'),
             ('(windowEncoder codec history out)', f'({ENC}::encodeWith history out)')]:
    assert WV02.count(a) == 1, a
    WV02 = WV02.replace(a, b)
D('windowDecodeV02', ['data : Bytes', 'window : (List Bytes)'], '(Option Bytes)', WV02)
D('windowDecode', ['data : Bytes', 'window : (List Bytes)'], '(Option Bytes)',
  "(match (windowDecodeAs 0 data window)\n    | Option.some out => (ctor Option.some [Bytes] out)\n    | Option.none => (match (windowDecodeAs 1 data window)\n      | Option.some out => (ctor Option.some [Bytes] out)\n      | Option.none => (match (windowDecodeAs 2 data window)\n        | Option.some out => (ctor Option.some [Bytes] out)\n        | Option.none => (windowDecodeV02 data window))))")
# v0.3 window frames: ulz or ncm over [v0.3 library, frames], with the v0.3 suffixes
D('v03Root', [], 'Bytes', "(cat (number (v03Blocks)) (v03Library))")
D('v03WindowIdentity', ['window : (List Bytes)'], '(Option Bytes)',
  "(digest (cat (cat (cat (cat (basisTag) (number (1 + (windowCount window)))) (number (prim length (v03Root) : Nat))) (v03Root)) (windowRoots window)))")
for k in (1, 2):
    for c, name in ((f'ulz{k}', f'v03Window{k}Suffix'), (f'ncm{k}', f'v03Window{k}NcmSuffix'), (f'ncm{k}x', f'v03Window{k}XNcmSuffix')):
        D(name, [], 'Bytes', '0x' + GB.V03_TEMPLATES[c][1].hex())
D('v03WindowSuffix', ['codec : Nat', 'k : Nat'], 'Bytes',
  "(if (codec =? 0) then (if (k =? 1) then (v03Window1Suffix) else (if (k =? 2) then (v03Window2Suffix) else 0x)) else (if (codec =? 1) then (if (k =? 1) then (v03Window1NcmSuffix) else (if (k =? 2) then (v03Window2NcmSuffix) else 0x)) "
  "else (if (k =? 1) then (v03Window1XNcmSuffix) else (if (k =? 2) then (v03Window2XNcmSuffix) else 0x))))")
D('v03WindowArchive', ['codec : Nat', 'k : Nat', 'identity : Bytes', 'n : Nat', 'payload : Bytes'], '(Option Bytes)',
  f"(if ((1 <=? k) && (k <=? 2)) then (seal (windowPreimage identity n (cat (cat (cat (windowPrefix codec k) (number (prim length payload : Nat))) payload) (v03WindowSuffix codec k)))) else {NONE_B})")
WV03 = WDA
for a, b in [('(windowArchive codec k identity (first h) payload)', '(v03WindowArchive codec k identity (first h) payload)'),
             ('(windowIdentity window)', '(v03WindowIdentity window)')]:
    assert WV03.count(a) == 1, a
    WV03 = WV03.replace(a, b)
D('windowDecodeV03As', ['codec : Nat', 'data : Bytes', 'window : (List Bytes)'], '(Option Bytes)', WV03)
D('windowDecodeV03', ['data : Bytes', 'window : (List Bytes)'], '(Option Bytes)',
  "(match (windowDecodeV03As 0 data window)\n    | Option.some out => (ctor Option.some [Bytes] out)\n    | Option.none => (match (windowDecodeV03As 1 data window)\n      | Option.some out => (ctor Option.some [Bytes] out)\n      | Option.none => (windowDecodeV03As 2 data window)))")
# A profiled frame over the k = windowCount window frames (k = 0: a linked archive over the standard basis), its
# payload coded under the profile word its entry carries; x = 1 over the x86 filter. As for every entry, the
# rebuilt archive confirms the identity, the template and the seal, and the payload must be the encoder's.
D('profiledIdentity', ['window : (List Bytes)'], '(Option Bytes)',
  "(match window\n    | List.nil => (ctor Option.some [Bytes] (basisIdentity))\n    | List.cons oldest rest => (windowIdentity window))")
D('profiledArchive', ['x : Nat', 'k : Nat', 'identity : Bytes', 'n : Nat', 'profile : Nat', 'payload : Bytes'], '(Option Bytes)',
  f"(if (frameWord profile) then (seal (windowPreimage identity n (profiledBody x k profile payload))) else {NONE_B})")
PDA = (f"(let k : Nat := (windowCount window); (match (linkedHeader data)\n    | Option.none => {NONE_B}\n    | Option.some h => (let at : Nat := (((second h) + 32) + (prim length (profilePrefix x k) : Nat)); (match (readUleb 10 data at 1 0)\n      | Option.none => {NONE_B}\n      | Option.some pl => (match (readUleb 10 data (((second pl) + (first pl)) + 1) 1 0)\n        | Option.none => {NONE_B}\n        | Option.some pw => (let profile : Nat := (first pw); (match (payloadAt data at)\n          | Option.none => {NONE_B}\n          | Option.some payload => (match (profiledIdentity window)\n            | Option.none => {NONE_B}\n            | Option.some identity => (match (profiledArchive x k identity (first h) profile payload)\n              | Option.none => {NONE_B}\n              | Option.some rebuilt => (if (prim equal rebuilt data : Bool) then (let history : Bytes := (windowHistory window); (let out : Bytes := (profiledDecoder x history payload profile); "
       f"(if (((prim length out : Nat) =? (first h)) && (prim equal (profiledEncoder x history out profile) payload : Bool)) then (ctor Option.some [Bytes] out) else {NONE_B}))) else {NONE_B}))))))))))")
D('profiledDecodeAs', ['x : Nat', 'data : Bytes', 'window : (List Bytes)'], '(Option Bytes)', PDA)
D('profiledDecodeCurrent', ['data : Bytes', 'window : (List Bytes)'], '(Option Bytes)',
  "(match (profiledDecodeAs 0 data window)\n    | Option.some out => (ctor Option.some [Bytes] out)\n    | Option.none => (profiledDecodeAs 1 data window))")
# profile 0x1ffff is the v0.3 coder byte for byte, so its frame is written exactly as v0.3 wrote it: the old-shape
# entry over the v0.3 identity, which every v0.3 decoder reads and which carries no profile word
D('v03Candidate', ['x : Nat', 'window : (List Bytes)', 'n : Nat', 'payload : Bytes'], '(Option Bytes)',
  "(match window\n    | List.nil => (if (x =? 1) then (v03Archive (x86Prefix) (v03NcmxSuffix) n payload) else (v03Archive (linkedPrefix) (v03NcmSuffix) n payload))\n    | List.cons oldest rest => (match (v03WindowIdentity window)\n      | Option.none => " + NONE_B + "\n      | Option.some identity => (v03WindowArchive (x + 1) (windowCount window) identity n payload)))")
# The v0.4 basis (gen_basis V04_*): its library is the first v04Blocks blocks of this one. v0.4 wrote profiled frames
# (and its V03 frames as v0.3 archives): the same prefixes, the frozen v0.4 tails that reach ncmRunProfile at the v0.4
# distance, over the v0.4 identity (alone, or over a window).
D('v04Root', [], 'Bytes', "(cat (number (v04Blocks)) (v04Library))")
D('v04WindowIdentity', ['window : (List Bytes)'], '(Option Bytes)',
  "(digest (cat (cat (cat (cat (basisTag) (number (1 + (windowCount window)))) (number (prim length (v04Root) : Nat))) (v04Root)) (windowRoots window)))")
for x, xs in ((0, ''), (1, 'x')):
    for k in (0, 1, 2):
        c = 'ncmp' + (str(k) if k else '') + xs
        D(f'v04Profile{k}{xs.upper()}Tail', [], 'Bytes', '0x' + GB.V04_PTAILS[c].hex())
D('v04ProfileTail', ['x : Nat', 'k : Nat'], 'Bytes',
  "(if (x =? 0) then (if (k =? 0) then (v04Profile0Tail) else (if (k =? 1) then (v04Profile1Tail) else (v04Profile2Tail))) "
  "else (if (k =? 0) then (v04Profile0XTail) else (if (k =? 1) then (v04Profile1XTail) else (v04Profile2XTail))))")
D('v04ProfiledIdentity', ['window : (List Bytes)'], '(Option Bytes)',
  "(match window\n    | List.nil => (ctor Option.some [Bytes] (v04Identity))\n    | List.cons oldest rest => (v04WindowIdentity window))")
D('v04ProfiledArchive', ['x : Nat', 'k : Nat', 'identity : Bytes', 'n : Nat', 'profile : Nat', 'payload : Bytes'], '(Option Bytes)',
  f"(if ((k <=? 2) && (v04ProfileWord profile)) then (seal (windowPreimage identity n (cat (cat (cat (cat (profilePrefix x k) (number (prim length payload : Nat))) payload) (cat 0x00 (number profile))) (v04ProfileTail x k)))) else {NONE_B})")
PV04 = PDA
for a, b in [('(profiledIdentity window)', '(v04ProfiledIdentity window)'),
             ('(profiledArchive x k identity (first h) profile payload)', '(v04ProfiledArchive x k identity (first h) profile payload)')]:
    assert PV04.count(a) == 1, a
    PV04 = PV04.replace(a, b)
D('v04ProfiledDecodeAs', ['x : Nat', 'data : Bytes', 'window : (List Bytes)'], '(Option Bytes)', PV04)
# the archive coding `input` (after the frames of `window`) under `profile`, with codec 1 ncm or 2 ncm over the
# x86 filter: the candidate the client's strategy portfolio weighs. A profiled frame needs no block added after v0.4,
# so it is written over the v0.4 basis, whose x86 tails are 2 bytes shorter: exactly the frame v0.4 writes.
# A word that records table bits (>= 2^20) is read by the current blocks only, so its frame is the current era's.
D('profileCandidate', ['codec : Nat', 'profile : Nat', 'window : (List Bytes)', 'input : Bytes'], '(Option Bytes)',
  f"(let x : Nat := (if (codec =? 2) then 1 else 0); (let history : Bytes := (windowHistory window); (let payload : Bytes := (profiledEncoder x history input profile); "
  f"(if ((0 <? (prim length payload : Nat)) && (prim equal (profiledDecoder x history payload profile) input : Bool)) then (if ((profile =? 131071) && ((windowCount window) <=? 2)) then (v03Candidate x window (prim length input : Nat) payload) else (if ((v04ProfileWord profile) && ((windowCount window) <=? 2)) then (match (v04ProfiledIdentity window)\n    | Option.none => {NONE_B}\n    | Option.some identity => (v04ProfiledArchive x (windowCount window) identity (prim length input : Nat) profile payload)) else (match (profiledIdentity window)\n    | Option.none => {NONE_B}\n    | Option.some identity => (profiledArchive x (windowCount window) identity (prim length input : Nat) profile payload)))) else {NONE_B}))))")
# The profile the encoder learns for `input` under `profile` (NcmEncode.learnProfile), over the x86 filter when x = 1,
# and the candidate that codes it too: the learned frame is kept when it is no larger than the strategy's own.
D('profiledLearner', ['x : Nat', 'history : Bytes', 'input : Bytes', 'profile : Nat'], 'Nat',
  f"(if (x =? 0) then ({NENC}::learnProfile history input profile) else "
  f"(let h : Bytes := (Uorc.Codec.X86::run 0 0 history); (let f : Bytes := (Uorc.Codec.X86::run 0 (prim length history : Nat) input); ({NENC}::learnProfile h f profile))))")
D('learnedCandidate', ['codec : Nat', 'profile : Nat', 'window : (List Bytes)', 'input : Bytes'], '(Option Bytes)',
  f"(let plain : (Option Bytes) := (profileCandidate codec profile window input); (let learned : Nat := (profiledLearner (if (codec =? 2) then 1 else 0) (windowHistory window) input profile); "
  f"(if (learned =? profile) then plain else (match (profileCandidate codec learned window input)\n    | Option.none => plain\n    | Option.some l => (match plain\n      | Option.none => (ctor Option.some [Bytes] l)\n      | Option.some p => (if ((prim length l : Nat) <=? (prim length p : Nat)) then (ctor Option.some [Bytes] l) else plain))))))")

# ---- kappa-named model states: a state frame is a linked archive over the basis [library, S], S a model state ----
# (bytes the ncm coder codes first, gen_state.py), named by the identity of that basis like any one-root window. Its
# entry (gen_basis WCODECS) applies ncmRunWarm to the payload, root 1 and the frame's profile word, so the frame is
# coded as the continuation of S; x = 1 over the x86 filter, S filtered first and the frame placed after it.
for x, xs in ((0, ''), (1, 'x')):
    c = 'ncmw' + xs
    D(f'warm{xs.upper()}Prefix', [], 'Bytes', '0x' + GB.PTEMPLATES[c][0].hex())
    D(f'warm{xs.upper()}Tail', [], 'Bytes', '0x' + GB.PTEMPLATES[c][1].hex())
D('warmPrefixOf', ['x : Nat'], 'Bytes', "(if (x =? 0) then (warmPrefix) else (warmXPrefix))")
D('warmTailOf', ['x : Nat'], 'Bytes', "(if (x =? 0) then (warmTail) else (warmXTail))")
D('stateBody', ['x : Nat', 'profile : Nat', 'payload : Bytes'], 'Bytes',
  "(cat (cat (cat (cat (warmPrefixOf x) (number (prim length payload : Nat))) payload) (cat 0x00 (number profile))) (warmTailOf x))")
# windowIdentity [state], written out: an empty state names nothing, and the window count is stated as
# 1 + 0 * |state| so the kernel cannot reduce it. With a literal count the identity's prefix (tag, count, root
# length, the 40 KB library root) is a closed term, and the export's certificate evaluates it.
D('stateIdentity', ['state : Bytes'], '(Option Bytes)', f"(if (0 <? (prim length state : Nat)) then (digest (cat (cat (cat (cat (basisTag) (number (2 + (prim multiply 0 (prim length state : Nat) : Nat)))) (number (prim length (libraryRoot) : Nat))) (libraryRoot)) (windowRoots (cons state (nil Bytes))))) else {NONE_B})")
D('stateArchive', ['x : Nat', 'identity : Bytes', 'n : Nat', 'profile : Nat', 'payload : Bytes'], '(Option Bytes)',
  f"(if (profileWord profile) then (seal (windowPreimage identity n (stateBody x profile payload))) else {NONE_B})")
D('stateDecoder', ['x : Nat', 'state : Bytes', 'payload : Bytes', 'profile : Nat'], 'Bytes',
  f"(if (x =? 0) then ({NDEC}::runWarm state payload profile) else "
  f"(Uorc.Codec.X86::run 1 (prim length state : Nat) ({NDEC}::runWarm (Uorc.Codec.X86::run 0 0 state) payload profile)))")
D('stateEncoder', ['x : Nat', 'state : Bytes', 'input : Bytes', 'profile : Nat'], 'Bytes',
  f"(if (x =? 0) then ({NENC}::runWarm state input profile) else "
  f"(let h : Bytes := (Uorc.Codec.X86::run 0 0 state); (let f : Bytes := (Uorc.Codec.X86::run 0 (prim length state : Nat) input); ({NENC}::runWarm h f profile))))")
# the archive coding `input` from the model state `state` under `profile`: a candidate of the strategy portfolio
D('stateCandidate', ['x : Nat', 'profile : Nat', 'state : Bytes', 'input : Bytes'], '(Option Bytes)',
  f"(let payload : Bytes := (stateEncoder x state input profile); (if ((0 <? (prim length payload : Nat)) && (prim equal (stateDecoder x state payload profile) input : Bool)) then (match (stateIdentity state)\n    | Option.none => {NONE_B}\n    | Option.some identity => (stateArchive x identity (prim length input : Nat) profile payload)) else {NONE_B}))")
SDA = (f"(match (linkedHeader data)\n    | Option.none => {NONE_B}\n    | Option.some h => (let at : Nat := (((second h) + 32) + (prim length (warmPrefixOf x) : Nat)); (match (readUleb 10 data at 1 0)\n      | Option.none => {NONE_B}\n      | Option.some pl => (match (readUleb 10 data (((second pl) + (first pl)) + 1) 1 0)\n        | Option.none => {NONE_B}\n        | Option.some pw => (let profile : Nat := (first pw); (match (payloadAt data at)\n          | Option.none => {NONE_B}\n          | Option.some payload => (match (stateIdentity state)\n            | Option.none => {NONE_B}\n            | Option.some identity => (match (stateArchive x identity (first h) profile payload)\n              | Option.none => {NONE_B}\n              | Option.some rebuilt => (if (prim equal rebuilt data : Bool) then (let out : Bytes := (stateDecoder x state payload profile); "
       f"(if (((prim length out : Nat) =? (first h)) && (prim equal (stateEncoder x state out profile) payload : Bool)) then (ctor Option.some [Bytes] out) else {NONE_B})) else {NONE_B})))))))))")
D('stateDecodeAs', ['x : Nat', 'data : Bytes', 'state : Bytes'], '(Option Bytes)', SDA)
# the state a frame names, of the client's states: the one whose basis [library, S] has the frame's identity
D('namedState', ['data : Bytes', 'states : (List Bytes)'], '(Option Bytes)',
  f"(match states\n    | List.nil => {NONE_B}\n    | List.cons state rest => (match (linkedHeader data)\n      | Option.none => {NONE_B}\n      | Option.some h => (match (prim slice data (second h) 32 : (Option Bytes))\n        | Option.none => {NONE_B}\n        | Option.some named => (match (stateIdentity state)\n          | Option.none => {NONE_B}\n          | Option.some identity => (if (prim equal named identity : Bool) then (ctor Option.some [Bytes] state) else (namedState data rest))))))", 'states')
D('stateDecode', ['data : Bytes', 'states : (List Bytes)'], '(Option Bytes)',
  f"(match (namedState data states)\n    | Option.none => {NONE_B}\n    | Option.some state => (match (stateDecodeAs 0 data state)\n      | Option.some out => (ctor Option.some [Bytes] out)\n      | Option.none => (stateDecodeAs 1 data state)))")
D('profiledDecode', ['data : Bytes', 'window : (List Bytes)'], '(Option Bytes)',
  "(match (profiledDecodeCurrent data window)\n    | Option.some out => (ctor Option.some [Bytes] out)\n    | Option.none => (match (v04ProfiledDecodeAs 0 data window)\n      | Option.some out => (ctor Option.some [Bytes] out)\n      | Option.none => (v04ProfiledDecodeAs 1 data window)))")
D('fastDecodeWith', ['data : Bytes', 'window : (List Bytes)'], '(Option Bytes)',
  "(match (match window\n    | List.nil => (fastDecode data)\n    | List.cons oldest rest => (match (windowDecode data window)\n      | Option.some out => (ctor Option.some [Bytes] out)\n      | Option.none => (windowDecodeV03 data window)))\n    | Option.some out => (ctor Option.some [Bytes] out)\n    | Option.none => (profiledDecode data window))")
WMAT = (f"(if (((byteAt data 6) =? 1) && ((byteAt data 7) =? 1)) then (match (linkedHeader data)\n    | Option.none => {NONE_B}\n    | Option.some h => (let at : Nat := ((second h) + 32); (let end : Nat := (prim subtract (prim length data : Nat) 32 : Nat); (match (prim slice data (second h) 32 : (Option Bytes))\n      | Option.none => {NONE_B}\n      | Option.some identity => (match (prim slice data at (prim subtract end at : Nat) : (Option Bytes))\n        | Option.none => {NONE_B}\n        | Option.some body => (match (prim slice data 0 end : (Option Bytes))\n          | Option.none => {NONE_B}\n          | Option.some pre => (match (seal pre)\n            | Option.none => {NONE_B}\n            | Option.some resealed => (match (windowIdentity window)\n              | Option.none => {NONE_B}\n              | Option.some expected => (if (prim equal resealed data : Bool) then (match (if (prim equal identity expected : Bool) then (linkedBody body) else (match (v04WindowIdentity window)\n                | Option.none => {NONE_B}\n                | Option.some v04 => (if (prim equal identity v04 : Bool) then (linkedBodyAs (v04Blocks) (v04Library) body) else (match (v03WindowIdentity window)\n                | Option.none => {NONE_B}\n                | Option.some v03 => (if (prim equal identity v03 : Bool) then (linkedBodyAs (v03Blocks) (v03Library) body) else (match (v02WindowIdentity window)\n                  | Option.none => {NONE_B}\n                  | Option.some old => (if (prim equal identity old : Bool) then (linkedBodyAs (v02Blocks) (v02Library) body) else {NONE_B})))))))\n                | Option.none => {NONE_B}\n                | Option.some whole => (seal (windowPreimage identity (first h) whole))) else {NONE_B}))))))))) else {NONE_B})")
D('windowMaterialize', ['data : Bytes', 'window : (List Bytes)'], '(Option Bytes)', WMAT)
# (identity, library root) of the basis a window archive over `window` names: the v0.2 one, else this one
D('windowBasis', ['data : Bytes', 'window : (List Bytes)'], '(Option (Bytes * Bytes))',
  "(match (windowIdentity window)\n    | Option.none => (ctor Option.none [(Bytes * Bytes)])\n    | Option.some current => (match (v02WindowIdentity window)\n      | Option.none => (ctor Option.none [(Bytes * Bytes)])\n      | Option.some old => (match (linkedHeader data)\n        | Option.none => (ctor Option.some [(Bytes * Bytes)] (current, (libraryRoot)))\n        | Option.some h => (match (prim slice data (second h) 32 : (Option Bytes))\n          | Option.none => (ctor Option.some [(Bytes * Bytes)] (current, (libraryRoot)))\n          | Option.some named => (if (prim equal named old : Bool) then (ctor Option.some [(Bytes * Bytes)] (old, (v02Root))) else (match (v03WindowIdentity window)\n            | Option.none => (ctor Option.none [(Bytes * Bytes)])\n            | Option.some v03 => (if (prim equal named v03 : Bool) then (ctor Option.some [(Bytes * Bytes)] (v03, (v03Root))) else (match (v04WindowIdentity window)\n              | Option.none => (ctor Option.none [(Bytes * Bytes)])\n              | Option.some v04 => (if (prim equal named v04 : Bool) then (ctor Option.some [(Bytes * Bytes)] (v04, (v04Root))) else (ctor Option.some [(Bytes * Bytes)] (current, (libraryRoot))))))))))))")
src = ("\\begin{lexlean}{Uorc.Codec.Archive}\n\\importmodule{Uorc.Codec.LzDecode}\n\\importmodule{Uorc.Codec.LzEncode}\n\\importmodule{Uorc.Codec.NcmDecode}\n\\importmodule{Uorc.Codec.NcmEncode}\n\\importmodule{Uorc.Codec.X86}\n\\importmodule{Uorc.Integrity.Sha256}\n\\title{Parameters}\n\\begin{semanticmodule}\n"
       + "\n".join(defs) + "\\end{semanticmodule}\n\\end{lexlean}\n")
open(os.path.join(HERE, '..', 'src', 'Uorc', 'Codec', 'Archive.lex.tex'), 'w').write(src)
print('prefix', len(PREFIX), 'suffix', len(SUFFIX))
