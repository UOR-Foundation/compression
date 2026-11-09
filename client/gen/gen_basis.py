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
    _, nrun = ncmgraph.library(g)
    return g, {'ulz': urun, 'ncm': nrun}


G, RUN = libraries()
L = len(G.blocks)
LIBB = b''.join(block_bytes(b) for b in G.blocks)
LIB = wire.uleb(L) + LIBB
STD_ID = wire.chain(TAG + wire.uleb(1) + wire.uleb(len(LIB)) + LIB)


def entry(g, run, payload):
    e = g.block('entry', [], S)
    e.returns(e.apply(run, [e.seq(payload)]))
    return e


def template(codec):
    e = entry(G, RUN[codec], PH)
    eb = block_bytes(e)
    G.blocks.pop()
    body = wire.uleb(1) + eb + wire.uleb(0)
    cut = body.index(bytes([3, 2]) + PH)
    assert body.count(bytes([3, 2]) + PH) == 1
    return body[:cut + 1], body[cut + 4:]


TEMPLATES = {c: template(c) for c in ('ulz', 'ncm')}


def linked_body(codec, payload):
    pre, suf = TEMPLATES[codec]
    return pre + wire.uleb(len(payload)) + payload + suf


def materialize(body):
    """The self-contained graph a linked body means (one entry block, entry 0)."""
    assert body[0] == 1 and body[-1] == 0
    return wire.uleb(L + 1) + LIBB + body[1:-1] + wire.uleb(L)


def linked_archive(codec, n, payload):
    body = linked_body(codec, payload)
    pre = b'UORC' + bytes([2, 1, 1, 1]) + wire.uleb(1) + wire.uleb(n) + wire.uleb(len(body)) + STD_ID + body
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
        write_hex(f'linked-{c}-suffix.hex', suf)
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
