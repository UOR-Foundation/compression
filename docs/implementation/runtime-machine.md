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

Metadata local/backward-block offsets are zero-based, as specified by
RuntimeProgram and RuntimeAdmissionChecks. The independent reference Graph uses
one-based distances; fixture conversion uses the explicit opcode operand roles.
No symbolic constructor order is used as a wire tag. Initial unused-parameter
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
