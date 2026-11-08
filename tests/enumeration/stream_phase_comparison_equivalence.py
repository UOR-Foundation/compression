#!/usr/bin/env python3
"""Focused exact-source Phase repair diagnostics, never compiler acceptance.

Run without arguments or optimized Python. The frozen source checkpoint and the
current EnumerationStream are compared as complete canonical modules. Only nine
scalar equality predicates may become exhaustive source-owned Phase matches.
The source evaluator supplies finite diagnostic evidence, not SDK elaboration,
native execution, kernel proof, resource bounds, or controller qualification.
"""
import ast
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[2]
BASELINE = '134db36039927f559b43caa28f744c8338699857'
MODULE = 'Uorc.Search.EnumerationStream'
SOURCE = 'src/Uorc/Search/EnumerationStream.lex.tex'
PHASES = (
    'allocateControl', 'beginRead', 'read', 'retainSpan', 'offered',
    'releaseSpan', 'drainSpan', 'releaseControl', 'drainControl',
    'cancelSelect', 'complete', 'cancelled',
)
SITES = (
    ('offerMatches', ('body', 'left'), 'offered'),
    ('viewOutcome', ('body', 'condition'), 'offered'),
    ('viewOutcome', ('body', 'else_value', 'condition'), 'complete'),
    ('viewOutcome', ('body', 'else_value', 'else_value', 'condition'), 'cancelled'),
    ('quiescent', ('body', 'left', 'left'), 'complete'),
    ('quiescent', ('body', 'left', 'right'), 'cancelled'),
    ('cancel', ('body', 'condition', 'right'), 'cancelled'),
    ('cancel', ('body', 'else_value', 'fields', 1, 'value', 'condition'), 'drainSpan'),
    ('cancel', ('body', 'else_value', 'fields', 1, 'value', 'else_value', 'condition'),
     'drainControl'),
)
PRODUCER = 'Uorc.Search.OwnedEnumeration'
FUNDING = 'Uorc.Search.EnumerationFunding'
HEAP = 'Uorc.Core.Heap'
RESOURCES = 'Uorc.Core.RuntimeResources'
VALUE = 'Uorc.Core.RuntimeValue'
SEQUENCE = 'Uorc.Core.RuntimeSequence'
BUILDER = 'Uorc.Core.RuntimeByteBuilder'
AUDIT = 'Uorc.Core.HeapAudit'
WIRE = 'Uorc.Wire.OwnedRead'
MAX = (1 << 64) - 1
PROGRESS = Path(tempfile.gettempdir()) / 'enumeration-stream-phase-comparison-progress.jsonl'
STARTED = time.monotonic()
FIXTURES = (b'', b'\x00', b'\xff', b'\x00\x00', b'\x00\xff\xff',
            b'\xff\x00\xff', bytes(range(17)), bytes([0, 255]) * 17)



def require(condition, message):
    if not condition:
        raise ValueError(message)


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':')).encode()


def sha(value):
    return hashlib.sha256(value).hexdigest()


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, 'duplicate JSON key: ' + key)
        result[key] = value
    return result


def parse(data):
    text = data.decode('utf-8')
    marker = r'\semanticdata{'
    require(text.count(marker) == 1, 'expected exactly one semanticdata form')
    require(text.startswith(r'\begin{lexlean}{' + MODULE + '}\n'),
            'wrong module header')
    start = text.index(marker) + len(marker)
    node, length = json.JSONDecoder(object_pairs_hook=unique_object).raw_decode(text[start:])
    require(text[start:start + length].encode() == canonical(node),
            'noncanonical semantic JSON')
    require(node.get('spec') == 'lexlean/semantic-module/1', 'wrong semantic schema')
    names = [d.get('name') for d in node['declarations']]
    require(all(isinstance(n, str) for n in names) and len(set(names)) == len(names),
            'missing or duplicate declaration')
    return {'raw': data, 'ast': node, 'prefix': text[:start].encode(),
            'suffix': text[start + length:].encode()}


def declarations(module):
    return {d['name']: d for d in module['declarations']}


def at(node, path):
    for key in path:
        node = node[key]
    return node


def replace(module, name, path, value):
    node = declarations(module)[name]
    at(node, path[:-1])[path[-1]] = copy.deepcopy(value)


def phase_project():
    return {'kind': 'project', 'field': 'phase',
            'value': {'kind': 'var', 'name': 'cursor'}}


def primitive(target):
    return {'kind': 'primitive', 'operation': 'equal', 'result': {'kind': 'bool'},
            'arguments': [phase_project(), {'kind': 'constructor',
                          'constructor': {'name': 'Phase.' + target}, 'arguments': []}]}


def predicate(target):
    return {'kind': 'match', 'scrutinee': phase_project(), 'branches': [
        {'constructor': {'name': 'Phase.' + name}, 'binders': [],
         'body': {'kind': 'bool', 'value': name == target}} for name in PHASES]}


def walk(node):
    if isinstance(node, dict):
        yield node
        for child in node.values():
            yield from walk(child)
    elif isinstance(node, list):
        for child in node:
            yield from walk(child)


def check_shapes(module):
    ds = declarations(module)
    require(ds['Phase'] == {'kind': 'inductive', 'name': 'Phase',
            'constructors': [{'name': n, 'fields': []} for n in PHASES],
            'parameters': [], 'type_parameters': []}, 'Phase domain/payload shape changed')
    require(next(f['type'] for f in ds['Cursor']['fields'] if f['name'] == 'phase') ==
            {'kind': 'named', 'member': {'name': 'Phase'}, 'arguments': []},
            'cursor.phase is not the exact source Phase type')
    for name, path, target in SITES:
        node = at(ds[name], path)
        require(node.get('kind') == 'match', name + '/' + target + ': expected Phase match')
        require(set(node) == {'kind', 'scrutinee', 'branches'} and
                node['scrutinee'] == phase_project(), 'wrong Phase match scrutinee')
        branches = node['branches']
        require(len(branches) == len(PHASES) and
                [b.get('constructor') for b in branches] ==
                [{'name': 'Phase.' + n} for n in PHASES],
                'missing, duplicate, reordered or foreign Phase match branch')
        for branch in branches:
            require(set(branch) == {'constructor', 'binders', 'body'} and
                    branch['binders'] == [] and set(branch['body']) == {'kind', 'value'} and
                    branch['body']['kind'] == 'bool' and
                    isinstance(branch['body']['value'], bool),
                    'wrong Phase match branch type or binder')
    bad = [n for n in walk(module) if n.get('kind') == 'primitive' and
           n.get('operation') == 'equal' and any(
               x.get('kind') == 'constructor' and
               x.get('constructor', {}).get('name', '').startswith('Phase.')
               for x in n.get('arguments', []))]
    require(not bad, 'source still uses scalar equal on Phase')


def check_exact(before, after):
    expected = copy.deepcopy(before['ast'])
    old = declarations(expected)
    for name, path, target in SITES:
        require(at(old[name], path) == primitive(target), 'unexpected frozen comparison site')
        replace(expected, name, path, predicate(target))
    require(after['prefix'] == before['prefix'] and after['suffix'] == before['suffix'],
            'non-JSON source bytes changed')
    require(after['ast'] == expected, 'module differs beyond the exact nine Phase predicates')
    require(after['raw'] == before['prefix'] + canonical(expected) + before['suffix'],
            'source bytes differ beyond exact Phase predicate substitutions')
    check_shapes(after['ast'])


def progress(stage, **details):
    # Scratch telemetry is not a success result or part of the receipt.
    with PROGRESS.open('a') as log:
        log.write(json.dumps({'elapsed_seconds': round(time.monotonic() - STARTED, 3),
                              'stage': stage, **details}, sort_keys=True) + '\n')


def check_dispatch(evaluator):
    expected = {MODULE + '.' + d['name']: d for d in evaluator.mods[MODULE]
                if d['kind'] == 'definition'}
    actual = {name: definition for name, definition in evaluator.funcs.items()
              if name.startswith(MODULE + '.')}
    require(actual == expected, 'Stream module/function dispatch inventory is not exact')
    return len(expected)


def install(evaluator, module):
    # Startup establishes exact dispatch coherence. Remove the entire previous
    # Stream definition inventory, then install the entire selected inventory.
    # This preserves nested dispatch without scanning every unrelated function.
    previous = evaluator.mods[MODULE]
    for definition in previous:
        if definition['kind'] == 'definition':
            del evaluator.funcs[MODULE + '.' + definition['name']]
    evaluator.mods[MODULE] = module['declarations']
    evaluator.funcs.update({MODULE + '.' + d['name']: d for d in module['declarations']
                            if d['kind'] == 'definition'})


def evaluate(evaluator, module, name, *arguments):
    original = evaluator.mods[MODULE]
    try:
        install(evaluator, module)
        return evaluator.call(MODULE, name, *arguments)
    finally:
        install(evaluator, {'declarations': original})


def sample_cursor(e):
    handle = lambda identifier: {'type': VALUE + '.Handle', 'id': identifier}
    return {'type': MODULE + '.Cursor',
            'source': {'type': PRODUCER + '.Ticket', 'owner': handle(17),
                       'sequence': handle(23), 'length': 29},
            'phase': e.tag(MODULE + '.Phase.offered'), 'control': handle(31),
            'reader': e.call(MODULE, 'emptyReader'),
            'span': {'type': SEQUENCE + '.Span', 'image': handle(37),
                     'offset': 41, 'length': 43},
            'spanOwned': True, 'offeredPosition': 47, 'cancelling': False, 'paid': 53}


def bad_offers(offer):
    result = [('exact', offer)]
    for path in (('owner', 'id'), ('source', 'owner', 'id'),
                 ('source', 'sequence', 'id'), ('source', 'length'), ('position',),
                 ('span', 'image', 'id'), ('span', 'offset'), ('span', 'length')):
        wrong = copy.deepcopy(offer)
        at(wrong, path[:-1])[path[-1]] += 1
        result.append(('/'.join(path), wrong))
    return result


def check_semantics(evaluator, old, new):
    cursor = sample_cursor(evaluator)
    T = evaluator.tag
    old_ds, new_ds = declarations(old), declarations(new)
    predicate_cases = offer_cases = cancellation_cases = outcome_cases = quiescent_cases = 0
    for phase in PHASES:
        phase_value = T(MODULE + '.Phase.' + phase)
        for name, path, target in SITES:
            environment = {'cursor': dict(cursor, phase=phase_value)}
            before = evaluator.ev(at(old_ds[name], path), MODULE, environment)
            after = evaluator.ev(at(new_ds[name], path), MODULE, environment)
            require(after is (phase == target) and before == after,
                    'Phase predicate semantic mismatch: ' + name + '/' + target + '/' + phase)
            predicate_cases += 1
        for owned in (False, True):
            for cancelling in (False, True):
                c = dict(cursor, phase=phase_value, spanOwned=owned, cancelling=cancelling)
                offer = evaluate(evaluator, new, 'offer', c)
                for label, presented in bad_offers(offer):
                    good = phase == 'offered' and owned and not cancelling and label == 'exact'
                    before = evaluate(evaluator, old, 'offerMatches', c, presented)
                    after = evaluate(evaluator, new, 'offerMatches', c, presented)
                    require(before == after and after is good,
                            'offer correlation mismatch: ' + phase + '/' + label)
                    before = evaluate(evaluator, old, 'acknowledge', c, presented)
                    after = evaluate(evaluator, new, 'acknowledge', c, presented)
                    expected = (T('Result.ok', dict(c, phase=T(MODULE + '.Phase.releaseSpan')))
                                if good else T('Result.error', T(MODULE + '.Error.invalidOffer')))
                    require(before == after == expected, 'acknowledge custody/preservation mismatch')
                    offer_cases += 1
                for control in (0, 31):
                    for paid in (0, 53):
                        state = dict(c, control=dict(c['control'], id=control), paid=paid)
                        before = evaluate(evaluator, old, 'cancel', state)
                        after = evaluate(evaluator, new, 'cancel', state)
                        preserve = cancelling or phase == 'cancelled'
                        next_phase = phase if phase in ('drainSpan', 'drainControl') else 'cancelSelect'
                        expected = state if preserve else dict(state,
                            phase=T(MODULE + '.Phase.' + next_phase), cancelling=True, paid=0)
                        require(before == after == expected, 'cancel custody/prepayment mismatch')
                        require(evaluate(evaluator, new, 'cancel', after) == after,
                                'cancel is not idempotent')
                        cancellation_cases += 1
                        before = evaluate(evaluator, old, 'viewOutcome', state)
                        after = evaluate(evaluator, new, 'viewOutcome', state)
                        expected = (T(MODULE + '.Outcome.offered', evaluate(evaluator, new, 'offer', state))
                                    if phase == 'offered' else T(MODULE + '.Outcome.' +
                                    (phase if phase in ('complete', 'cancelled') else 'pending')))
                        require(before == after == expected, 'viewOutcome constructor mismatch')
                        outcome_cases += 1
                        before = evaluate(evaluator, old, 'quiescent', state)
                        after = evaluate(evaluator, new, 'quiescent', state)
                        expected = phase in ('complete', 'cancelled') and control == 0 and not owned and paid == 0
                        require(before == after and after is expected, 'quiescent custody mismatch')
                        quiescent_cases += 1
    require((predicate_cases, offer_cases, cancellation_cases, outcome_cases, quiescent_cases) ==
            (108, 432, 192, 192, 192), 'missing or duplicated exhaustive semantic cases')
    return {'predicate_constructor_cases': predicate_cases, 'offer_and_acknowledge_cases': offer_cases,
            'cancel_constructor_flag_cases': cancellation_cases, 'view_outcome_cases': outcome_cases,
            'quiescence_cases': quiescent_cases}


def load_dependencies(e):
    path = ROOT / 'tests/enumeration/semantic-cache/wire-native-d27be3e.json'
    require(sha(path.read_bytes()) == '58e49bc558d38d0414b69a3e27c0380dfaf0f768fce968cb271a8f5f1637ee88',
            'wrong native reader cache')
    cache = json.loads(path.read_text(), object_pairs_hook=unique_object)
    require(cache['source_commit'] == 'd27be3e8f7dba73440ec29fa8dba236275162ca7' and
            {m['module'] for m in cache['modules']} == {WIRE, 'Uorc.Wire.FrameEventContract'} and
            len(cache['modules']) == 2, 'wrong native reader cache inventory')
    wire = next(m for m in cache['modules'] if m['module'] == WIRE)
    require(sha((ROOT / wire['source_path']).read_bytes()) == wire['source_sha256'] ==
            '50b6b1d412b507fb490b177e9faee2e7979d0fbad9c4116e7860ef3a438ef8fc',
            'native reader source changed')
    require(wire['repeat_read'] == wire['notation_round_trip'] == 'byte_exact' and
            sha(json.dumps(wire['parsed_module'], ensure_ascii=False, separators=(',', ':')).encode()) ==
            wire['parser_output_sha256'], 'native reader parsed IR mismatch')
    e.mods[WIRE] = wire['parsed_module']['declarations']
    for definition in e.mods[WIRE]:
        if definition['kind'] == 'definition':
            e.funcs[WIRE + '.' + definition['name']] = definition
    return {'cache_path': str(path.relative_to(ROOT)), 'cache_sha256': sha(path.read_bytes()),
            'evaluated_module': WIRE, 'source_sha256': wire['source_sha256'],
            'parser_output_sha256': wire['parser_output_sha256'],
            'reuses_recorded_parser_evidence_only': True}


def fixture(e, data, budget):
    arena = e.call(HEAP, 'emptyArena', budget['maxOwnedBytes'])
    enumeration = e.call(PRODUCER, 'start')
    changed = e.call(PRODUCER, 'perform', arena, enumeration)
    arena, enumeration = changed['arena'], changed['cursor']
    if data:
        begun = e.call(BUILDER, 'begin', arena)
        require(begun[0] == 'Result.ok', 'fixture builder begin failed')
        arena, builder = begun[1][0]['arena'], begun[1][0]['outcome'][1][0]
        for byte in data:
            changed = e.call(BUILDER, 'beginAppendOwned', arena, builder, byte)[1][0]
            for _ in range(10000):
                if not changed['outcome'][0].endswith('.pending'):
                    break
                changed = e.call(BUILDER, 'step', changed['arena'], changed['cursor'])[1][0]
            else:
                raise ValueError('fixture builder append did not finish')
            arena, builder = changed['arena'], changed['outcome'][1][0]
        changed = e.call(BUILDER, 'beginFinishOwned', arena, builder)[1][0]
        for _ in range(10000):
            if not changed['outcome'][0].endswith('.pending'):
                break
            changed = e.call(BUILDER, 'step', changed['arena'], changed['cursor'])[1][0]
        else:
            raise ValueError('fixture builder finish did not finish')
        arena = changed['arena']
        enumeration = dict(enumeration, current=changed['outcome'][1][0], length=len(data))
    changed = e.call(PRODUCER, 'perform', arena, enumeration)
    arena, enumeration = changed['arena'], changed['cursor']
    started = e.call(MODULE, 'start', enumeration, changed['outcome'][1][0])
    require(started[0] == 'Result.ok', 'actual fixture Stream start failed')
    usage = e.call(FUNDING, 'achievedUsage', arena, e.call(RESOURCES, 'zeroUsage'))
    require(usage[0] == 'Result.ok', 'actual fixture usage failed')
    return arena, usage[1][0], enumeration, started[1][0]


def state_bytes(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'),
                      default=lambda b: {'bytes_hex': b.hex()}).encode()


def run_paths(e, module):
    original = e.mods[MODULE]
    install(e, module)
    try:
        return actual_paths(e)
    finally:
        install(e, {'declarations': original})


def actual_paths(e):
    T = e.tag
    budget = {'type': RESOURCES + '.Budget', 'maxWork': MAX, 'maxOwnedBytes': 1000000,
              'maxOutputBytes': MAX, 'maxFrames': MAX}
    ADV, CANCEL = T(MODULE + '.Request.advance'), T(MODULE + '.Request.cancel')
    zero_delta = e.call(SEQUENCE, 'zeroDelta')
    counts = {'fund_calls': 0, 'custody_audits': 0, 'operation_prefixes': 0, 'offers': 0,
              'partial_cancellations': 0, 'cancel_requests': 0, 'ack_requests': 0,
              'zero_credit_checks': 0, 'resource_refusals': 0, 'split_payment_checks': 0,
              'invalid_offer_checks': 0, 'invalid_source_checks': 0, 'terminal_replays': 0,
              'repeated_funded_cancel_prepayment_checks': 0}
    phases = set()
    transcript = hashlib.sha256()

    def audit(arena, enumeration, cursor):
        roots = e.call(PRODUCER, 'rootInventory', enumeration) + e.call(MODULE, 'rootInventory', cursor)
        require(e.call(AUDIT, 'auditRoots', arena, roots), 'joint producer/stream custody audit failed')
        counts['custody_audits'] += 1
        phases.add(cursor['phase'][0].rsplit('.', 1)[-1])

    def fund(arena, usage, enumeration, cursor, request=ADV, credits=MAX, limits=None):
        input_bytes = state_bytes((arena, usage, enumeration, cursor, request, credits, limits))
        changed = e.call(MODULE, 'fund', arena, limits or budget, usage,
                         enumeration, cursor, request, credits)
        require(input_bytes == state_bytes((arena, usage, enumeration, cursor, request, credits, limits)),
                'fund mutated its input or producer ticket')
        require(changed['chargedWork'] == changed['usage']['work'] - usage['work'] and
                changed['remainingCredits'] == credits - changed['chargedWork'],
                'charged work accounting mismatch')
        audit(changed['arena'], enumeration, changed['cursor'])
        transcript.update(input_bytes + b'\n' + state_bytes(changed) + b'\n')
        counts['fund_calls'] += 1
        counts['cancel_requests'] += request == CANCEL
        counts['ack_requests'] += request[0].endswith('.acknowledge')
        return changed

    def unchanged(changed, arena, usage, cursor, credits, label):
        require((changed['arena'], changed['usage'], changed['cursor'], changed['remainingCredits'],
                 changed['chargedWork'], changed['delta']) ==
                (arena, usage, cursor, credits, 0, zero_delta), label)

    def cancellation(arena, usage, enumeration, cursor):
        changed = fund(arena, usage, enumeration, cursor, CANCEL)
        for _ in range(10000):
            audit(changed['arena'], enumeration, changed['cursor'])
            if changed['outcome'][0].endswith('.cancelled'):
                require(e.call(MODULE, 'rootInventory', changed['cursor']) == [] and
                        e.call(MODULE, 'quiescent', changed['cursor']), 'cancel did not drain Stream custody')
                require(e.call(PRODUCER, 'ticketMatches', enumeration, changed['cursor']['source']),
                        'cancel revoked producer custody')
                replay = fund(changed['arena'], changed['usage'], enumeration, changed['cursor'], CANCEL)
                unchanged(replay, changed['arena'], changed['usage'], changed['cursor'], MAX,
                          'terminal cancellation replay changed state')
                counts['terminal_replays'] += 1
                return changed
            require(not changed['outcome'][0].endswith('.unresolved'), 'actual cancellation became unresolved')
            # Repeated cancellation preserves an already-cancelling partial payment.
            fee = e.call(MODULE, 'requiredWork', changed['cursor'])
            if fee:
                partial = fund(changed['arena'], changed['usage'], enumeration, changed['cursor'], credits=fee - 1)
                require(e.call(MODULE, 'cancel', partial['cursor']) == partial['cursor'],
                        'repeated cancellation discarded partial prepayment')
                replay = fund(partial['arena'], partial['usage'], enumeration, partial['cursor'],
                              CANCEL, credits=0)
                unchanged(replay, partial['arena'], partial['usage'], partial['cursor'], 0,
                          'repeated funded cancellation discarded partial prepayment')
                counts['repeated_funded_cancel_prepayment_checks'] += 1
            changed = fund(changed['arena'], changed['usage'], enumeration, changed['cursor'])
        raise ValueError('actual cancellation did not finish')

    for data in FIXTURES:
        progress('full-path-fixture-start', fixture_hex=data.hex(), fund_calls=counts['fund_calls'])
        arena, usage, enumeration, cursor = fixture(e, data, budget)
        initial_enumeration = copy.deepcopy(enumeration)
        read = bytearray()
        old_offer = None
        for _ in range(10000):
            audit(arena, enumeration, cursor)
            cancelled = cancellation(arena, usage, enumeration, cursor)
            require(cancelled['arena']['liveEntries'] >= 1 and
                    e.call(HEAP, 'sequenceInfo', cancelled['arena'], enumeration['current'])[0] == 'Result.ok',
                    'cancellation released producer sequence/control')
            counts['operation_prefixes'] += 1
            fee = e.call(MODULE, 'requiredWork', cursor)
            if fee:
                zero = fund(arena, usage, enumeration, cursor, credits=0)
                unchanged(zero, arena, usage, cursor, 0, 'unpaid stream event executed')
                counts['zero_credit_checks'] += 1
                denied = fund(arena, usage, enumeration, cursor, credits=fee,
                              limits=dict(budget, maxWork=usage['work'] + fee - 1))
                unchanged(denied, arena, usage, cursor, fee, 'resource refusal changed unexecuted prefix')
                require(denied['outcome'][1][0][0].endswith('.resource'), 'wrong resource refusal')
                counts['resource_refusals'] += 1
                full = fund(arena, usage, enumeration, cursor, credits=fee)
                partial = fund(arena, usage, enumeration, cursor, credits=fee - 1)
                split = fund(partial['arena'], partial['usage'], enumeration, partial['cursor'], credits=1)
                require(tuple(full[k] for k in ('arena', 'usage', 'cursor', 'outcome')) ==
                        tuple(split[k] for k in ('arena', 'usage', 'cursor', 'outcome')) and
                        full['chargedWork'] == partial['chargedWork'] + split['chargedWork'],
                        'split advance payment changed execution or total charge')
                counts['split_payment_checks'] += 1
                cancellation(partial['arena'], partial['usage'], enumeration, partial['cursor'])
                counts['partial_cancellations'] += 1
            changed = fund(arena, usage, enumeration, cursor)
            arena, usage, cursor = changed['arena'], changed['usage'], changed['cursor']
            require(enumeration == initial_enumeration, 'Stream advanced the producer')
            outcome = changed['outcome']
            require(not outcome[0].endswith('.unresolved'), 'actual Stream path became unresolved')
            if outcome[0].endswith('.offered'):
                offer = outcome[1][0]
                span = offer['span']
                require(span['length'] > 0 and cursor['spanOwned'] and
                        e.call(MODULE, 'rootInventory', cursor).count(span['image']) == 1,
                        'offered span has no unique Stream custody root')
                entry = e.call(HEAP, 'lookup', arena, span['image'])[1][0]
                require(entry['roots'] >= 1, 'offered image lacks its retained lease root')
                payload = entry['node'][1][1]
                read.extend(payload[span['offset']:span['offset'] + span['length']])
                replay = fund(arena, usage, enumeration, cursor, credits=0)
                unchanged(replay, arena, usage, cursor, 0, 'offered replay changed custody or charge')
                require(replay['outcome'] == outcome, 'offered replay changed offer')
                counts['zero_credit_checks'] += 1
                invalid = bad_offers(offer)[1:]
                if old_offer is not None:
                    invalid.append(('previous-offer', old_offer))
                for _, bad in invalid:
                    denied = fund(arena, usage, enumeration, cursor,
                                  T(MODULE + '.Request.acknowledge', bad))
                    unchanged(denied, arena, usage, cursor, MAX, 'invalid offer consumed custody or credit')
                    require(denied['outcome'][1][0][0].endswith('.invalidOffer'), 'invalid offer accepted')
                    counts['invalid_offer_checks'] += 1
                ack = T(MODULE + '.Request.acknowledge', offer)
                ack_fee = e.call('Uorc.Core.RuntimeWork', 'fee',
                                 T('Uorc.Core.RuntimeWork.Operation.acknowledgment'))
                zero = fund(arena, usage, enumeration, cursor, ack, credits=0)
                unchanged(zero, arena, usage, cursor, 0, 'unpaid acknowledgment executed')
                counts['zero_credit_checks'] += 1
                denied = fund(arena, usage, enumeration, cursor, ack, credits=ack_fee,
                              limits=dict(budget, maxWork=usage['work'] + ack_fee - 1))
                unchanged(denied, arena, usage, cursor, ack_fee, 'refused acknowledgment changed custody')
                require(denied['outcome'][1][0][0].endswith('.resource'), 'wrong acknowledgment refusal')
                counts['resource_refusals'] += 1
                full = fund(arena, usage, enumeration, cursor, ack, credits=ack_fee)
                require(full['cursor'] == dict(cursor, phase=T(MODULE + '.Phase.releaseSpan')) and
                        full['arena'] == arena and full['delta'] == zero_delta,
                        'acknowledgment changed custody before release')
                partial = fund(arena, usage, enumeration, cursor, ack, credits=ack_fee - 1)
                split = fund(partial['arena'], partial['usage'], enumeration, partial['cursor'], ack, credits=1)
                require(tuple(full[k] for k in ('arena', 'usage', 'cursor', 'outcome')) ==
                        tuple(split[k] for k in ('arena', 'usage', 'cursor', 'outcome')) and
                        full['chargedWork'] == partial['chargedWork'] + split['chargedWork'],
                        'split acknowledgment changed execution or total charge')
                counts['split_payment_checks'] += 1
                replay = fund(partial['arena'], partial['usage'], enumeration, partial['cursor'], credits=0)
                unchanged(replay, partial['arena'], partial['usage'], partial['cursor'], 0,
                          'replay discarded acknowledgment prepayment')
                denied = fund(partial['arena'], partial['usage'], enumeration, partial['cursor'],
                              T(MODULE + '.Request.acknowledge', invalid[0][1]))
                unchanged(denied, partial['arena'], partial['usage'], partial['cursor'], MAX,
                          'invalid offer discarded acknowledgment prepayment')
                cancellation(partial['arena'], partial['usage'], enumeration, partial['cursor'])
                counts['partial_cancellations'] += 1
                changed = fund(arena, usage, enumeration, cursor, ack)
                arena, usage, cursor = changed['arena'], changed['usage'], changed['cursor']
                old_offer = offer
                counts['offers'] += 1
            elif outcome[0].endswith('.complete'):
                require(bytes(read) == data, 'Stream bytes changed')
                require(e.call(MODULE, 'rootInventory', cursor) == [] and e.call(MODULE, 'quiescent', cursor),
                        'complete retained Stream custody')
                require(e.call(PRODUCER, 'ticketMatches', enumeration, cursor['source']),
                        'complete revoked producer ticket')
                replay = fund(arena, usage, enumeration, cursor, credits=0)
                unchanged(replay, arena, usage, cursor, 0, 'complete replay changed state')
                require(replay['outcome'] == outcome, 'complete replay changed outcome')
                counts['terminal_replays'] += 1
                break
        else:
            raise ValueError('actual Stream did not finish')
        # Drain the still-independent producer only after adapter completion.
        changed = e.call(FUNDING, 'cancel', arena, budget, usage, enumeration, MAX)
        for _ in range(10000):
            if changed['outcome'][0].endswith('.cancelled'):
                break
            require(not changed['outcome'][0].endswith('.unresolved'), 'producer cleanup failed')
            changed = e.call(FUNDING, 'fund', changed['arena'], budget, changed['usage'],
                             changed['cursor'], T(FUNDING + '.Request.advance'), MAX)
        else:
            raise ValueError('producer cleanup did not finish')
        require(changed['arena']['liveEntries'] == changed['arena']['ownedBytes'] == 0,
                'independent producer cleanup leaked custody')
    arena, usage, enumeration, cursor = fixture(e, b'\x00', budget)
    revoked = dict(enumeration, phase=T(PRODUCER + '.Phase.scanStart'))
    for request in (ADV, T(MODULE + '.Request.acknowledge', e.call(MODULE, 'offer', cursor))):
        denied = fund(arena, usage, revoked, cursor, request)
        unchanged(denied, arena, usage, cursor, MAX, 'invalid source consumed custody or credit')
        require(denied['outcome'][1][0][0].endswith('.invalidSource'), 'invalid source accepted')
        counts['invalid_source_checks'] += 1
    require((counts['operation_prefixes'], counts['offers'], counts['partial_cancellations']) ==
            (111, 11, 114) and counts['repeated_funded_cancel_prepayment_checks'] > 0,
            'actual full-path case inventory incomplete')
    require(phases == set(PHASES), 'actual full-path phase inventory incomplete: ' + str(sorted(phases)))
    progress('full-path-pass', counts=counts)
    return {'fixtures_hex': [data.hex() for data in FIXTURES], 'counts': counts,
            'visited_phases': sorted(phases), 'full_input_output_transcript_sha256': transcript.hexdigest()}
def negative_controls(evaluator, before, after, expected_paths):
    rejected = []

    def reject(label, expected, gate, mutate, full_restore=False):
        progress('mutant-start', mutant=label)
        node = copy.deepcopy(after['ast'])
        mutate(node)
        try:
            gate(node)
        except ValueError as error:
            require(expected in str(error), 'wrong failure for ' + label + ': ' + str(error))
            rejected.append({'defect': label, 'diagnostic': str(error)})
        else:
            raise ValueError('planted real-source-IR defect escaped: ' + label)
        check_shapes(after['ast'])
        check_semantics(evaluator, before['ast'], after['ast'])
        if full_restore:
            progress('mutant-full-path-restore-start', mutant=label)
            require(run_paths(evaluator, after['ast']) == expected_paths,
                    'restored full-path evidence differs after ' + label)
        progress('mutant-rejected-and-restored', mutant=label)

    for name, path, target in SITES:
        label = name + '/' + target
        reject('scalar-equal-restored/' + label, 'expected Phase match', check_shapes,
               lambda n, name=name, path=path, target=target:
               replace(n, name, path, primitive(target)))
        reject('omit-Phase-branch/' + label, 'Phase match branch', check_shapes,
               lambda n, name=name, path=path:
               at(declarations(n)[name], path)['branches'].pop())
        reject('duplicate-Phase-branch/' + label, 'Phase match branch', check_shapes,
               lambda n, name=name, path=path:
               at(declarations(n)[name], path)['branches'].__setitem__(
                   0, copy.deepcopy(at(declarations(n)[name], path)['branches'][1])))
        for wrong_phase in (target, next(p for p in PHASES if p != target)):
            reject('wrong-truth-value/' + label + '/' + wrong_phase,
                   'Phase predicate semantic mismatch',
                   lambda n: check_semantics(evaluator, before['ast'], n),
                   lambda n, name=name, path=path, wrong_phase=wrong_phase:
                   at(declarations(n)[name], path)['branches'][PHASES.index(wrong_phase)]
                   ['body'].update(value=wrong_phase != target))
    reject('add-unhandled-Phase-constructor', 'Phase domain/payload shape changed', check_shapes,
           lambda n: declarations(n)['Phase']['constructors'].append({'name': 'extra', 'fields': []}))
    reject('add-Phase-payload', 'Phase domain/payload shape changed', check_shapes,
           lambda n: declarations(n)['Phase']['constructors'][0]['fields'].append({'kind': 'bool'}))
    reject('change-Phase-scrutinee', 'wrong Phase match scrutinee', check_shapes,
           lambda n: at(declarations(n)['offerMatches'], SITES[0][1])['scrutinee'].update(field='source'))
    exact = lambda n: check_exact(before, dict(after, ast=n,
                         raw=after['prefix'] + canonical(n) + after['suffix']))
    reject('change-unrelated-acknowledge-binder', 'module differs beyond', exact,
           lambda n: declarations(n)['acknowledge']['parameters'][1].update(name='ticket'))
    reject('change-unrelated-cancel-prepayment', 'module differs beyond', exact,
           lambda n: next(f for f in declarations(n)['cancel']['body']['else_value']['fields']
                         if f['field'] == 'paid')['value'].update(value='1'))

    def unpaid(node):
        tests = [x for x in walk(declarations(node)['pay']['body'])
                 if x.get('kind') == 'if' and x['condition'].get('kind') == 'primitive'
                 and x['condition'].get('operation') == 'equal']
        require(len(tests) == 1, 'unpaid mutation has ambiguous target')
        tests[0]['condition'] = {'kind': 'bool', 'value': True}

    def missing_lease(node):
        calls = [x for x in walk(declarations(node)['retainSpan']['body'])
                 if x.get('kind') == 'call' and x.get('function') == {'module': HEAP, 'name': 'retain'}]
        require(len(calls) == 1, 'lease mutation has ambiguous target')
        calls[0]['arguments'][1] = {'kind': 'call', 'function': {'module': VALUE, 'name': 'nullHandle'},
                                   'arguments': []}

    def wrong_charge(node):
        fields = [x for x in walk(declarations(node)['pay']['body'])
                  if x.get('field') == 'chargedWork' and x.get('value') == {'kind': 'var', 'name': 'taken'}]
        require(len(fields) == 1, 'charge mutation has ambiguous target')
        fields[0]['value'] = {'kind': 'integer', 'representation': 'uint64', 'value': '0'}

    reject('execute-unpaid-event', 'unpaid stream event executed',
           lambda n: run_paths(evaluator, n), unpaid, full_restore=True)
    reject('omit-image-retain', 'joint producer/stream custody audit failed',
           lambda n: run_paths(evaluator, n), missing_lease, full_restore=True)
    reject('erase-partial-charge-receipt', 'charged work accounting mismatch',
           lambda n: run_paths(evaluator, n), wrong_charge, full_restore=True)
    require(len(rejected) == 53, 'missing or duplicated intended mutant inventory')
    return rejected


def frozen_inputs():
    # Bind all source/model files plus the evaluator, cache, old diagnostics and
    # their receipts. No old receipt is rewritten as evidence for this patch.
    paths = ['src', 'model', 'CONFORMANCE.md', 'prismpm.lock', 'template.lock',
             'tests/enumeration', 'tests/heap/semantic-cache',
             'docs/implementation']
    entries = subprocess.run(['git', '-C', str(ROOT), 'ls-tree', '-rz', BASELINE, '--', *paths],
                             check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE).stdout
    manifest = {}
    for entry in entries.split(b'\0'):
        if not entry:
            continue
        info, path = entry.split(b'\t', 1)
        path = path.decode()
        if path == SOURCE:
            continue
        mode, kind, blob = info.decode().split()
        require(kind == 'blob' and mode in ('100644', '100755'), 'unexpected frozen input kind')
        data = (ROOT / path).read_bytes()
        git_hash = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
        require(git_hash == blob, 'frozen source/evidence changed: ' + path)
        manifest[path] = sha(data)
    require(len(manifest) > 100 and
            'src/Uorc/Search/UnsignedByteConversion.lex.tex' in manifest and
            'src/Uorc/Search/OwnedEnumerationModel.lex.tex' in manifest and
            'src/Uorc/Search/OwnedEnumerationProofs.lex.tex' in manifest and
            'tests/enumeration/helper_surface_policy_equivalence.py' in manifest and
            'docs/implementation/owned-enumeration-helper-surface-policy-evidence.json' in manifest and
            'docs/implementation/owned-enumeration-helper-surface-policy-repair.md' in manifest and
            'docs/implementation/owned-enumeration-stream-diagnostic-evidence.json' in manifest,
            'frozen input inventory is missing protected roots')
    return manifest


IMPLEMENTATION_SHA256 = 'a7aba4cf2547c4169f96bc541ae87bb384c01ffaa737487f4ed04fe6060995c7'


def check_implementation(source=None):
    # This hash prevents accidental gate omissions, not coordinated hostile edits.
    if source is None:
        source = Path(__file__).read_text()
    tree = ast.parse(source)
    require(not any(isinstance(n, ast.Assert) for n in ast.walk(tree)),
            'diagnostic implementation uses removable assertions')
    nodes = [n for n in tree.body if not (
        isinstance(n, ast.FunctionDef) and n.name == 'check_implementation') and not (
        isinstance(n, ast.Assign) and len(n.targets) == 1 and
        isinstance(n.targets[0], ast.Name) and n.targets[0].id == 'IMPLEMENTATION_SHA256')]
    digest = sha(ast.dump(ast.Module(body=nodes, type_ignores=[]), include_attributes=False).encode())
    require(digest == IMPLEMENTATION_SHA256,
            'diagnostic executable inventory changed, filtered, skipped, stale or wrong-file')


def implementation_controls():
    source = Path(__file__).read_text()
    controls = (
        ('filter-constructor-cases', 'for phase in PHASES:', 'for phase in PHASES[:1]:'),
        ('omit-real-source-mutants', 'rejected = negative_controls(evaluator, before, after, paths)',
         'rejected = []'),
        ('disable-filter-guard', "require(len(sys.argv) == 1, 'filters, selection and other arguments are not allowed')",
         "require(True, 'filters, selection and other arguments are not allowed')"),
        ('disable-optimization-guard', "require(sys.flags.optimize == 0 and __debug__, 'optimized Python is not allowed')",
         "require(True, 'optimized Python is not allowed')"),
        ('use-stale-source', 'before, after = parse(baseline), parse((ROOT / SOURCE).read_bytes())',
         'before, after = parse(baseline), parse(baseline)'),
        ('skip-full-fund-paths', "paths = run_paths(evaluator, after['ast'])", 'paths = old_paths'),
        ('skip-nested-module-switch', 'install(evaluator, module)', 'pass'),
        ('skip-frozen-input-binding', 'manifest = frozen_inputs()', 'manifest = {}'),
        ('skip-full-path-restoration', "restored_paths = run_paths(evaluator, after['ast'])", 'restored_paths = paths'),
    )
    rejected = []
    for label, before, after in controls:
        lines = source.splitlines(keepends=True)
        locations = [i for i, line in enumerate(lines) if line.strip() == before]
        require(len(locations) == 1, 'ambiguous implementation mutant: ' + label)
        index = locations[0]
        lines[index] = lines[index].replace(before, after)
        try:
            check_implementation(''.join(lines))
        except ValueError as error:
            require('diagnostic executable inventory changed' in str(error),
                    'wrong implementation-control failure: ' + label)
            rejected.append(label)
        else:
            raise ValueError('diagnostic implementation omission escaped: ' + label)
    require(len(rejected) == 9, 'missing implementation-control inventory')
    check_implementation(source)
    executions = []
    for label, arguments, diagnostic in (
            ('optimized-process', ['-O'], 'optimized Python is not allowed'),
            ('filtered-process', [], 'filters, selection and other arguments are not allowed')):
        command = [sys.executable, '-B', *arguments, str(Path(__file__).resolve())]
        if label == 'filtered-process':
            command.append('--filter=offered')
        result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        require(result.returncode != 0 and result.stdout == '' and diagnostic in result.stderr,
                'real process bypass control escaped: ' + label)
        executions.append({'control': label, 'returncode': result.returncode, 'diagnostic': diagnostic})
    return rejected, executions


def main():
    require(sys.flags.optimize == 0 and __debug__, 'optimized Python is not allowed')
    require(len(sys.argv) == 1, 'filters, selection and other arguments are not allowed')
    check_implementation()
    PROGRESS.write_text('')
    progress('gate-start')
    manifest = frozen_inputs()
    baseline = subprocess.run(['git', '-C', str(ROOT), 'show', BASELINE + ':' + SOURCE],
                              check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE).stdout
    before, after = parse(baseline), parse((ROOT / SOURCE).read_bytes())
    check_shapes(after['ast'])
    check_exact(before, after)
    import diagnostic as evaluator
    require(canonical(evaluator.mods[MODULE]) == canonical(after['ast']['declarations']),
            'evaluator loaded stale or wrong source module')
    dispatch_count = check_dispatch(evaluator)
    reader = load_dependencies(evaluator)
    progress('synthetic-semantics-start', frozen_input_count=len(manifest))
    cases = check_semantics(evaluator, before['ast'], after['ast'])
    progress('original-full-paths-start')
    old_paths = run_paths(evaluator, before['ast'])
    progress('patched-full-paths-start')
    paths = run_paths(evaluator, after['ast'])
    require(old_paths == paths, 'original and patched full fund input/output traces differ')
    progress('full-traces-equal-mutants-start')
    rejected = negative_controls(evaluator, before, after, paths)
    check_exact(before, after)
    restored = check_semantics(evaluator, before['ast'], after['ast'])
    progress('final-full-path-restore-start')
    restored_paths = run_paths(evaluator, after['ast'])
    require(cases == restored and paths == restored_paths, 'restored diagnostic inventory mismatch')
    require(frozen_inputs() == manifest, 'frozen inputs changed while executing')
    require((ROOT / SOURCE).read_bytes() == after['raw'], 'Stream source changed while executing')
    require(check_dispatch(evaluator) == dispatch_count, 'restored dispatch inventory changed')
    progress('implementation-controls-start')
    implementation_rejections, process_rejections = implementation_controls()
    report = {
        'schema': 'uorc/enumeration-stream-phase-comparison-diagnostic/1',
        'status': 'pass-source-diagnostic-only-unaccepted',
        'baseline_commit': BASELINE, 'source_path': SOURCE,
        'baseline_source_sha256': sha(before['raw']), 'current_source_sha256': sha(after['raw']),
        'diagnostic_sha256': sha(Path(__file__).read_bytes()),
        'phase_constructors': list(PHASES), 'phase_constructor_count': len(PHASES),
        'phase_payload_count': 0, 'exact_predicate_replacement_count': len(SITES),
        'sites': [{'definition': name, 'path': list(path), 'target': target,
                   'finding_origin': 'static-source-finding-not-new-compiler-evidence'}
                  for name, path, target in SITES],
        'all_other_stream_source_ast_and_bytes_unchanged': True,
        'frozen_input_count': len(manifest), 'frozen_input_manifest_sha256': sha(canonical(manifest)),
        'frozen_input_sha256': manifest, 'native_reader_input': reader,
        'complete_stream_definition_dispatch_count': dispatch_count,
        'semantic_cases': cases, 'actual_full_fund_paths': paths,
        'original_patched_full_input_output_traces_equal': old_paths == paths,
        'rejected_real_source_ir_mutants': rejected,
        'restored_source_and_full_path_pass': True,
        'rejected_implementation_omissions': implementation_rejections,
        'rejected_real_process_controls': process_rejections,
        'qualification': {'finite_source_evaluation_only': True,
            'official_source_check': False, 'kernel_proofs': False,
            'generated_native_execution': False, 'full_sdk_acceptance': False,
            'archive_controller_integration': False},
    }
    progress('gate-pass')
    print(json.dumps(report, sort_keys=True, indent=2))


if __name__ == '__main__':
    main()
