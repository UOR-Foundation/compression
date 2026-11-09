# Runs the kernel graph by the Python mirror of the reference semantics (g.py) on archives the
# prototype encoder (ulz.py, byte-identical to the LexLean encoder) writes, and compares the output.
import os, sys, random
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import ulz, ulzgraph, g
random.seed(7)
cases = [b'', b'a', b'abcabcabcabcabc hello hello', bytes(3000), os.urandom(2000),
         bytes(random.choice(b'ab') for _ in range(3000)), open(__file__, 'rb').read()]
for data in cases:
    G, entry = ulzgraph.build(ulz.compress(data))
    assert g.evaluate(G, entry) == data, len(data)
print('kernel graph decodes', len(cases), 'cases;', sum(len(b.ins) for b in G.blocks), 'instructions in', len(G.blocks), 'blocks')
