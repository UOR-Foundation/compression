# UORC frame + graph wire encoding, mirroring Uorc.Wire.Frame preimage and Uorc.Wire.GraphEmit (scratch aid).
import sys
import os
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
OPC = {n: i for i, n in enumerate(['naturalConstant','byteConstant','predicateConstant','sequenceConstant','naturalAdd','naturalSubtract','naturalMultiply','naturalDivide','naturalRemainder','naturalAnd','naturalOr','naturalXor','naturalNot','naturalShiftLeft','naturalShiftRight','naturalEqual','naturalLess','byteEqual','byteLess','naturalToByte','byteToNatural','predicateNot','predicateAnd','predicateOr','sequenceLength','sequenceIndex','sequenceSlice','sequenceConcat','sequenceEqual','singletonSequence','basisGet','storeNew','storeRead','storeWrite','choose','apply','tabulate','fold','scan'])}
TC = {'natural': 0, 'byte': 1, 'predicate': 2, 'sequence': 3, 'store': 4}
def uleb(n):
    out = bytearray()
    while True:
        if n < 128: out.append(n); return bytes(out)
        out.append(n % 128 + 128); n //= 128
def instr(t):
    op = t[0]; o = bytearray([OPC[op]])
    if op == 'naturalConstant': o += uleb(t[1])
    elif op == 'byteConstant': o.append(t[1])
    elif op == 'predicateConstant': o.append(1 if t[1] else 0)
    elif op == 'sequenceConstant': o += uleb(len(t[1])) + bytes(t[1])
    elif op == 'apply': o += uleb(t[1]) + uleb(len(t[2])) + b''.join(uleb(x) for x in t[2])
    elif op == 'tabulate': o += uleb(t[1]) + uleb(t[2]) + uleb(len(t[3])) + b''.join(uleb(x) for x in t[3])
    elif op == 'fold': o += uleb(t[1]) + uleb(t[2]) + uleb(t[3]) + uleb(len(t[4])) + b''.join(uleb(x) for x in t[4])
    else: o += b''.join(uleb(x) for x in t[1:])
    return bytes(o)
def graph_bytes(g, entry):
    o = bytearray(uleb(len(g.blocks)))
    for b in g.blocks:
        ins, ret = b.lowered()
        o += uleb(len(b.params)) + bytes(TC[t] for t in b.params) + bytes([TC[b.result]]) + uleb(len(ins)) + b''.join(instr(t) for t in ins) + uleb(ret)
    return bytes(o + uleb(entry.index))
def archive(g, entry, lengths):
    body = graph_bytes(g, entry)
    return b'UORC' + bytes([2, 1, 0, 1]) + uleb(len(lengths)) + b''.join(uleb(n) for n in lengths) + uleb(len(body)) + body
import hashlib
DOM = bytes.fromhex('554f52432d5348413235362d434841494e2d763100')
def chain(data):
    c = hashlib.sha256(DOM + b'\x00').digest()
    full = (len(data) // 64) * 64
    if len(data) % 64 == 0 and len(data) > 0:
        pass
    i = 0
    # absorbByte: a full frame is hashed when the 64th byte arrives
    while len(data) - i >= 64:
        c = hashlib.sha256(DOM + b'\x01' + c + data[i:i + 64]).digest(); i += 64
    rest = data[i:]
    return hashlib.sha256(DOM + b'\x02' + c + bytes([len(rest)]) + rest).digest()
def sealed(pre):
    return pre + chain(pre)
