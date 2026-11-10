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
import ulzgraph, ncmgraph, wire

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
    return g, {'ulz': urun, 'ncm': nrun, 'ulz1': uwith, 'ulz2': uwith}


# Window entries: roots 1..K of the basis are the outputs of the K preceding frames, oldest first.
WINDOWS = {'ulz1': 1, 'ulz2': 2}


G, RUN = libraries()
L = len(G.blocks)
LIBB = b''.join(block_bytes(b) for b in G.blocks)
LIB = wire.uleb(L) + LIBB
STD_ID = wire.chain(TAG + wire.uleb(1) + wire.uleb(len(LIB)) + LIB)


def entry(g, run, payload, k=0):
    e = g.block('entry', [], S)
    if k == 0:
        e.returns(e.apply(run, [e.seq(payload)]))
        return e
    pl = e.seq(payload)
    history = e.basis(1)
    for r in range(2, k + 1):
        history = e.sconcat(history, e.basis(r))
    e.returns(e.apply(run, [pl, history]))
    return e


def basis_id(window):
    """Identity of the basis [library, window roots..]."""
    pre = TAG + wire.uleb(1 + len(window)) + wire.uleb(len(LIB)) + LIB
    for r in window:
        pre += wire.uleb(len(r)) + r
    return wire.chain(pre)


def template(codec):
    e = entry(G, RUN[codec], PH, WINDOWS.get(codec, 0))
    eb = block_bytes(e)
    G.blocks.pop()
    body = wire.uleb(1) + eb + wire.uleb(0)
    cut = body.index(bytes([3, 2]) + PH)
    assert body.count(bytes([3, 2]) + PH) == 1
    return body[:cut + 1], body[cut + 4:]


TEMPLATES = {c: template(c) for c in ('ulz', 'ncm', 'ulz1', 'ulz2')}


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


def linked_body(codec, payload):
    pre, suf = TEMPLATES[codec]
    return pre + wire.uleb(len(payload)) + payload + suf


def materialize(body):
    """The self-contained graph a linked body means (one entry block, entry 0)."""
    assert body[0] == 1 and body[-1] == 0
    return wire.uleb(L + 1) + LIBB + body[1:-1] + wire.uleb(L)


def linked_archive(codec, n, payload, window=()):
    body = linked_body(codec, payload)
    ident = basis_id(window) if window else STD_ID
    pre = b'UORC' + bytes([2, 1, 1, 1]) + wire.uleb(1) + wire.uleb(n) + wire.uleb(len(body)) + ident + body
    return wire.sealed(pre)


def write_hex(name, data):
    text = data.hex()
    text = '\n'.join(text[i:i + 128] for i in range(0, len(text), 128)) + '\n'
    open(os.path.join(HERE, '..', '..', 'crates', 'uorc', 'src', name), 'w').write(text)


if __name__ == '__main__':
    write_hex('basis-id.hex', STD_ID)
    write_hex('basis-library.hex', LIB)
    assert TEMPLATES['ulz'][0] == TEMPLATES['ncm'][0]
    write_hex('linked-prefix.hex', TEMPLATES['ulz'][0])
    for c, (pre, suf) in TEMPLATES.items():
        if c in WINDOWS:
            write_hex(f'linked-{c}-prefix.hex', pre)
        write_hex(f'linked-{c}-suffix.hex', suf)
    write_hex('linked-ref.hex', REF_BODY)
    print('library blocks', L, 'bytes', len(LIB), 'identity', STD_ID.hex())
    for c, (pre, suf) in TEMPLATES.items():
        print(c, 'linked prefix', pre.hex(), 'suffix', suf.hex())
    if '--check' in sys.argv:
        import ulz, ncm_model
        text = b'the standard basis carries the kernels; archives name it by identity. ' * 3
        for c, enc in (('ulz', ulz.compress), ('ncm', ncm_model.py_encode)):
            payload = enc(text)
            # the materialized graph is the libraries plus the entry, exactly as built structurally
            g, runs = libraries()
            e = entry(g, runs[c], payload)
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
        frames = [b'window frame one: the standard basis carries the kernels. ' * 4,
                  b'window frame two: archives name the kernels by identity. ' * 4,
                  b'window frame three: the standard basis carries the kernels; archives name it by identity. ' * 2]
        for c, k in WINDOWS.items():
            window = frames[2 - k:2]
            hist = b''.join(window)
            payload = ulz.compress_with(hist, frames[2])
            g, runs = libraries()
            e = entry(g, runs[c], payload, k)
            assert materialize(linked_body(c, payload)) == wire.graph_bytes(g, e), c
            out = evaluate(g, e, basis=[LIB] + window)
            assert out == frames[2], (c, out[:60])
            alone = ulz.compress(frames[2])
            print(c, 'window payload', len(payload), 'vs alone', len(alone), '; decodes on the mirror')
