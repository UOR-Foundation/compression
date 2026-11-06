"""Plant actual source-IR stream defects and require the intended assertion."""
import copy
import sys
import stream_diagnostic_test as controls

e, S, M, H, RR, T, MAX = (controls.e, controls.S, controls.M, controls.H,
                         controls.RR, controls.T, controls.MAX)


def walk(value):
    if isinstance(value, dict):
        yield value
        for child in value.values():
            yield from walk(child)
    elif isinstance(value, list):
        for child in value:
            yield from walk(child)


def first_offer(arena, usage, enumeration, cursor):
    for _ in range(10000):
        changed = controls.fund(arena, usage, enumeration, cursor)
        arena, usage, cursor = changed['arena'], changed['usage'], changed['cursor']
        assert not changed['outcome'][0].endswith('.unresolved'), changed['outcome']
        if changed['outcome'][0].endswith('.offered'):
            return arena, usage, cursor, changed['outcome'][1][0]
    raise AssertionError('no stream offer reached')


def probe_lease():
    arena, usage, enumeration, cursor = controls.fixture(b'\x00')
    arena, usage, cursor, offer = first_offer(arena, usage, enumeration, cursor)
    entry = e.call(H, 'lookup', arena, offer['span']['image'])[1][0]
    assert entry['roots'] >= 1, 'offered image lacks its retained lease root'


def probe_prepaid():
    arena, usage, enumeration, cursor = controls.fixture(b'\x00')
    observed = e.call('Uorc.Search.EnumerationFunding', 'achievedUsage', arena, usage)[1][0]
    changed = controls.fund(arena, observed, enumeration, cursor, credits=0)
    assert (changed['arena'], changed['usage'], changed['cursor']) == (arena, observed, cursor), \
        'unpaid stream event executed'


def probe_generation():
    arena, usage, enumeration, cursor = controls.fixture(b'\x00')
    arena, usage, cursor, old = first_offer(arena, usage, enumeration, cursor)
    cancelled = controls.cancellation(arena, usage, enumeration, cursor)
    started = e.call(S, 'start', enumeration, cursor['source'])
    assert started[0] == 'Result.ok'
    arena, usage, cursor, new = first_offer(cancelled['arena'], cancelled['usage'], enumeration,
                                          started[1][0])
    assert old['owner'] != new['owner']
    changed = controls.fund(arena, usage, enumeration, cursor, T(S + '.Request.acknowledge', old))
    assert changed['outcome'][0].endswith('.unresolved'), 'stale stream generation acknowledged'


def probe_position():
    arena, usage, enumeration, cursor = controls.fixture(b'\x00')
    arena, usage, cursor, offer = first_offer(arena, usage, enumeration, cursor)
    wrong = dict(offer, position=offer['position'] + 1)
    changed = controls.fund(arena, usage, enumeration, cursor,
                            T(S + '.Request.acknowledge', wrong))
    assert changed['outcome'][0].endswith('.unresolved'), 'wrong logical span position acknowledged'


def omit_retain(definition):
    calls = [term for term in walk(definition['body'])
             if term.get('kind') == 'call' and term.get('function') ==
             {'module': H, 'name': 'retain'}]
    assert len(calls) == 1
    calls[0]['arguments'][1] = {'kind': 'call',
                               'function': {'module': 'Uorc.Core.RuntimeValue', 'name': 'nullHandle'},
                               'arguments': []}


def execute_unpaid(definition):
    tests = [term for term in walk(definition['body'])
             if term.get('kind') == 'if' and term['condition'].get('kind') == 'primitive'
             and term['condition'].get('operation') == 'equal']
    assert len(tests) == 1
    tests[0]['condition'] = {'kind': 'bool', 'value': True}


def erase_condition(definition, field):
    matched = []
    for term in walk(definition['body']):
        if term.get('kind') == 'call' and term.get('function', {}).get('name') == 'sameHandle':
            args = term['arguments']
            if field == 'owner' and args[0].get('kind') == 'project' and args[0].get('field') == 'control':
                matched.append(term)
        elif field == 'position' and term.get('kind') == 'primitive' and term.get('operation') == 'equal':
            if term['arguments'][0].get('kind') == 'project' and term['arguments'][0].get('field') == 'offeredPosition':
                matched.append(term)
    assert len(matched) == 1
    matched[0].clear()
    matched[0].update(kind='bool', value=True)


CASES = [
    ('missing-image-lease', 'retainSpan', omit_retain, probe_lease,
     'offered image lacks its retained lease root'),
    ('unpaid-event', 'pay', execute_unpaid, probe_prepaid, 'unpaid stream event executed'),
    ('stale-generation', 'offerMatches', lambda d: erase_condition(d, 'owner'), probe_generation,
     'stale stream generation acknowledged'),
    ('wrong-position', 'offerMatches', lambda d: erase_condition(d, 'position'), probe_position,
     'wrong logical span position acknowledged'),
]


def run_controls():
    for _, _, _, probe, _ in CASES:
        probe()
    print('PASS original stream mutation probes')
    for name, function, mutate, probe, intended in CASES:
        key = S + '.' + function
        original = e.funcs[key]
        mutant = copy.deepcopy(original)
        mutate(mutant)
        e.funcs[key] = mutant
        try:
            probe()
        except AssertionError as error:
            if str(error) != intended:
                raise AssertionError('wrong mutant diagnostic: ' + name + ': ' + str(error)) from error
            print('REJECTED', name, intended)
        else:
            raise AssertionError('stream defect escaped: ' + name)
        finally:
            e.funcs[key] = original
        probe()
    print('PASS all four stream mutants rejected at intended assertions and restored')


if __name__ == '__main__':
    if not __debug__:
        raise SystemExit('optimized Python disables stream mutation assertions')
    if len(sys.argv) != 1:
        raise SystemExit('stream mutation controls accept no filters or arguments')
    run_controls()
