# The ncm decoder as a UORC program graph, compiled from ncm_model by irgraph (scratch writing aid).
# library(): every block but the entry, the kernel that a linked archive names by identity.
# build(payload): library + entry, the self-contained kernel archive's program.
import sys
import os
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from g import Graph, N, Y, S, T
import irgraph, ncm_model


def library(g=None):
    c = irgraph.Compiler(ncm_model.P, g)
    roms = c.load_roms()
    inp = c.input_loader()
    dec = c.function('decode')
    b = c.g.block('outByte', [N, T], Y)
    b.returns(b.sread(b.arg(1), b.add(ncm_model.P.tables['H'], b.arg(0))))
    out = b
    # run(payload): the whole decode, so the entry is one application
    b = c.g.block('run', [S], S)
    pl = b.arg(0)
    st = b.fold(b.slen(pl), b.snew(b.byte(0)), inp, [pl])
    st = b.apply(roms, [st])
    st = b.apply(dec, [st])
    n = c.get(b, st, c.gkey['n'], ncm_model.P.globals['n'])
    b.returns(b.tabulate(n, out, [st]))
    return c.g, b


def build(payload):
    g, run = library()
    e = g.block('entry', [], S)
    e.returns(e.apply(run, [e.seq(payload)]))
    return g, e
