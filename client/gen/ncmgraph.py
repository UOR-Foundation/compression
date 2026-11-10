# The ncm decoder (the v0.2 coder frozen in ncm_v02.py; that program is ncm_model with hoff = 0) as a
# UORC program graph, compiled by irgraph (scratch writing aid). The window decoder (ncm_model) follows
# as its own blocks, so the v0.2 blocks, and the archives that name them, keep their indices.
# build(payload): library + entry, the self-contained kernel archive's program.
import sys
import os
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from g import Graph, N, Y, S, T
import irgraph, ncm_model, ncm_v02, ncm_v03


def library(g=None):
    c = irgraph.Compiler(ncm_v02.P, g)
    roms = c.load_roms()
    inp = c.input_loader()
    dec = c.function('decode')
    b = c.g.block('outByte', [N, T], Y)
    b.returns(b.sread(b.arg(1), b.add(ncm_v02.P.tables['H'], b.arg(0))))
    out = b
    # run(payload): the whole decode, so the entry is one application
    b = c.g.block('run', [S], S)
    pl = b.arg(0)
    st = b.fold(b.slen(pl), b.snew(b.byte(0)), inp, [pl])
    st = b.apply(roms, [st])
    st = b.apply(dec, [st])
    n = c.get(b, st, c.gkey['n'], ncm_v02.P.globals['n'])
    b.returns(b.tabulate(n, out, [st]))
    return c.g, b


def window_library(g):
    """ncmRunWith(payload, history), appended after every v0.2 block: the v0.3 coder (frozen in ncm_v03)."""
    c = irgraph.Compiler(ncm_v03.P, g)
    roms = c.load_roms()
    inp = c.input_loader()
    # runWith(payload, history): the decode against a window; H starts holding the history, the match
    # model reaches into it, and only the coded bytes after it are the output
    H = ncm_v03.P.tables['H']
    b = c.g.block('ncmHistoryByte', [N, T, S], T)
    b.returns(b.swrite(b.arg(1), b.add(H, b.arg(0)), b.sidx(b.arg(2), b.arg(0))))
    hb = b
    b = c.g.block('ncmOutAt', [N, T, N], Y)
    b.returns(b.sread(b.arg(1), b.add(b.add(H, b.arg(2)), b.arg(0))))
    oat = b
    decw = c.function('decodeWith')
    b = c.g.block('ncmRunWith', [S, S], S)
    pl, hs = b.arg(0), b.arg(1)
    st = b.fold(b.slen(pl), b.snew(b.byte(0)), inp, [pl])
    st = b.apply(roms, [st])
    h = b.slen(hs)
    st = b.fold(h, st, hb, [hs])
    st = b.apply(decw, [st, h])
    n = c.get(b, st, c.gkey['n'], ncm_v03.P.globals['n'])
    b.returns(b.tabulate(n, oat, [st, h]))
    return b


def profile_library(g, prefix='ncmP'):
    """ncmRunProfile(payload, history, profile), appended after the v0.3 blocks: the decode of the current
    coder (ncm_model) against a window, with the frame's profile word."""
    c = irgraph.Compiler(ncm_model.P, g)
    roms = c.load_roms()
    inp = c.input_loader()
    H = ncm_model.P.tables['H']
    b = c.g.block(prefix + 'HistoryByte', [N, T, S], T)
    b.returns(b.swrite(b.arg(1), b.add(H, b.arg(0)), b.sidx(b.arg(2), b.arg(0))))
    hb = b
    b = c.g.block(prefix + 'OutAt', [N, T, N], Y)
    b.returns(b.sread(b.arg(1), b.add(b.add(H, b.arg(2)), b.arg(0))))
    oat = b
    decp = c.function('decodeProfile')
    b = c.g.block('ncmRunProfile', [S, S, N], S)
    pl, hs, pf = b.arg(0), b.arg(1), b.arg(2)
    st = b.fold(b.slen(pl), b.snew(b.byte(0)), inp, [pl])
    st = b.apply(roms, [st])
    h = b.slen(hs)
    st = b.fold(h, st, hb, [hs])
    st = b.apply(decp, [st, h, pf])
    n = c.get(b, st, c.gkey['n'], ncm_model.P.globals['n'])
    b.returns(b.tabulate(n, oat, [st, h]))
    return b


def build(payload):
    g, run = library()
    e = g.block('entry', [], S)
    e.returns(e.apply(run, [e.seq(payload)]))
    return g, e
