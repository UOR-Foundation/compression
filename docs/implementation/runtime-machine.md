<!-- uorc:non-authoritative -->

# Owned decoder machine implementation evidence

This is scoped implementation evidence. The LexLean source is authoritative.
Issue #14 and product acceptance remain open. The complete handle-only execution
engine is implemented; its general refinement and locked-SDK qualification are
separate obligations.

The branch starts at public integration `c4f261008c614ea66cbe103f0d229c6b8819bd88`.
It imports the Heap-owned source from `d10bf90dcdf8c0f536632dc1be28d14fa37de127`
and the Sequence/Builder/Table corrections at
`c3e2df89d535e7a34cfc29ff13b1f37d7c0f184d`, without modifying that contributor
branch. Compatible Core helpers and source repairs come from
`cc987e418cbc3d39c019fc41869b34c8b1f92066`. No SDK lock or universal policy changes.

## Actual entry points and ownership

`RuntimeMachine.lex.tex` prepares all 39 normalized symbolic operations and owns
checked scalar arithmetic, typed references, captures and loop control.
`RuntimeMachineEngine.lex.tex` composes that preparation with the actual Heap,
resumable RuntimeTable, RuntimeSequence, RuntimeStore and RuntimeByteBuilder
protocols. It executes actual programs, including Apply, Choose, Tabulate, Fold
and Scan. No primitive returns a fake successful value or unavailable stub.

The controller-facing roots are `RuntimeMachineEngine.start`, `tick`, `extend`,
`cancel`, `ownedRoots`, `takeOutput`, `takeOverflowReceipt` and `takeReady`.
`start(ready,budget)` consumes the admitted Ready lineage, including its sole
arena, program/basis tickets, all CallerRoots and Usage. Active state has no
independent Ready or arena snapshot. Captures borrow immutable metadata and
transfer one item after its destination edge is installed. Frame cons cells and
bounded control records have explicit control owners; value aliases preserve
outside-ticket multiplicity. Exact signatures and capacity charges are in
[runtime-machine-engine.md](runtime-machine-engine.md).

`takeOutput` returns an updated state plus exactly one owned output handle;
`valueOwned` distinguishes an empty output ticket from an already transferred
result. Cleanup preserves Ready's program, basis and caller tickets.
`takeReady` consumes the cleaned state and returns the achieved arena and those
tickets. It refuses while a frozen overflow receipt remains unconsumed. No active
machine detach or arbitrary validation-snapshot reattachment is introduced.

A costly event is computed only after its checked work charge succeeds. Failed
native operations preserve their achieved arena, cursor, temporary tickets and
cumulative delta. If composing a cumulative UInt64 delta overflows, the accepted
prefix and achieved event remain separately owned in a frozen receipt; cleanup
continues reporting per-event deltas. `takeOverflowReceipt` transfers both pieces
before Ready may leave. Modeled errors, generated ComputeError and unknown/fatal
host outcomes remain distinct; no host rollback guarantee is asserted.

Historical wire local/backward-block distances are one-based. RuntimeProgram
backwardIndex/localIndex/priorBlockIndex and RuntimeAdmissionChecks now enforce
1..count with absolute index count-distance. Fixtures pass those same operands
to metadata and the independent Graph; the old compensating conversion is
removed. Slot positions, capture-array positions and lifetime indices remain
zero-based absolute positions. No symbolic constructor order is used as a wire
tag. Initial unused-parameter
releases use block field7, and releasesAfter uses the instruction's zero-based
index. Erasure removes slot edges. Return holds its result while retiring the
whole slot root, without an instructionCount+1 sentinel. Scan observes before
each update and performs the final updater, including when that updater fails.

## Executed controls

The official release LexLean 0.4.0 binary SHA256 is
`ae0112f07e2de5bd1cca1b98f7785919de93d9e2bb41bf294df14ddeb1a472d2`;
Lean 4.32.1 SHA256 is
`e8baaa71855a616dc351028f3ad2200051b0671f423a1696a100e809302d5550`.
These are diagnostics, not the immutable SDK selected by prismpm.lock.
The official migrator prepared an isolated language-1.2 computational projection
of mixed-version dependencies. Dependency theorem bodies were omitted in that
projection; their original product source and preservation policies were not
changed. Exact diagnostic inputs are retained in the receipts.

The generated Lean modules kernel-compile, and actual generated Lean evaluator
execution passes:

- All 12 helper groups, including 847 arithmetic/shift comparisons against the
  independent ReferenceSemantics over eleven boundary words per operand;
- The unfiltered `RuntimeMachineEngineTests.run` aggregate of 68 source-owned
  typed scenarios: every normalized opcode in actual programs compared with the
  independent reference, capture/choice/reference failures, empty and nonempty
  iterations, aliases/default removal, and a failing final Scan updater;
- Exact work/frame/output/owned-byte suspension and extension, child-frame
  refusal, partial table allocation, retained output after cleanup, all four
  pending-owner cancellations with per-tick HeapAudit, seven aliased CallerRoots
  domains, checked counter/copy-cost boundaries and overflow-receipt cleanup.

The partial-allocation control observes 12 achieved handles and 13,968 bytes
before a 14,000-byte extra quota refuses; extension completes the same operation.
Artificial internal counter/ledger states are explicitly protocol controls,
not claims that admission accepts malformed input.

Four isolated source mutations build and kernel-compile, then fail their intended
source-owned controls: add changed to subtract, local offset shifted by one,
output ticket omitted from root inventory, and work charged after execution.
The three checked-counter controls also failed against the preceding engine and
pass against the corrected source. Finite controls demonstrate sensitivity;
they do not constitute general refinement proofs.

Portable receipts are in `tests/runtime-machine/evidence/`: helpers.json,
engine.json and mutants.json. The engine receipt binds exact source inputs to
build `df82ed191c742ee74be23c088387b9cf3e2fa2240c4b18763d525386ea001fb2`.
`EngineRun.lean` asserts the complete source aggregate without a host-side filter.
Scoped exact axiom traces in runtime-machine-axioms.json contain only the empty
set or {Quot.sound, propext}; all 67 engine function policies match. These checks
do not claim full repository proof verification.

## Cost boundary and remaining general obligations

Ready.usage.work includes its achieved admission/transport baseline. Engine
Usage.work then charges actual prepaid execution/control/copy events. Heap.Delta
retains a separate ownership-operation ledger. Sequence representation and
transport partitioning can change actual traversal, allocation and reclamation
work. This operational Usage is not a partition-independent canonical decoder
cost for minimum certificates. `RuntimeMachineRefinement.operationalCreditsSince`
subtracts an explicit admission baseline only; it does not make those execution
credits canonical. Core/search must supply the deterministic canonical policy
and prove its relation to these operational costs before certificate eligibility
uses it. Real operational work and memory accounting must remain intact.

`RuntimeMachineRefinement.lex.tex` is verification-only. It interprets actual
arena images/sequences and 64-bit store paths into independent reference values,
relates live frame slots with an explicit liveness witness, and states actual
ownership and completion goals. These denotations are not retained engine
backing. Two general helper control lemmas and the general
natural_meaning_is_exact and preflight_preserves_computation lemmas are proved.
The reachableOwnershipGoal and completedReferenceGoal definitions are open goal
interfaces, not asserted theorems. The smallest remaining bridge requires:

1. Admitted program/basis/capture and live-slot denotation, including valid
   zero-based metadata to one-based reference correspondence and last-use erasure;
2. Per-event or compositional simulation for every instruction, continuation and
   owner cursor, preserving denotation, root multiplicity, charged retained
   state, cumulative achieved effects, prepaid work and refusal/resume prefixes;
3. Reachable ownership/peak correctness and final output byte interpretation
   against independent evaluateGraph, plus appropriate termination/definedness;
4. The canonical cost-profile refinement described above.

Certified native export, generated sharing/Clone inspection and measured physical
copy/RSS retention, complete product controller integration, full model
projection and locked-SDK just vv remain open. Logical owned bytes are not RSS.
The engine does not consume Span normally; a nonempty unexpected Span's separate
owned image ticket is preserved before protocol failure. Span consumer/emitter
qualification belongs to its actual consumer lane.

## Continuing qualification

Two additional general source lemmas prove cancellation preserves ownedPrefix
(the actual root audit and owned-byte agreement) and the achieved arena, Usage,
current delta, operation ledger and frozen event. They kernel-compile in exact
build 9bcb12757cbc08ae9b68c9933cf76dc1310d8682e16e8b81e8cb9a4953607e92;
observed axiom sets are {Quot.sound, propext} and {propext}. These establish the
cancellation request transition, not all subsequent cleanup events.

A cross-model review subsequently found that historical Graph/local/block
distances are one-based. The earlier RuntimeProgram readers and component
fixtures used zero-based distances with a fixture conversion. The Core owner is
correcting those consumers against Graph-derived admission controls; machine
fixtures must pass historical operands directly. The preceding 68-case report
is historical component evidence for its exact source shapes and is not a
general reference-equivalence result. Corrected composition validation is pending.

Native export and measurement preparation is retained in
`tests/runtime-machine/production-request.json` and native-qualification.md.
The request is not an accepted export or evidence of physical measurements.

## Composition correction checkpoint

The current source binds public integration
`f947c988c6769fac2d94a850afee19015d18653c`. It corrects actual Scan consumption:
metadata operand2 and first capture lane are observer; operand3 and second lane
are updater. Internal Action.scan still carries updater then observer. The
fixtures now preserve the same observer/updater ordering in metadata and Graph.
Distinct typed/captured observer/updater and byte-state controls demonstrated
the preceding implementation's type error and silent wrong byte before repair.

Representational sequence length, builder length, store work, slot/instruction
counter, capture-counter and effect-ledger overflow are operational inability,
not semantic arithmetic errors or predecessor exclusion. Native owner failures
keep their actual achieved state and tickets. A compact 64-doubling sequence DAG
followed by a one-byte slice succeeds independently without materializing its
logical 2^64 bytes; its old semantic length-overflow classification was rejected
by the registered control before repair.

Engine prepayment now calls the shared RuntimeWork tariff authority, including
checked builder copy bounds, before the actual bounded event. Store begin/step
use the shared indexedStart tariff. Operational fees remain distinct from
Heap.Delta and the provisional canonical semantic ledger. No complete canonical
event/storage refinement is claimed.

The strict thirteen-group helper driver passes against the corrected Core
computational projection, including direct one-based reference and released-slot
controls. Receipt: tests/runtime-machine/evidence/helpers-f947.json. New actual
engine and completed-admission lifecycle controls are authored and under
execution; their registry status distinguishes checked source from passed
behavior. The preceding 68-case receipt is retained as historical component
evidence for its exact old fixtures, which reordered references and Scan roles;
it does not establish equivalence of the canonical wire/admission program.

The official migrator exposed four small source compatibility repairs in the
imported composition: RuntimeAdmissionBoundary explicitly imports Order,
InstructionCheck and Lifetime; RuntimeAdmissionCredit alpha-renames arena locals
that collide with its arena declaration; RuntimeAdmissionCreditTests renames the
reserved partial binder; RuntimeReferenceTests marks mathematical Graph-based
controls non-executable as required by their independent reference calls.
Computational behavior is unchanged by these authoring repairs. No compiler, SDK
lock, universal policy or contributor branch is changed.


## Owned unit-cache and fixed-width ledger checkpoint

RuntimeMachineSemantic supplies the shared fixed-width adapter for
logicalReferenceEvents1, with explicit profile/storage guards, checked counters,
retained failed prefixes and proof-side lifts to the single CanonicalCost fee
authority. Its ten fresh-profile generated Lean evaluator controls pass and
71 exact axiom policies match. This adapter is not yet an Engine event ledger.

Each Engine frame now owns an additional unitsTable root and a UInt64
accumulatorUnits witness; State carries valueUnits. Data and cache mutations
commit separately through native retained-failure table cursors. Slot/capture
counters advance after both commits, and reclamation erases both edges before
retiring their roots. Aliases, arguments, captures and returned values propagate
the witness. Store writes inspect the existing leaf before the native mutation
and update 1 + 9 times the exception count without traversing the store. Cache
addressability and invariant failures preserve achieved state as operational
inability. Cached-unit correspondence to the independent reference remains a
proof obligation, including canonical store reachability/cardinality.

RuntimePeak now records the actual pre-event arena after prepayment and folds
the returned arena before later release, including failed prefixes and native
table transition peaks. Metric errors fail with achieved state rather than
becoming zero. The 23-module cache/peak generated Lean kernel closure compiled
and all 83 Engine function axiom policies matched. Canonical formatting
preserved generated Engine and Semantic Lean byte for byte; exact identities
are recorded in tests/runtime-machine/evidence/canonical-format.json.

The corrected earlier Engine source passes its complete 83-root generated Lean
evaluator driver, including the actual admitted Credit/Boundary/Engine/output
lifecycle. That receipt is baseline evidence, not execution of the new cache
schema. The new 93-root source checks/build passes; focused peak controls and a
peak-bypass mutant have executed. The source-owned common-Graph natural-state
and byte-state Scan paths emit actual wire and invoke ProgramParse, Admission
and Engine. Their first run exposed a test-controller foreignRelease boundary
failure; the corrected parser release drain and cache/cancellation runs remain
under execution. Exact partial results are retained in increment93-progress.json.

The verification-only refinement adds general constructive output-ticket and
Ready handoff laws, including alias multiplicity and value interpretation.
The new cache-context module kernel-compiles and all 32 declared axiom sets
match; refinement-cache-kernel.json binds the exact source and generated module.
Neither these
handoff lemmas nor finite controls establish complete reachable-state simulation.
Actual canonical event coverage and return/accumulator binding rules, global
reference/ownership/prefix preservation, authentic parsed IR export using the
existing SDK reader, accepted native export and physical measurements, and the
locked-SDK whole-product acceptance boundary remain open.
