"""Finite actual-source stream custody controls, never archive membership/proof."""
import hashlib
import json
from pathlib import Path
import diagnostic as e

M = 'Uorc.Search.OwnedEnumeration'
S = 'Uorc.Search.EnumerationStream'
W = 'Uorc.Wire.OwnedRead'
F = 'Uorc.Search.EnumerationFunding'
H = 'Uorc.Core.Heap'
RR = 'Uorc.Core.RuntimeResources'
BB = 'Uorc.Core.RuntimeByteBuilder'
AUDIT = 'Uorc.Core.HeapAudit'
MAX = (1 << 64) - 1
T = e.tag
assert S + '.start' in e.funcs, 'UC-SYN-01 stream custody adapter is not implemented'
# Only the separately source-bound official-reader bundle may supply native IR.
cache_path = Path(__file__).with_name('semantic-cache') / 'wire-native-d27be3e.json'
assert hashlib.sha256(cache_path.read_bytes()).hexdigest() == '58e49bc558d38d0414b69a3e27c0380dfaf0f768fce968cb271a8f5f1637ee88'
cache = json.loads(cache_path.read_text())
assert cache['source_commit'] == 'd27be3e8f7dba73440ec29fa8dba236275162ca7'
assert len(cache['modules']) == 2
assert {module['module'] for module in cache['modules']} == {W, 'Uorc.Wire.FrameEventContract'}
wire = next(module for module in cache['modules'] if module['module'] == W)
assert wire['source_sha256'] == '50b6b1d412b507fb490b177e9faee2e7979d0fbad9c4116e7860ef3a438ef8fc'
assert hashlib.sha256((e.REPO / wire['source_path']).read_bytes()).hexdigest() == wire['source_sha256']
assert wire['repeat_read'] == wire['notation_round_trip'] == 'byte_exact'
assert hashlib.sha256(json.dumps(wire['parsed_module'], ensure_ascii=False,
                                separators=(',', ':')).encode()).hexdigest() == wire['parser_output_sha256']
e.mods[W] = wire['parsed_module']['declarations']
for definition in e.mods[W]:
    if definition['kind'] == 'definition':
        e.funcs[W + '.' + definition['name']] = definition
budget = {'type': RR + '.Budget', 'maxWork': MAX, 'maxOwnedBytes': 1000000,
          'maxOutputBytes': MAX, 'maxFrames': MAX}
ADV = T(S + '.Request.advance')
CANCEL = T(S + '.Request.cancel')


def fixture(data):
    arena = e.call(H, 'emptyArena', budget['maxOwnedBytes'])
    enumeration = e.call(M, 'start')
    allocated = e.call(M, 'perform', arena, enumeration)
    arena, enumeration = allocated['arena'], allocated['cursor']
    if data:
        begun = e.call(BB, 'begin', arena)[1][0]
        arena, builder = begun['arena'], begun['outcome'][1][0]
        for byte in data:
            changed = e.call(BB, 'beginAppendOwned', arena, builder, byte)[1][0]
            while changed['outcome'][0].endswith('.pending'):
                changed = e.call(BB, 'step', changed['arena'], changed['cursor'])[1][0]
            arena, builder = changed['arena'], changed['outcome'][1][0]
        changed = e.call(BB, 'beginFinishOwned', arena, builder)[1][0]
        while changed['outcome'][0].endswith('.pending'):
            changed = e.call(BB, 'step', changed['arena'], changed['cursor'])[1][0]
        arena = changed['arena']
        enumeration = dict(enumeration, current=changed['outcome'][1][0], length=len(data))
    offered = e.call(M, 'perform', arena, enumeration)
    arena, enumeration = offered['arena'], offered['cursor']
    ticket = offered['outcome'][1][0]
    started = e.call(S, 'start', enumeration, ticket)
    assert started[0] == 'Result.ok'
    usage = e.call(F, 'achievedUsage', arena, e.call(RR, 'zeroUsage'))
    assert usage[0] == 'Result.ok'
    return arena, usage[1][0], enumeration, started[1][0]


def fund(arena, usage, enumeration, cursor, request=ADV, credits=MAX, limits=None):
    return e.call(S, 'fund', arena, limits or budget, usage, enumeration, cursor, request, credits)


def audit_custody(arena, enumeration, cursor):
    roots = e.call(M, 'rootInventory', enumeration) + e.call(S, 'rootInventory', cursor)
    assert e.call(AUDIT, 'auditRoots', arena, roots), 'joint producer/stream root-multiset audit failed'


def cancellation(arena, usage, enumeration, cursor):
    changed = fund(arena, usage, enumeration, cursor, CANCEL)
    for _ in range(10000):
        audit_custody(changed['arena'], enumeration, changed['cursor'])
        if changed['outcome'][0].endswith('.cancelled'):
            assert e.call(S, 'rootInventory', changed['cursor']) == []
            assert e.call(S, 'quiescent', changed['cursor'])
            assert e.call(M, 'ticketMatches', enumeration, changed['cursor']['source'])
            return changed
        assert not changed['outcome'][0].endswith('.unresolved'), changed['outcome']
        changed = fund(changed['arena'], changed['usage'], enumeration, changed['cursor'])
    raise AssertionError('stream cancellation did not finish')


def run_controls():
    def terms(value):
        if isinstance(value, dict):
            yield value
            for child in value.values():
                yield from terms(child)
        elif isinstance(value, list):
            for child in value:
                yield from terms(child)
    calls = [term['function'] for declaration in e.mods[S] for term in terms(declaration)
             if term.get('kind') == 'call']
    assert {ref['name'] for ref in calls if ref.get('module') == M} == {'ticketMatches'}, \
        'stream adapter must not acknowledge or advance its enumerator'
    assert {ref['name'] for ref in calls if ref.get('module') == W} == {'begin', 'step'}, \
        'stream adapter must reuse the shared OwnedRead'
    assert {ref['name'] for ref in calls if ref.get('module') == 'Uorc.Core.RuntimeSequence'} <= {
        'addRoot', 'zeroDelta'}, 'stream adapter added an independent sequence traversal'
    prefixes = 0
    spans = 0
    partial_cancellations = 0
    for data in [b'', b'\x00', b'\xff', b'\x00\x00', b'\x00\xff\xff',
                 b'\xff\x00\xff', bytes(range(17)), bytes([0, 255]) * 17]:
        arena, usage, enumeration, cursor = fixture(data)
        original = enumeration
        read = bytearray()
        old_offer = None
        for _ in range(10000):
            audit_custody(arena, enumeration, cursor)
            # Fork every actual operation prefix and drain only adapter custody.
            cancelled = cancellation(arena, usage, enumeration, cursor)
            assert cancelled['arena']['liveEntries'] >= 1  # Producer control remains live.
            assert e.call(H, 'sequenceInfo', cancelled['arena'], enumeration['current'])[0] == 'Result.ok'
            prefixes += 1
            fee = e.call(S, 'requiredWork', cursor)
            if fee:
                zero = fund(arena, usage, enumeration, cursor, credits=0)
                assert (zero['arena'], zero['usage'], zero['cursor']) == (arena, usage, cursor)
                limits = dict(budget, maxWork=usage['work'] + fee - 1)
                refused = fund(arena, usage, enumeration, cursor, credits=fee, limits=limits)
                assert (refused['arena'], refused['usage'], refused['cursor']) == (arena, usage, cursor)
                assert refused['remainingCredits'] == fee and refused['chargedWork'] == 0
                assert refused['outcome'][1][0][0].endswith('.resource')
                full = fund(arena, usage, enumeration, cursor, credits=fee)
                partial = fund(arena, usage, enumeration, cursor, credits=fee - 1)
                split = fund(partial['arena'], partial['usage'], enumeration, partial['cursor'], credits=1)
                assert (full['arena'], full['usage'], full['cursor'], full['outcome']) == (
                    split['arena'], split['usage'], split['cursor'], split['outcome'])
                cancelling = e.call(S, 'cancel', partial['cursor'])
                assert e.call(S, 'cancel', cancelling) == cancelling
                cancellation(partial['arena'], partial['usage'], enumeration, partial['cursor'])
                partial_cancellations += 1
            changed = fund(arena, usage, enumeration, cursor)
            arena, usage, cursor = changed['arena'], changed['usage'], changed['cursor']
            assert enumeration == original
            audit_custody(arena, enumeration, cursor)
            outcome = changed['outcome']
            assert not outcome[0].endswith('.unresolved'), outcome
            if outcome[0].endswith('.offered'):
                offer = outcome[1][0]
                span = offer['span']
                assert span['length'] > 0 and cursor['spanOwned']
                assert e.call(S, 'rootInventory', cursor).count(span['image']) == 1
                entry = e.call(H, 'lookup', arena, span['image'])[1][0]
                assert entry['roots'] >= 1, 'offered image lacks its retained lease root'
                payload = entry['node'][1][1]
                read.extend(payload[span['offset']:span['offset'] + span['length']])
                replay = fund(arena, usage, enumeration, cursor, credits=0)
                assert (replay['arena'], replay['usage'], replay['cursor'], replay['outcome']) == (
                    arena, usage, cursor, outcome)
                bad = dict(offer, owner=dict(offer['owner'], id=offer['owner']['id'] + 1))
                denied = fund(arena, usage, enumeration, cursor, T(S + '.Request.acknowledge', bad))
                assert (denied['arena'], denied['usage'], denied['cursor']) == (arena, usage, cursor)
                assert denied['outcome'][0].endswith('.unresolved')
                assert denied['outcome'][1][0][0].endswith('.invalidOffer')
                if old_offer is not None:
                    denied = fund(arena, usage, enumeration, cursor,
                                  T(S + '.Request.acknowledge', old_offer))
                    assert denied['outcome'][0].endswith('.unresolved')
                    assert denied['outcome'][1][0][0].endswith('.invalidOffer')
                acknowledgment = T(S + '.Request.acknowledge', offer)
                ack_fee = e.call('Uorc.Core.RuntimeWork', 'fee', T('Uorc.Core.RuntimeWork.Operation.acknowledgment'))
                zero = fund(arena, usage, enumeration, cursor, acknowledgment, credits=0)
                assert (zero['arena'], zero['usage'], zero['cursor']) == (arena, usage, cursor)
                denied = fund(arena, usage, enumeration, cursor, acknowledgment, credits=ack_fee,
                              limits=dict(budget, maxWork=usage['work'] + ack_fee - 1))
                assert (denied['arena'], denied['usage'], denied['cursor']) == (arena, usage, cursor)
                assert denied['outcome'][1][0][0].endswith('.resource')
                assert denied['remainingCredits'] == ack_fee and denied['chargedWork'] == 0
                full = fund(arena, usage, enumeration, cursor, acknowledgment, credits=ack_fee)
                partial = fund(arena, usage, enumeration, cursor, acknowledgment, credits=ack_fee - 1)
                split = fund(partial['arena'], partial['usage'], enumeration, partial['cursor'], acknowledgment, credits=1)
                assert (full['arena'], full['usage'], full['cursor'], full['outcome']) == (
                    split['arena'], split['usage'], split['cursor'], split['outcome'])
                # Interleaved replay or rejected correlation does not consume/reset the paid prefix.
                replay = fund(partial['arena'], partial['usage'], enumeration, partial['cursor'], credits=0)
                assert replay['cursor'] == partial['cursor'] and replay['chargedWork'] == 0
                denied = fund(partial['arena'], partial['usage'], enumeration, partial['cursor'],
                              T(S + '.Request.acknowledge', bad), credits=ack_fee)
                assert denied['cursor'] == partial['cursor'] and denied['chargedWork'] == 0
                assert e.call(S, 'cancel', e.call(S, 'cancel', partial['cursor'])) == e.call(S, 'cancel', partial['cursor'])
                cancellation(partial['arena'], partial['usage'], enumeration, partial['cursor'])
                partial_cancellations += 1
                changed = fund(arena, usage, enumeration, cursor,
                               T(S + '.Request.acknowledge', offer))
                arena, usage, cursor = changed['arena'], changed['usage'], changed['cursor']
                old_offer = offer
                spans += 1
            elif outcome[0].endswith('.complete'):
                assert bytes(read) == data
                assert e.call(S, 'rootInventory', cursor) == []
                assert e.call(S, 'quiescent', cursor)
                assert e.call(M, 'ticketMatches', enumeration, cursor['source'])
                break
        else:
            raise AssertionError('stream did not finish')
        # Only now drain the separately still-outstanding producer.
        changed = e.call(F, 'cancel', arena, budget, usage, enumeration, MAX)
        for _ in range(10000):
            if changed['outcome'][0].endswith('.cancelled'):
                break
            assert not changed['outcome'][0].endswith('.unresolved'), changed['outcome']
            changed = e.call(F, 'fund', changed['arena'], budget, changed['usage'],
                             changed['cursor'], T(F + '.Request.advance'), MAX)
        assert changed['arena']['liveEntries'] == changed['arena']['ownedBytes'] == 0
        print('PASS stream bytes', data.hex(), flush=True)
    print('PASS all stream custody prefixes', prefixes, 'spans', spans, flush=True)
    print('PASS funded partial-prepayment cancellation forks', partial_cancellations, flush=True)
    # Actual typed defensive boundaries, with no successful consumer fabricated.
    arena, usage, enumeration, cursor = fixture(b'\x00')
    bad_ticket = dict(cursor['source'], owner=dict(cursor['source']['owner'], id=0))
    assert e.call(S, 'start', enumeration, bad_ticket)[0] == 'Result.error'
    revoked = dict(enumeration, phase=T(M + '.Phase.scanStart'))
    denied = fund(arena, usage, revoked, cursor)
    assert (denied['arena'], denied['usage'], denied['cursor']) == (arena, usage, cursor)
    assert denied['outcome'][1][0][0].endswith('.invalidSource')
    allocated = fund(arena, usage, enumeration, cursor)
    # Defensive observer boundary: a real achieved allocation is supplied with a
    # deliberately mismatched observation budget. Full fund rejects this budget
    # before allocation; this is not a reachable full-fund refusal claim.
    raw = {'type': S + '.RawTransition', **{key: allocated[key] for key in
           ['arena', 'cursor', 'delta', 'outcome']}}
    post_limit = dict(budget, maxOwnedBytes=allocated['arena']['ownedBytes'] - 1)
    post_failed = e.call(S, 'observeResult', post_limit, raw, allocated['usage'],
                         allocated['remainingCredits'], allocated['chargedWork'])
    assert post_failed['outcome'][1][0][0].endswith('.resource')
    assert (post_failed['arena'], post_failed['cursor'], post_failed['delta'], post_failed['usage']) == (
        allocated['arena'], allocated['cursor'], allocated['delta'], allocated['usage'])
    assert post_failed['chargedWork'] == allocated['chargedWork']
    assert post_failed['remainingCredits'] == allocated['remainingCredits']
    audit_custody(post_failed['arena'], enumeration, post_failed['cursor'])
    cancellation(post_failed['arena'], post_failed['usage'], enumeration, post_failed['cursor'])
    print('PASS defensive post-observer refusal retains real achieved allocation and drains', flush=True)
    staged = allocated['cursor']
    broken = dict(staged, source=dict(staged['source'], sequence=staged['control']))
    failed = e.call(S, 'beginRead', allocated['arena'], broken)
    assert failed['outcome'][1][0][0].endswith('.reader')
    no_memory = dict(arena, exhausted=True)
    refused = fund(no_memory, usage, enumeration, cursor)
    assert refused['arena'] == no_memory and refused['cursor'] == cursor
    assert refused['outcome'][1][0][0].endswith('.heap')
    assert refused['usage']['work'] == usage['work'] + e.call(S, 'requiredWork', cursor)
    overpaid = dict(cursor, paid=e.call(S, 'requiredWork', cursor) + 1)
    refused = fund(arena, usage, enumeration, overpaid)
    assert refused['cursor'] == overpaid and refused['chargedWork'] == 0
    assert refused['outcome'][1][0][0].endswith('.invalidFunding')
    following = dict(cursor['reader'], root=cursor['source']['sequence'],
                     length=cursor['source']['length'], offset=0)
    failed = e.call(S, 'borrowedSpan', arena, cursor, cursor['source']['sequence'], 0, 0, following)
    assert failed['outcome'][1][0][0].endswith('.invalidSpan')
    at_limit = dict(cursor, reader=dict(cursor['reader'], offset=MAX))
    failed = e.call(S, 'borrowedSpan', arena, at_limit, cursor['source']['sequence'], 0, 1, following)
    assert failed['outcome'][1][0][0].endswith('.addressSpaceExhausted')
    assert failed['arena'] == arena and failed['cursor'] == at_limit
    print('PASS typed source, reader, resource, heap, offer, funding, span and overflow boundaries')



if __name__ == '__main__':
    import sys
    if not __debug__:
        raise SystemExit('optimized Python disables stream assertions')
    if len(sys.argv) != 1:
        raise SystemExit('stream controls accept no filters or arguments')
    run_controls()
