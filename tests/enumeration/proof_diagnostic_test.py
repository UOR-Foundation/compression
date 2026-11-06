#!/usr/bin/env python3
"""Finite evaluation of owned proof-source statements, never proof acceptance.

Inputs are exactly: 80 seeded general environments (seed 20261105), the source
modelDecode outputs for ranks 0..279 with both original and +256 Nat payloads,
25 arbitrary fixed-width Cursor/empty-Arena environments, and all-zero words
of lengths 0..14 for decoder/cell-retention controls. modelAt is evaluated only
at ranks 0..279. General list lengths are 0..12; arbitrary Nat payloads are
0..1023. Scalar heads are 0..99999, scalar payloads use the eight listed
boundaries, intermediates are 0..9999, and ranks are within UInt64 bounds.

All definitions are evaluated from parsed source by diagnostic.py. The only
term-evaluator extension is ordinary logical iff; there is no optimized model,
reference implementation, production replacement, or SDK/compiler bootstrap.
Every listed statement is evaluated. False implication antecedents are counted
separately, never as active coverage. Successful finite statements, shape/hash
checks, and source mutants do not establish general truth or kernel acceptance.
"""
import hashlib
import json
from pathlib import Path
import random
import re
import sys

if sys.flags.optimize != 0 or not __debug__:
    raise RuntimeError('optimized Python is not allowed for source diagnostics')
if sys.argv[1:]:
    raise RuntimeError('filter, selection, and other arguments are not allowed')
sys.setrecursionlimit(20000)
import diagnostic as source

ROOT = Path(__file__).resolve().parents[2]
MODEL = 'Uorc.Search.OwnedEnumerationModel'
PROOFS = 'Uorc.Search.OwnedEnumerationProofs'
MODULES = (MODEL, PROOFS)
OWNED = 'Uorc.Search.OwnedEnumeration'
HEAP = 'Uorc.Core.Heap'
BUILDER = 'Uorc.Core.RuntimeByteBuilder'
VALUE = 'Uorc.Core.RuntimeValue'
RESOURCES = 'Uorc.Core.RuntimeResources'
SEQUENCE = 'Uorc.Core.RuntimeSequence'
MAX_WORD = (1 << 64) - 1
SEED = 20261105
EXPECTED_STATEMENTS = {
    "append_attempt_preserves_exact_predecessor": "cfb8186eaa2a4a4aa6f6486f7c54e9ab442d19f715f32cfacc46b55bcf7694df",
    "build_choice_preserves_exact_predecessor": "9244d8d4283488695bce23bf9d804361a16ace7e26e26a70da803d19067ea594",
    "build_read_preserves_exact_predecessor": "80e628e5899a93937e23ef683f420625f543f81f523653aba2280571fc8070c8",
    "builder_refusal_preserves_exact_unexecuted_state": "966696c55bb0f1ad6a8abb1d30f62bd9f8c45c3406b480068e6a96b860dcd78e",
    "builder_result_preserves_exact_predecessor": "0067f8553dc2d67c570597c5b81eca053039d189bab51eb906ae9d1cb601da33",
    "completed_pivot_scan_enters_build_begin": "f9626883c7652b8f8f7ca7aed80bc3a25a5dbcab95b3a14535cf876a33cbf71e",
    "completed_pivot_scan_keeps_same_length": "dacfcb02ad292153fcb598206260a519da9e9d970e5380c27bdfc39f87535027",
    "completed_pivot_scan_restarts_build_at_zero": "bc51d0c140e03e4f338f6d8b46187d68e9ed17251e9017b94f381b4fa59cb028",
    "finish_attempt_preserves_exact_predecessor": "151a107f796651a70a1fa93ad3f8e69d74c6dafd93f9b9e8a05b96c73541ac54",
    "install_clears_transferred_next_root": "a2df4316da2b57a76b0b4ebc3b41b395f655941fff35fa60a0e0d2e294ca596c",
    "install_enters_offer_only": "b81e19522681254a1583328b96d286aa2b66fe3080a59b06b392cc48b2a32452",
    "install_selects_completed_next_length": "c831f0d7b7193644173bdb4ab98e222f7ca2cbd4d9173bc69a9f18460070e325",
    "install_selects_completed_next_root": "15b5eb14bcdebafb563385dda07a7aedf01b577a342ce02d80f0fa04f2a7a8df",
    "model_adjacent_ranks_have_no_intermediate_rank": "2e18dea7cf5d0ee54bb09485aaa80ed6183b8974e32b9f43ad6b3a9003e46ba7",
    "model_at_rank_is_exact_pointwise_normalization": "45a474c82b6a534006d337e8c42739939f1602004006889a42f0034a5b960b50",
    "model_byte_bounded_finite_word_has_exact_coverage": "93eae5bb0efa272b847c7f3b11d636f4d723d3c8235a14f7fd675273bb6d041f",
    "model_byte_is_identity_on_every_byte": "5ee86ccfbfe71d57c1a0ae122375f2c3cd683932189eb26f5bad2eb567745181",
    "model_byte_normalization_is_idempotent": "4c91113739e16c231b1c71b8346badd5c06404bcedc50a7632a18901ef5dce39",
    "model_byte_stays_within_unsigned_range": "0679e6bbbdf73622f7639d939fdcce775d7a10006bcc29846dec2a36a65789bd",
    "model_cons_rank_decodes_exact_head": "4c8640f549889cc430e5bbf991010d70df727566a6e4bd902522a1feea5396b0",
    "model_cons_rank_decodes_exact_tail_rank": "85076c274384fd92bb7fb3935bd3979d0aae1ec4da8611d482070b30832ecd79",
    "model_coverage_iff_normalized_coverage": "0e9ea2a6f09181558dbdcf3b9185c558a25098bbe0e1d06612d3fd093e60d898",
    "model_decoder_quotient_strictly_decreases": "23e4a2ffc738af8abbbec49e97905cc29db6622cf87729a984ae45b26981f009",
    "model_decoding_rank_is_exact_pointwise_normalization": "c124fa3155ba5fbce519b84051ae3d4cc40222e912793f9375fd49ba589eb567",
    "model_equal_ranks_have_equal_decodings": "1d784b8015dd740324075dfe2ddf8bdd5738c08c5c9503591dde16902b4039d7",
    "model_equal_ranks_imply_normalized_word_equality": "9de686dca3690a63e0d41c7245bfa0ee6b4b6e4f6968705c50e74fd139966109",
    "model_equal_words_have_equal_observed_ranks": "97b0dd73e5a9e9aaba59d0fe8bd12bcc07226eb2a7067f8cee284a744746c592",
    "model_every_finite_unsigned_byte_word_has_exact_coverage": "ba2bcc2bf3934e9565ac843b3d35db5d4345ef4b89af4f7cbc418435307022c1",
    "model_every_finite_unsigned_word_is_pointwise_normalized": "dd40ab169c6e667c870554754335815d9ba1c50cca566f2a245790ee1a4e9344",
    "model_every_iterated_word_is_byte_bounded": "d9349baa8eac7e2555ed4c9c8a9f1be3c0b91ab308bbc868ea7757d1c6bb3670",
    "model_every_iterated_word_is_pointwise_normalized": "2240c9dae191e14bb7df4bd5c6aae537decb72c3fdbaf7358667ee5ea1701181",
    "model_every_pointwise_normalized_finite_word_is_reached": "640d023ad6af2891b39ebbac9a21047db3222627737e74e7c16d86146aa161c2",
    "model_natural_word_round_trip_is_pointwise_normalization": "fd9cdc184492a967459bc709cd23641062ab2aff82a6418db00a5b2bd0a209d3",
    "model_next_byte_ignores_only_payload_modulo_aliases": "eefecb9dcb2713559cb461b3b33ebdea91f0632503e338f855aaf3eaba3a6527",
    "model_next_byte_is_already_normalized": "ec4708a6aac9ab858883e855c1459a535ee67d52b8d0e5c177aefa49f5bd761f",
    "model_nonempty_rank_is_nonzero": "d01963ec139e21beaafc2d90455ae29f9ff83d9490f910e47b13dc8b46f856c9",
    "model_normalization_commutes_with_successor": "15c23bfebe891e38985400c7914067f7681d51f285d63a5436553b7197bb296b",
    "model_normalization_distributes_over_byte_choice": "8f83328b9263979e14c9fe22ce1a7083d574c6da58065cae4600a74d2784f846",
    "model_normalization_is_idempotent": "6f78a283951b1663df7dcf1387fc97f0b053843ee38ed848bdbf116546b53b4b",
    "model_normalization_preserves_every_list_cell": "d99e069b2ecaf11f10177c2ca60f52858f3c111a148c96613d8aee00ebd61bae",
    "model_normalization_preserves_rank": "97a335375bfa46be621e5f8762b295f7c965a8aca4a03ec35c5fbee2f8f5d703",
    "model_normalized_equality_iff_decoded_equality": "80b728847bb323a8b88da5404cc2f59f3aeab0b014c64bb7b108ff09a0e81678",
    "model_normalized_image_is_byte_bounded": "3c8ae75f6a67bd2e6b6ec64e9698c7b380635d23f1f18f0b00260a216515e88d",
    "model_normalized_scalar_round_trip": "cd78d6cc2b4971b352843add1f5ef7509133cb3fb8f84aace993cd7af32916bc",
    "model_normalized_word_equality_implies_equal_ranks": "58d2b0ba4c3a5b048c243b0fb9374a4540d14f5ff3d81ea07235175200dc2c87",
    "model_normalized_words_equal_iff_ranks_equal": "df2edb27fa0dee1c2ac63d321cfa1bb29fd9d8414b6987183ae318482140b195",
    "model_rank_equality_iff_normalized_observed_ranks": "6394e414bd2c509b19cf3ffae95d7714552096ebdd21c41a5f8a0eeef351b769",
    "model_rank_equality_iff_pointwise_normalized_word_equality": "3674119f5693fce1af83e436d8dfb14289226aa6ac4a8d8d223275fc199500ab",
    "model_rank_is_injective_on_pointwise_fixed_words": "7d0f7c5f193e610c5727b9143909046c6b20bad02a9731b36844e07ac37ac84a",
    "model_reverse_enumeration_has_every_natural_rank": "53f122b8e953b39cb6d9f5ea99094c7b7048b09e16e5abbd20ca9567d478a277",
    "model_reverse_successor_has_exact_cons_form": "5052c3a1d4a9c711448991e8292a6b3259c2e2f34088cd1fd386358f94ac9734",
    "model_reverse_successor_increments_rank": "1d2bf4f6c662238d8f7e603a2e6fdac6208c48b0198f31622f1f8b166f268ff7",
    "model_successor_preserves_byte_bounded_words": "b9d2a686417ec9b5f8d07d7ad80a44f7d9585d25dbcc2d0f4b34b899df84bf6e",
    "model_unsigned_byte_is_already_normalized": "dab8a82ef93c251422d90855a0153cda9d9512a513135a02f268ffb4b0a522bb",
    "model_unsigned_byte_word_round_trip": "2eedcbfce8f88262e0d4a7e1606bfb66f41881eff29d3d653f21481e62f45502",
    "model_unsigned_inverse_preserves_every_list_cell": "3fd99c3c3516402c00f5f12b4fe66730ad4f8dcfd5a12461660b548e3e30568d",
    "model_unsigned_scalar_round_trip": "53632a030f2f757fd3b9cbcd0ca1588987bfc023ffab34dac23a07735fc5ae6a",
    "model_unsigned_word_embedding_preserves_every_list_cell": "e0c48c75e821dae1bdfe9b2fe1d9a23dfab33985288d6768e51038078a3d485e",
    "scan_phase_acknowledgment_is_rejected": "33e425195d84680976d6e9361ccdd39c2ffec2d36af4b41baf3fd31f4b239119",
    "scan_phase_cannot_match_any_outstanding_ticket": "1e05a6e572ccc8a463bac214060572c3ed4d87bb9c6cfbfb5a32710f9b34d9a3",
    "scan_phase_rejected_ack_funding_preserves_exact_state": "a38d789e86a4cb59c864aed2d07b96b4b808fca7f12220cafc1077b6b0a12c32",
    "scan_preserves_exact_predecessor": "cf6bc6517946d5d3120ddb273f10889b399296677bbd40b81892957e20ee1829",
    "waiting_primitive_replays_without_changing_any_state": "1b7469a0a82b13af679ef0f95a6502d54ef3a83e248c7e56f66e5eba36bbb0d6"
}
EXPENSIVE = {
    'model_reverse_enumeration_has_every_natural_rank',
    'model_every_iterated_word_is_pointwise_normalized',
    'model_coverage_iff_normalized_coverage',
    'model_every_pointwise_normalized_finite_word_is_reached',
    'model_at_rank_is_exact_pointwise_normalization',
    'model_byte_bounded_finite_word_has_exact_coverage',
    'model_every_iterated_word_is_byte_bounded',
    'model_every_finite_unsigned_byte_word_has_exact_coverage',
}
MUTANT_OWNERS = {
    'successor-plus-two': 'model_reverse_successor_increments_rank',
    'drop-zero-normalization': 'model_normalization_preserves_every_list_cell',
    'constant-zero-embedding': 'model_unsigned_byte_word_round_trip',
    'scan-current-root-erase': 'scan_preserves_exact_predecessor',
    'builder-refusal-phase-advance': 'builder_refusal_preserves_exact_unexecuted_state',
    'install-old-current': 'install_selects_completed_next_root',
}


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':')).encode()


def sha(data):
    return hashlib.sha256(data).hexdigest()


base_ev = source.ev

def evaluate(term, module, environment):
    if term['kind'] == 'iff':
        return evaluate(term['left'], module, environment) == evaluate(term['right'], module, environment)
    return base_ev(term, module, environment)

source.ev = evaluate


def closure_manifest():
    pending = list(MODULES)
    visited = {}
    while pending:
        module = pending.pop()
        if module in visited:
            continue
        path = ROOT / 'src' / (module.replace('.', '/') + '.lex.tex')
        data = path.read_bytes()
        imports = sorted(set(re.findall(r'\\importmodule\{([^}]+)\}', data.decode())))
        visited[module] = {'path': path.relative_to(ROOT).as_posix(),
                           'sha256': sha(data), 'imports': imports}
        pending.extend(imports)
    cache = ROOT / 'tests/heap/semantic-cache/runtime-owners.json'
    configurations = {}
    for name in ['lexlean.toml', 'lexlean.lock', 'prismpm.lock',
                 'lake-manifest.json', 'lean-toolchain']:
        path = ROOT / name
        if path.exists():
            configurations[name] = sha(path.read_bytes())
    root_closures = {}
    for root in MODULES:
        todo = [root]
        reached = set()
        while todo:
            module = todo.pop()
            if module not in reached:
                reached.add(module)
                todo.extend(visited[module]['imports'])
        root_closures[root] = sorted(reached)
    return {'source_import_closure': dict(sorted(visited.items())),
            'root_closures': root_closures,
            'parsed_runtime_cache_sha256': sha(cache.read_bytes()),
            'diagnostic_evaluator_sha256': sha((ROOT / 'tests/enumeration/diagnostic.py').read_bytes()),
            'diagnostic_script_sha256': sha(Path(__file__).read_bytes()),
            'configuration_hashes': configurations}


def general_environments():
    rng = random.Random(SEED)
    payloads = [0, 1, 254, 255, 256, 511, 65535, 65536]
    for sample in range(80):
        words = [rng.randrange(1024) for _ in range(rng.randrange(13))]
        left = [rng.randrange(1024) for _ in range(rng.randrange(8))]
        if sample % 4 == 0:
            words = [x % 256 for x in words]
            left = [x % 256 for x in left]
        if sample % 3 == 0:
            right = list(left)
        elif sample % 3 == 1:
            right = [x % 256 + 256 for x in left]
        else:
            right = [rng.randrange(1024) for _ in range(rng.randrange(8))]
        left_rank = rng.randrange(1 << 64)
        right_rank = left_rank if sample % 4 == 0 else rng.randrange(1 << 64)
        yield {'words': words, 'payload': payloads[sample % len(payloads)],
               'head': rng.randrange(100000), 'tail': words,
               'rank': [0, 1, 2, 255, 256, 257, MAX_WORD][sample % 7],
               'leftRank': left_rank, 'rightRank': right_rank,
               'left': left, 'right': right, 'chooseLeft': bool(sample % 2),
               'value': rng.randrange(256), 'bytes': [x % 256 for x in words],
               'intermediate': rng.randrange(10000)}


def rank_environments():
    for rank in range(280):
        word = source.call(MODEL, 'modelDecode', rank)
        for words in [word, [x + 256 for x in word]]:
            yield {'words': words, 'rank': rank, 'bytes': word}


def machine_environments():
    rng = random.Random(SEED)
    arena = source.call(HEAP, 'emptyArena', 1000000)
    start = source.call(OWNED, 'start')
    budget = {'type': RESOURCES + '.Budget', 'maxWork': MAX_WORD,
              'maxOwnedBytes': 1000000, 'maxOutputBytes': MAX_WORD,
              'maxFrames': MAX_WORD}
    phases = [x['name'] for x in next(d for d in source.mods[OWNED] if d['name'] == 'Phase')['constructors']]
    for sample in range(25):
        cursor = dict(start)
        for key in ['length', 'nextLength', 'position', 'pivot', 'index', 'paid']:
            cursor[key] = rng.choice([0, 1, 255, 256, MAX_WORD, rng.randrange(65536)])
        for key in ['pivotValue', 'byteValue']:
            cursor[key] = rng.randrange(256)
        for key in ['ticketOwned', 'pivotFound', 'builderHeld', 'builderActive', 'cancelling']:
            cursor[key] = bool(rng.randrange(2))
        for key in ['control', 'current', 'next', 'walking']:
            cursor[key] = {'type': VALUE + '.Handle', 'id': rng.choice([0, 1, 2, 65536, MAX_WORD])}
        cursor['phase'] = source.tag(OWNED + '.Phase.' + phases[sample % len(phases)])
        failure = source.tag(BUILDER + '.BuilderFailure.invalidBuilder')
        outcomes = [source.tag(BUILDER + '.Outcome.pending'),
                    source.tag(BUILDER + '.Outcome.builder', cursor['builder']),
                    source.tag(BUILDER + '.Outcome.sequence', cursor['next']),
                    source.tag(BUILDER + '.Outcome.cancelled')]
        built = {'type': BUILDER + '.BuilderTransition', 'arena': arena,
                 'cursor': cursor['builderCursor'], 'delta': source.call(SEQUENCE, 'zeroDelta'),
                 'work': 0, 'copied': 0, 'outcome': outcomes[sample % 4]}
        result = source.tag('Result.error', failure) if sample % 5 == 0 else source.tag('Result.ok', built)
        yield {'arena': arena, 'cursor': cursor, 'result': result, 'failure': failure,
               'ticket': source.call(OWNED, 'ticket', cursor), 'budget': budget,
               'usage': source.call(RESOURCES, 'zeroUsage'), 'available': rng.randrange(64)}


def all_antecedents_hold(statement, module, environment):
    while statement['kind'] == 'implies':
        if not evaluate(statement['premise'], module, environment):
            return False
        statement = statement['conclusion']
    return True


def run():
    theorems = {d['name']: (module, d) for module in MODULES
                for d in source.mods[module] if d['kind'] == 'theorem'}
    if len(theorems) != sum(d['kind'] == 'theorem' for module in MODULES for d in source.mods[module]):
        raise AssertionError('duplicate theorem name across split modules')
    if set(theorems) != set(EXPECTED_STATEMENTS):
        raise AssertionError('missing, extra, or renamed source theorem in specimen inventory')
    for name, (module, theorem) in theorems.items():
        if sha(canonical(theorem['statement'])) != EXPECTED_STATEMENTS[name]:
            raise AssertionError('changed statement requires explicit specimen review: ' + name)
    groups = {'general-80': list(general_environments()),
              'rank-0-through-279-two-payload-images': list(rank_environments()),
              'arbitrary-cursor-empty-arena-25': list(machine_environments())}
    inventory = {}
    for name in sorted(theorems):
        module, theorem = theorems[name]
        if module != (MODEL if name.startswith('model_') else PROOFS):
            raise AssertionError('unexpected theorem origin module: ' + name)
        group = ('rank-0-through-279-two-payload-images' if name in EXPENSIVE else
                 'general-80' if name.startswith('model_') else
                 'arbitrary-cursor-empty-arena-25')
        specimens = groups[group]
        active = 0
        inactive = 0
        iff_polarities = {}
        inputs = []
        for index, full_environment in enumerate(specimens):
            environment = {parameter['name']: full_environment[parameter['name']]
                           for parameter in theorem['parameters']}
            inputs.append(environment)
            source.steps = 0
            if not evaluate(theorem['statement'], module, environment):
                raise AssertionError({'theorem': name, 'specimen': index,
                                      'group': group, 'environment': environment})
            if all_antecedents_hold(theorem['statement'], module, environment):
                active += 1
            else:
                inactive += 1
            if theorem['statement']['kind'] == 'iff':
                pair = str(evaluate(theorem['statement']['left'], module, environment)) + '/' + str(evaluate(theorem['statement']['right'], module, environment))
                iff_polarities[pair] = iff_polarities.get(pair, 0) + 1
        inventory[name] = {'origin_module': module,
                           'reassigned_from': PROOFS if module == MODEL else None,
                           'statement_sha256': EXPECTED_STATEMENTS[name],
                           'root_proof_node': theorem['proof']['kind'],
                           'specimen_group': group, 'specimen_inputs_sha256': sha(canonical(inputs)),
                           'evaluated': len(specimens), 'all_top_level_antecedents_true': active,
                           'false_top_level_antecedent': inactive,
                           'iff_polarities': iff_polarities}
    zero_controls = []
    previous_rank = -1
    for length in range(15):
        word = [0] * length
        rank = source.call(MODEL, 'modelReverseRank', word)
        if source.call(MODEL, 'modelDecode', rank) != word or rank <= previous_rank:
            raise AssertionError('zero-cell/length distinction lost')
        if source.call(MODEL, 'modelBytesOfWords', source.call(MODEL, 'modelWordsOfBytes', word)) != word:
            raise AssertionError('UInt8 zero cells lost in pointwise round trip')
        zero_controls.append({'length': length, 'rank': rank})
        previous_rank = rank
    result = {'schema': 'uorc/owned-proof-source-diagnostic/1',
              'status': 'finite-source-statements-pass-unaccepted',
              'claim_boundary': 'Not LexLean, Lean, kernel, native, SDK, general proof, reachable-arena, or owned-machine refinement acceptance.',
              'specimen_bounds': {
                  'seed': SEED, 'general_environments': 80,
                  'general_list_length': [0, 12], 'general_list_payload': [0, 1023],
                  'scalar_head': [0, 99999],
                  'scalar_payloads': [0, 1, 254, 255, 256, 511, 65535, 65536],
                  'scalar_rank': [0, MAX_WORD], 'intermediate': [0, 9999],
                  'unsigned_byte': [0, 255], 'modelAt_evaluated_rank': [0, 279],
                  'rank_specimens_per_rank': 2, 'machine_environments': 25,
                  'zero_word_length_controls': [0, 14],
                  'machine_scope': 'arbitrary Cursor fields over an empty Arena, not reachable-state evidence'},
              'declared_axiom_policies': {
                  module + '.' + d['name']: d.get('axioms', []) for module in MODULES
                  for d in source.mods[module] if d['kind'] in ['definition', 'theorem']},
              'axiom_policy_boundary': 'Declared exact source policies only; no observed Lean axiom audit has run.',
              'theorem_inventory': inventory, 'zero_length_controls': zero_controls,
              'mutant_owner_map': {name: (MODEL if owner.startswith('model_') else PROOFS) + '.' + owner
                                   for name, owner in MUTANT_OWNERS.items()},
              'module_roots': list(MODULES), 'closure': closure_manifest(),
              'skipped_theorems': [], 'unsupported_theorems': []}
    print(json.dumps(result, sort_keys=True, indent=2))


if __name__ == '__main__':
    run()
