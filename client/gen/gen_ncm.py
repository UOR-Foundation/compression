# Generates Uorc/Codec/NcmDecode.lex.tex and NcmEncode.lex.tex from ncm_model (writing aid).
import sys
import os
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import irlex, ncm_model, ncm_streams
from state import definer
from ir import K

TB = 'Uorc.Codec.Table'
OUTDIR = os.path.join(HERE, '..', 'src', 'Uorc', 'Codec')


def sizing(L):
    D, need = L.D, L.need
    D('depthFor', ['fuel : Nat', 'n : Nat', 'depth : Nat'], 'Nat',
      "(match fuel\n    | Nat.zero => depth\n    | Nat.succ m => (if (n <=? (pow2 depth)) then depth else (depthFor m n (depth + 1))))", 'fuel')
    D('tbLoop', ['fuel : Nat', 'n : Nat', 'tb : Nat'], 'Nat',
      "(match fuel\n    | Nat.zero => tb\n    | Nat.succ m => (if ((pow2 tb) <? (prim multiply 1024 n : Nat)) then (tbLoop m n (tb + 1)) else tb))", 'fuel')
    D('tbFor', ['n : Nat'], 'Nat', "(tbLoop 8 n 16)")
    D('mbFor', ['tb : Nat'], 'Nat', "(if (22 <? tb) then 22 else (prim subtract tb 2 : Nat))")
    # the context table's bits: the ones the profile word records in bits 20-25 (TABLE_SHIFT), else the length's own;
    # the match tables' bits: the ones it records in bits 26-30 (MATCH_SHIFT), else the default context table's
    D('tbOf', ['n : Nat', 'profile : Nat'], 'Nat',
      f"(let t : Nat := (prim remainder (prim quotient profile {need(ncm_model.TABLE_UNIT)} 0 : Nat) {need(ncm_model.TABLE_FIELD)} 0 : Nat); (if (0 <? t) then t else (tbFor n)))")
    D('mbOf', ['n : Nat', 'profile : Nat'], 'Nat',
      f"(let m : Nat := (prim remainder (prim quotient profile {need(ncm_model.MATCH_UNIT)} 0 : Nat) {need(ncm_model.MATCH_FIELD)} 0 : Nat); (if (0 <? m) then m else (mbFor (tbFor n))))")
    D('zeros', ['depth : Nat'], 'Bytes', f"({TB}::fill 1 depth 0)")
    # a streams frame word (ncm::streams): streams - 1 in bits 37-52; a frame cut into sk streams of a
    # stride of 64-byte units; the merged tables' entry space
    sb, sf, su = need(ncm_model.STREAMS_BASE), need(ncm_model.STREAMS_FIELD), need(ncm_streams.STRIDE)
    D('streamsOf', ['word : Nat'], 'Nat', f"((prim remainder (prim quotient word {sb} 0 : Nat) {sf} 0 : Nat) + 1)")
    D('strideOf', ['n : Nat', 'sk : Nat'], 'Nat',
      f"(prim multiply {su} (prim quotient ((prim quotient (prim subtract (n + sk) 1 : Nat) sk 0 : Nat) + (prim subtract {su} 1 : Nat)) {su} 0 : Nat) : Nat)")
    D('spaceOf', ['n : Nat', 'h : Nat', 'word : Nat'], 'Nat',
      f"((((pow2 (tbOf n word)) + (prim multiply 2 (pow2 (mbOf n word)) : Nat)) + {need(ncm_streams.STATIC)}) + (h + n))")


def lean_of(e, leaf, need):
    if isinstance(e, int):
        return need(e)
    if isinstance(e, str):
        return leaf[e]
    op, a, b = e
    if isinstance(b, int) and not isinstance(a, int):
        a, b = b, a      # Nat.add and Nat.mul recurse on their second argument: a literal goes first
    a, b = lean_of(a, leaf, need), lean_of(b, leaf, need)
    if op == 'add':
        return f"({a} + {b})"
    if op == 'div':
        return f"(prim quotient {a} {b} 0 : Nat)"
    return f"(prim multiply {a} {b} : Nat)"


STREAM_T = list(ncm_streams.regions(lambda x: x))


def stream_tables(L, n, h, word, streams):
    """The streams regions sized for a frame of n bytes after h under word (streams), or for the serial
    coder, which is stream 0 of one and uses only its private block. The serial roots never size them
    by the word: a size that branches on it makes the certificate kernel evaluate both sizes."""
    leaf = {'zs': f'(spaceOf {n} {h} {word})', 'sk': f'(streamsOf {word})', 'sr': f'(strideOf {n} (streamsOf {word}))'}
    out = {}
    # a depth's fuel: the bits of a word, the widest address
    fuel = L.need(ncm_streams.WORD_BITS)
    for t, (w, entries) in ncm_streams.regions(lambda x: x).items():
        # a frame of one stream (the serial coder) uses only its private block
        lane = f"(prim quotient {L.need(ncm_streams.PB)} {L.need(ncm_streams.WORD)} 0 : Nat)"
        one = (f"(depthFor {fuel} ({lane} + (prim multiply 0 {n} : Nat)) 0)" if t == 'PL'
               else f"(prim multiply 0 {n} : Nat)")
        out[t] = f"({TB}::fill {L.need(w)} {f'(depthFor {fuel} {lean_of(entries, leaf, L.need)} 0)' if streams else one} 0)"
    return out


# the serial tables' depths, each named once for every root that sizes them: the small tables' memory,
# the second refinement's entries, the short match and repeat models' slots
MEM_DEPTH, A2_DEPTH = K('memDepth', 22), K('refineDepth', 22)
M2S_DEPTH, RS_DEPTH = K('shortMatchDepth', 6), K('repeatDepth', 7)


def tables(L, n, extra, history='0x', profile='profile', h='0', streams=False):
    # Every table's depth names n: the certificate kernel evaluates whatever is closed, and a closed state
    # would have it run the model's whole setup (thousands of table writes) at check time.
    t = {'glob': f'(zeros (depthFor {L.need(ncm_streams.WORD_BITS)} ({GLOBS} + (prim multiply 0 {n} : Nat)) 0))', 'mem': f'(zeros ({L.need(MEM_DEPTH)} + (prim multiply 0 {n} : Nat)))', 'T': f'({TB}::fill 3 (tbOf {n} {profile}) 0)', 'MT': f'({TB}::fill 4 (mbOf {n} {profile}) 0)',
         'H': f'(prim append {history} (zeros (depthFor 40 {n} 0)) : Bytes)', 'A2': f'({TB}::fill 2 ({L.need(A2_DEPTH)} + (prim multiply 0 {n} : Nat)) 0)',
         # the short match table, and the short match and repeat models' 64 and 128 slots
         'M2T': f'({TB}::fill 4 (mbOf {n} {profile}) 0)', 'M2S': f'({TB}::fill 3 ({L.need(M2S_DEPTH)} + (prim multiply 0 {n} : Nat)) 0)',
         'RS': f'({TB}::fill 3 ({L.need(RS_DEPTH)} + (prim multiply 0 {n} : Nat)) 0)'}
    t.update(stream_tables(L, n, h, profile, streams))
    t.update(extra)
    return t


def scalars(L, inp):
    return {s.split(':')[0]: (inp if s.startswith('input:') else '0') for s in L.scalars}


# decoder
BIG = ['T', 'H', 'MT', 'A2', 'M2T', 'M2S', 'RS'] + STREAM_T
V03 = str(ncm_model.V03)
dec = irlex.Lex(ncm_model.P, 'NcmDecode', ['decodeProfile', 'decodeWarm', 'decodeStreams'], BIG)
sizing(dec)
_g = list(ncm_model.P.globals)[-1]
GLOBS = f"({dec.gat(_g)} + {dec.gwidth(_g)})"   # the globals' bytes: the last one's end
# a frame's length: its first LENGTH bytes, most significant first
LEN, BYTE = dec.need(ncm_streams.LENGTH), dec.need(ncm_streams.BYTE)
be = '(getAt 1 input 0)'
for q in range(1, ncm_streams.LENGTH):
    be = f"((prim multiply {BYTE} {be} : Nat) + (getAt 1 input {q}))"
n = f"(let n : Nat := {be}; "
# against a window: H starts holding the history (h bytes), and only the n coded bytes after it are returned
body = (n + "(let h : Nat := (prim length history : Nat); "
        f"(let s0 : {dec.St} := {dec.L.make(tables(dec, 'n', {}, 'history'), scalars(dec.L, 'input'))}; "
        f"(let s1 : {dec.St} := (decodeProfile s0 h profile); (match (prim slice {dec.L.P('s1', 'H')} h n : (Option Bytes))\n    | Option.none => 0x\n    | Option.some b => b)))))")
# the decode under a profile word (ncm_model: bits 0-16 contexts, then the match features); V03 is the v0.3 coder
dec.D('runProfile', ['history : Bytes', 'input : Bytes', 'profile : Nat'], 'Bytes', f"(if ((prim length input : Nat) <? 4) then 0x else {body})")
dec.D('runWith', ['history : Bytes', 'input : Bytes'], 'Bytes', f"(runProfile history input {V03})")
dec.D('run', ['input : Bytes'], 'Bytes', "(runWith 0x input)")
# from a kappa-named model state: H starts holding the state (w bytes), which the model codes first; the tables are
# sized for w + n bytes and only the n bytes after the state are returned
wbody = (n + "(let w : Nat := (prim length state : Nat); "
         f"(let s0 : {dec.St} := {dec.L.make(tables(dec, '(w + n)', {}, 'state'), scalars(dec.L, 'input'))}; "
         f"(let s1 : {dec.St} := (decodeWarm s0 w profile); (match (prim slice {dec.L.P('s1', 'H')} w n : (Option Bytes))\n    | Option.none => 0x\n    | Option.some b => b)))))")
dec.D('runWarm', ['state : Bytes', 'input : Bytes', 'profile : Nat'], 'Bytes', f"(if ((prim length input : Nat) <? 4) then 0x else {wbody})")
# a streams frame (ncm::streams::decode): the payload's streams decoded over the shared state, after a window
# (history) or from a kappa-named model state the whole model codes first (state; the other one empty): H
# starts holding both, the tables are sized for the state and the frame
sbody = (n + "(let h : Nat := (prim length history : Nat); (let w : Nat := (prim length state : Nat); "
         f"(let s0 : {dec.St} := {dec.L.make(tables(dec, '(w + n)', {}, '(prim append history state : Bytes)', 'word', 'h', True), scalars(dec.L, 'input'))}; "
         f"(let s1 : {dec.St} := (decodeStreams s0 h w word); (match (prim slice {dec.L.P('s1', 'H')} (h + w) n : (Option Bytes))\n    | Option.none => 0x\n    | Option.some b => b))))))")
dec.D('streamsFrom', ['history : Bytes', 'state : Bytes', 'input : Bytes', 'word : Nat'], 'Bytes', f"(if ((prim length input : Nat) <? {LEN}) then 0x else {sbody})")
dec.D('runStreams', ['history : Bytes', 'input : Bytes', 'word : Nat'], 'Bytes', "(streamsFrom history 0x input word)")
dec.D('runWarmStreams', ['state : Bytes', 'input : Bytes', 'word : Nat'], 'Bytes', "(streamsFrom 0x state input word)")
open(os.path.join(OUTDIR, 'NcmDecode.lex.tex'), 'w').write(dec.source([TB]))

# encoder
enc = irlex.Lex(ncm_model.P, 'NcmEncode', ['encodeProfile', 'encodeLearn', 'encodeWarm', 'encodeStreams'], BIG + ['OUT'])
sizing(enc)
enc.D('start', ['history : Bytes', 'input : Bytes', 'n : Nat', 'cap : Nat', 'profile : Nat'], enc.St,
      enc.L.make(tables(enc, 'n', {'OUT': '(zeros (depthFor 40 cap 0))'}, 'history'), scalars(enc.L, 'input')))
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
# encode `input` after `history` as a streams payload under the frame word `word`; empty when a stream's coder
# passes its bound (twice its length and 64 bytes), as the frame's coder does
# the payload's room, then every stream's output
cap = (f"((prim multiply {enc.need(ncm_streams.LENGTH)} (streamsOf word) : Nat) + (prim multiply (prim multiply 2 (streamsOf word) : Nat) "
       f"((prim multiply 2 (strideOf n (streamsOf word)) : Nat) + {enc.need(ncm_streams.SLACK)}) : Nat))")
# after a window (history) or from a model state (state; the other one empty), as the decoder
st = enc.L.make(tables(enc, '(w + n)', {'OUT': f'(zeros (depthFor {enc.need(ncm_streams.WORD_BITS)} {cap} 0))'}, '(prim append history state : Bytes)', 'word', 'h', True), scalars(enc.L, 'input'))
frame_range = enc.need(ncm_streams.FRAME_RANGE)
# the payload and the merge's steps (ncm::streams::encode_counted): one for every bucket, match table entry, and mixer
# weight or map entry a stream changed in an epoch, summed over the epochs and streams; no payload when a
# stream's coder passes its bound
cnt = ("(let n : Nat := (prim length input : Nat); (let h : Nat := (prim length history : Nat); (let w : Nat := (prim length state : Nat); "
       f"(let s1 : {enc.St} := (encodeStreams {st} n h w word); "
       f"(if (0 <? {enc.gread('s1', 'ovf')}) then (0x, 0) else (match (prim slice {enc.L.P('s1', 'OUT')} 0 {enc.gread('s1', 'flen')} : (Option Bytes))\n    | Option.none => (0x, 0)\n"
       f"    | Option.some b => (b, {enc.gread('s1', 'msteps')})))))))")
enc.D('streamsFrom', ['history : Bytes', 'state : Bytes', 'input : Bytes', 'word : Nat'], '(Bytes * Nat)', f"(if ((prim length input : Nat) <? {frame_range}) then {cnt} else (0x, 0))")
enc.D('runStreams', ['history : Bytes', 'input : Bytes', 'word : Nat'], 'Bytes', "(first (streamsFrom history 0x input word))")
enc.D('countStreams', ['history : Bytes', 'input : Bytes', 'word : Nat'], '(Bytes * Nat)', "(streamsFrom history 0x input word)")
enc.D('runWarmStreams', ['state : Bytes', 'input : Bytes', 'word : Nat'], 'Bytes', "(first (streamsFrom 0x state input word))")
enc.D('countWarmStreams', ['state : Bytes', 'input : Bytes', 'word : Nat'], '(Bytes * Nat)', "(streamsFrom 0x state input word)")
open(os.path.join(OUTDIR, 'NcmEncode.lex.tex'), 'w').write(enc.source([TB]))
print('ok')
