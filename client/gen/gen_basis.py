# The standard kernel basis (writing aid): the ulz and ncm decoder libraries as one UORC graph library,
# its identity, and the linked archive bodies that name it.
#
# A linked archive is a conditional archive (scope 1) whose 32-byte basis identity is STD_ID and whose
# body is a graph `uleb(k) blocks.. uleb(e)` whose blocks may apply library blocks: linking prepends the
# library, so the archive means exactly the self-contained archive with body
#   uleb(L + k) ++ library blocks ++ blocks.. ++ uleb(L + e).
# The client writes one entry block, `run(sequenceConstant payload)`, against ulz's or ncm's run block.
#
#   python3 client/gen/gen_basis.py           writes the crates/uorc/src/*.hex basis files
#   python3 client/gen/gen_basis.py --check   also decodes a linked archive of each codec on the graph mirror
import sys
import os
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from g import Graph, S, evaluate
import ulzgraph, ncmgraph, x86graph, wire

PH = b'\xaa\xbb'
TAG = b'UORC-BASIS-V1\x00'


def block_bytes(b):
    ins, ret = b.lowered()
    return (wire.uleb(len(b.params)) + bytes(wire.TC[t] for t in b.params) + bytes([wire.TC[b.result]])
            + wire.uleb(len(ins)) + b''.join(wire.instr(t) for t in ins) + wire.uleb(ret))


def libraries():
    g = Graph()
    _, urun = ulzgraph.library(g)
    uwith = g.run_with
    _, nrun = ncmgraph.library(g)
    # every block above is the v0.2 library, unchanged; the blocks below extend it
    g.v02_blocks = len(g.blocks)
    nwith = ncmgraph.window_library(g)
    _, x86 = x86graph.library(g)
    g.x86 = x86
    # every block above is the v0.3 library, unchanged; the profiled coder (ncm_model) follows
    g.v03_blocks = len(g.blocks)
    nprof = ncmgraph.profile_library(g)
    # every block above is the v0.4 library, unchanged; the current profiled coder (ncm_model: the profile word's
    # table fields) and the warm-state coder follow
    g.v04_blocks = len(g.blocks)
    nprof = ncmgraph.profile_library(g, prefix='ncmT', model=ncmgraph.ncm_model, name='ncmRunTables')
    nwarm = ncmgraph.warm_library(g)
    runs = {'ncmw': nwarm, 'ulz': urun, 'ncm': nrun, 'ulz1': uwith, 'ulz2': uwith, 'ncm1': nwith, 'ncm2': nwith,
            'ncmp': nprof, 'ncmp1': nprof, 'ncmp2': nprof}
    runs.update({c + 'x': r for c, r in list(runs.items()) if c.startswith('ncm') and not c.endswith('x')})
    return g, runs


# Window entries: roots 1..K of the basis are the outputs of the K preceding frames, oldest first.
WINDOWS = {'ulz1': 1, 'ulz2': 2, 'ncm1': 1, 'ncm2': 2, 'ncmp1': 1, 'ncmp2': 2}
WINDOWS.update({c + 'x': k for c, k in list(WINDOWS.items()) if c.startswith('ncm')})
# The client compresses with ncm only; the ulz entries stay so earlier archives keep their meaning.
# Codecs ending in x code the x86-filtered bytes (x86_model): the entry undoes the filter on the
# decoder's output, and a window entry filters the history first, the frame placed after it.
CODECS = ['ulz', 'ncm', 'ulz1', 'ulz2', 'ncm1', 'ncm2', 'ncmx', 'ncm1x', 'ncm2x']
# Profiled codecs (ncmp*): the entry applies ncmRunProfile to the payload, the history (empty without a window)
# and the frame's profile word, a natural constant right after the payload: the template is
# prefix ++ uleb(len) ++ payload ++ 00 ++ uleb(profile) ++ tail.
PCODECS = ['ncmp', 'ncmp1', 'ncmp2', 'ncmpx', 'ncmp1x', 'ncmp2x']
PROFILE_MARK = 0x7ffff
# State codecs (ncmw*): the entry applies ncmRunWarm to the payload, basis root 1 (a kappa-named model state: bytes
# the whole model codes first) and the frame's profile word; the template has the profiled shape.
WCODECS = ['ncmw', 'ncmwx']


G, RUN = libraries()
L = len(G.blocks)
LIBB = b''.join(block_bytes(b) for b in G.blocks)
LIB = wire.uleb(L) + LIBB
STD_ID = wire.chain(TAG + wire.uleb(1) + wire.uleb(len(LIB)) + LIB)

# The v0.2 basis (client/gen/v02): its library is this one's first blocks, byte for byte, so a v0.2
# archive (which applies only those) means the same linked against this library. The client accepts
# its identity beside STD_ID.
def _hex(name):
    return bytes.fromhex(open(os.path.join(HERE, 'v02', name)).read().replace('\n', ''))
V02_LIB = _hex('basis-library.hex')
V02_ID = _hex('basis-id.hex')
assert V02_ID == wire.chain(TAG + wire.uleb(1) + wire.uleb(len(V02_LIB)) + V02_LIB)
assert V02_LIB == wire.uleb(G.v02_blocks) + b''.join(block_bytes(b) for b in G.blocks[:G.v02_blocks]), 'the v0.2 blocks changed'

# The v0.3 basis (client/gen/v03): its library is this one's first v03_blocks blocks, byte for byte, so v0.3
# archives keep their meaning too. The client accepts its identity beside STD_ID.
def _hex3(name):
    return bytes.fromhex(open(os.path.join(HERE, 'v03', name)).read().replace('\n', ''))
V03_LIB = _hex3('basis-library.hex')
V03_ID = _hex3('basis-id.hex')
assert V03_ID == wire.chain(TAG + wire.uleb(1) + wire.uleb(len(V03_LIB)) + V03_LIB)
assert V03_LIB == wire.uleb(G.v03_blocks) + b''.join(block_bytes(b) for b in G.blocks[:G.v03_blocks]), 'the v0.3 blocks changed'
# The v0.4 basis (client/gen/v04): its library is this one's first v04_blocks blocks, byte for byte, so v0.4
# archives keep their meaning (their entries reach the profiled blocks at the same relative distance only when
# nothing is inserted before them; blocks are appended).
def _hex4(name):
    return bytes.fromhex(open(os.path.join(HERE, 'v04', name)).read().replace('\n', ''))
V04_LIB = _hex4('basis-library.hex')
V04_ID = _hex4('basis-id.hex')
assert V04_ID == wire.chain(TAG + wire.uleb(1) + wire.uleb(len(V04_LIB)) + V04_LIB)
assert V04_LIB == wire.uleb(G.v04_blocks) + b''.join(block_bytes(b) for b in G.blocks[:G.v04_blocks]), 'the v0.4 blocks changed'
# v0.4 wrote profiled entries only (its V03 frames are v0.3 archives): their tails, frozen, with this basis's prefixes
V04_PTAILS = {c: _hex4(f'linked-{c}-tail.hex') for c in PCODECS}
# a v0.3 entry: its frozen suffix with the current prefix (prefixes reference no block)
V03_TEMPLATES = None


def entry(g, run, payload, k=0, x86=False, profile=None, warm=False):
    e = g.block('entry', [], S)
    if warm:
        pl = e.seq(payload)
        pf = e.nat(profile)
        state = e.basis(1)
        if not x86:
            e.returns(e.apply(run, [pl, state, pf]))
            return e
        out = e.apply(run, [pl, e.apply(g.x86, [state, e.nat(0), e.nat(0)]), pf])
        e.returns(e.apply(g.x86, [out, e.nat(1), e.slen(state)]))
        return e
    if profile is not None:
        pl = e.seq(payload)
        pf = e.nat(profile)
        history = e.seq(b'')
        if k > 0:
            history = e.basis(1)
            for r in range(2, k + 1):
                history = e.sconcat(history, e.basis(r))
        if not x86:
            e.returns(e.apply(run, [pl, history, pf]))
            return e
        out = e.apply(run, [pl, e.apply(g.x86, [history, e.nat(0), e.nat(0)]), pf])
        e.returns(e.apply(g.x86, [out, e.nat(1), e.slen(history)]))
        return e
    if k == 0:
        out = e.apply(run, [e.seq(payload)])
        e.returns(e.apply(g.x86, [out, e.nat(1), e.nat(0)]) if x86 else out)
        return e
    pl = e.seq(payload)
    history = e.basis(1)
    for r in range(2, k + 1):
        history = e.sconcat(history, e.basis(r))
    if not x86:
        e.returns(e.apply(run, [pl, history]))
        return e
    out = e.apply(run, [pl, e.apply(g.x86, [history, e.nat(0), e.nat(0)])])
    e.returns(e.apply(g.x86, [out, e.nat(1), e.slen(history)]))
    return e


def basis_id(window):
    """Identity of the basis [library, window roots..]."""
    pre = TAG + wire.uleb(1 + len(window)) + wire.uleb(len(LIB)) + LIB
    for r in window:
        pre += wire.uleb(len(r)) + r
    return wire.chain(pre)


def template(codec):
    e = entry(G, RUN[codec], PH, WINDOWS.get(codec, 0), codec.endswith('x'))
    eb = block_bytes(e)
    G.blocks.pop()
    body = wire.uleb(1) + eb + wire.uleb(0)
    cut = body.index(bytes([3, 2]) + PH)
    assert body.count(bytes([3, 2]) + PH) == 1
    return body[:cut + 1], body[cut + 4:]


TEMPLATES = {c: template(c) for c in CODECS}
V03_TEMPLATES = {c: (TEMPLATES[c][0], _hex3(f'linked-{c}-suffix.hex')) for c in CODECS}


def profile_template(codec):
    """(prefix, tail) of a profiled codec: its body is prefix ++ uleb(len) ++ payload ++ 00 ++ uleb(profile) ++ tail."""
    e = entry(G, RUN[codec], PH, WINDOWS.get(codec, 0), codec.endswith('x'), PROFILE_MARK, codec in WCODECS)
    eb = block_bytes(e)
    G.blocks.pop()
    body = wire.uleb(1) + eb + wire.uleb(0)
    mark = bytes([3, 2]) + PH + bytes([0]) + wire.uleb(PROFILE_MARK)
    assert body.count(mark) == 1, codec
    cut = body.index(mark)
    return body[:cut + 1], body[cut + len(mark):]


PTEMPLATES = {c: profile_template(c) for c in PCODECS + WCODECS}


def reference_body():
    """Body of a reference frame: a conditional archive over [library, F] whose one entry block returns
    basis root 1, the earlier frame F itself. It applies no library block, so it is the same bytes for
    every library; only the basis identity (the frame's kappa-label) names F."""
    e = G.block('entry', [], S)
    e.returns(e.basis(1))
    eb = block_bytes(e)
    G.blocks.pop()
    return wire.uleb(1) + eb + wire.uleb(0)


REF_BODY = reference_body()


def linked_body(codec, payload, profile=None):
    if profile is not None:
        pre, tail = PTEMPLATES[codec]
        return pre + wire.uleb(len(payload)) + payload + bytes([0]) + wire.uleb(profile) + tail
    pre, suf = TEMPLATES[codec]
    return pre + wire.uleb(len(payload)) + payload + suf


def materialize(body):
    """The self-contained graph a linked body means (one entry block, entry 0)."""
    assert body[0] == 1 and body[-1] == 0
    return wire.uleb(L + 1) + LIBB + body[1:-1] + wire.uleb(L)


def linked_archive(codec, n, payload, window=(), profile=None):
    body = linked_body(codec, payload, profile)
    ident = basis_id(window) if window else STD_ID
    pre = b'UORC' + bytes([2, 1, 1, 1]) + wire.uleb(1) + wire.uleb(n) + wire.uleb(len(body)) + ident + body
    return wire.sealed(pre)


def write_hex(name, data):
    text = data.hex()
    text = '\n'.join(text[i:i + 128] for i in range(0, len(text), 128)) + '\n'
    open(os.path.join(HERE, '..', '..', 'crates', 'uorc', 'src', name), 'w').write(text)


if __name__ == '__main__':
    write_hex('basis-id.hex', STD_ID)
    write_hex('basis-v02-id.hex', V02_ID)
    write_hex('basis-library.hex', LIB)
    # ulz and ncm entries of one shape differ only after the payload
    for c in ('ncm', 'ncm1', 'ncm2'):
        assert TEMPLATES[c][0] == TEMPLATES[c.replace('ncm', 'ulz')][0], c
    write_hex('linked-prefix.hex', TEMPLATES['ulz'][0])
    for c, (pre, suf) in TEMPLATES.items():
        if c in ('ulz1', 'ulz2') or c.endswith('x'):
            write_hex(f'linked-{c}-prefix.hex', pre)
        write_hex(f'linked-{c}-suffix.hex', suf)
    write_hex('linked-ref.hex', REF_BODY)
    write_hex('basis-v03-id.hex', V03_ID)
    write_hex('basis-v03-library.hex', V03_LIB)
    write_hex('basis-v04-id.hex', V04_ID)
    write_hex('basis-v04-library.hex', V04_LIB)
    for c, tail in V04_PTAILS.items():
        write_hex(f'v04-linked-{c}-tail.hex', tail)
    # v0.3 entries: the same prefixes; their suffixes name the run blocks at their v0.3 (relative) distance
    for c in CODECS:
        assert TEMPLATES[c][0] == V03_TEMPLATES[c][0], c
        write_hex(f'v03-linked-{c}-suffix.hex', V03_TEMPLATES[c][1])
    for c, (pre, tail) in PTEMPLATES.items():
        write_hex(f'linked-{c}-prefix.hex', pre)
        write_hex(f'linked-{c}-tail.hex', tail)
    print('library blocks', L, 'bytes', len(LIB), 'identity', STD_ID.hex())
    for c, (pre, suf) in TEMPLATES.items():
        print(c, 'linked prefix', pre.hex(), 'suffix', suf.hex())
    for c, (pre, tail) in PTEMPLATES.items():
        print(c, 'profiled prefix', pre.hex(), 'tail', tail.hex())
    if '--check' in sys.argv:
        import ulz, ncm_model, x86_model
        text = b'the standard basis carries the kernels; archives name it by identity. ' * 3
        text = text[:100] + bytes([0xE8, 0x10, 0, 0, 0, 0xE8, 0xF0, 0xFF, 0xFF, 0xFF]) + text[100:]
        xf = lambda d, base=0: x86_model.py_filter(d, 0, base)
        V03 = ncm_model.V03
        for c, enc in (('ulz', ulz.compress), ('ncm', lambda d: ncm_model.py_encode(d, b'', V03)),
                       ('ncmx', lambda d: ncm_model.py_encode(xf(d), b'', V03))):
            payload = enc(text)
            # the materialized graph is the libraries plus the entry, exactly as built structurally
            g, runs = libraries()
            e = entry(g, runs[c], payload, 0, c.endswith('x'))
            whole = wire.graph_bytes(g, e)
            assert materialize(linked_body(c, payload)) == whole, c
            out = evaluate(g, e)
            assert out == text, (c, out[:40])
            print(c, 'linked archive', len(linked_archive(c, len(text), payload)), 'bytes; decodes on the mirror')
        # a reference frame reproduces the earlier frame it names
        g, _ = libraries()
        e = g.block('entry', [], S)
        e.returns(e.basis(1))
        assert materialize(REF_BODY) == wire.graph_bytes(g, e)
        assert evaluate(g, e, basis=[LIB, text]) == text
        print('ref body', REF_BODY.hex(), '; decodes on the mirror')
        # window frames: the payload is encoded against the preceding frames' outputs (basis roots 1..K)
        call = bytes([0xE8, 0x40, 1, 0, 0])
        frames = [b'window frame one: the standard basis carries the kernels. ' * 4 + call,
                  b'window frame two: archives name the kernels by identity. ' * 4 + call,
                  b'window frame three: the standard basis carries the kernels; archives name it by identity. ' * 2 + call]
        wenc = {'ulz': ulz.compress_with, 'ncm': lambda h, d: ncm_model.py_encode(d, h, V03)}
        for c, k in WINDOWS.items():
            if c not in CODECS:
                continue
            window = frames[2 - k:2]
            hist = b''.join(window)
            if c.endswith('x'):
                payload = wenc[c[:3]](xf(hist), xf(frames[2], len(hist)))
            else:
                payload = wenc[c[:3]](hist, frames[2])
            g, runs = libraries()
            e = entry(g, runs[c], payload, k, c.endswith('x'))
            assert materialize(linked_body(c, payload)) == wire.graph_bytes(g, e), c
            out = evaluate(g, e, basis=[LIB] + window)
            assert out == frames[2], (c, out[:60])
            alone = wenc[c[:3]](b'', frames[2])  # the window's gain
            print(c, 'window payload', len(payload), 'vs alone', len(alone), '; decodes on the mirror')
        # profiled entries: the payload coded under the profile word the entry carries
        for c in PCODECS:
            k = WINDOWS.get(c, 0)
            window = frames[2 - k:2]
            hist = b''.join(window)
            for prof in (ncm_model.FULL, 0x1059e | ncm_model.F_REP):
                if c.endswith('x'):
                    payload = ncm_model.py_encode(xf(frames[2], len(hist)), xf(hist), prof)
                else:
                    payload = ncm_model.py_encode(frames[2], hist, prof)
                g, runs = libraries()
                e = entry(g, runs[c], payload, k, c.endswith('x'), prof)
                assert materialize(linked_body(c, payload, prof)) == wire.graph_bytes(g, e), c
                out = evaluate(g, e, basis=[LIB] + window)
                assert out == frames[2], (c, hex(prof), out[:60])
                print(c, hex(prof), 'profiled payload', len(payload), '; decodes on the mirror')
        # state entries: the payload coded as the continuation of a kappa-named state, basis root 1
        state = b''.join(frames[:2])
        for c in WCODECS:
            prof = ncm_model.FULL
            if c.endswith('x'):
                payload = ncm_model.py_encode_warm(xf(frames[2], len(state)), xf(state), prof)
            else:
                payload = ncm_model.py_encode_warm(frames[2], state, prof)
            g, runs = libraries()
            e = entry(g, runs[c], payload, 0, c.endswith('x'), prof, True)
            assert materialize(linked_body(c, payload, prof)) == wire.graph_bytes(g, e), c
            out = evaluate(g, e, basis=[LIB, state])
            assert out == frames[2], (c, out[:60])
            print(c, 'state payload', len(payload), 'vs alone', len(ncm_model.py_encode(frames[2])), '; decodes on the mirror')
