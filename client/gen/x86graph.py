# The x86 filter's decoding direction as a UORC program graph block, compiled from x86_model by irgraph
# (scratch writing aid). library(g) adds x86Run(sequence, dir, base) -> sequence: dir 0 applies the
# filter, 1 undoes it, with the bytes placed at `base` in the stream.
import sys
import os
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from g import Graph, N, Y, S, T
import irgraph, x86_model


def library(g=None):
    c = irgraph.Compiler(x86_model.P, g)
    roms = c.load_roms()
    inp = c.input_loader()
    f = c.function('filter')
    b = c.g.block('x86Byte', [N, T], Y)
    b.returns(b.sread(b.arg(1), b.add(x86_model.P.tables['OUT'], b.arg(0))))
    out = b
    b = c.g.block('x86Run', [S, N, N], S)
    sq = b.arg(0)
    n = b.slen(sq)
    st = b.fold(n, b.snew(b.byte(0)), inp, [sq])
    st = b.apply(roms, [st])
    st = b.apply(f, [st, n, b.arg(1), b.arg(2)])
    b.returns(b.tabulate(n, out, [st]))
    return c.g, b
