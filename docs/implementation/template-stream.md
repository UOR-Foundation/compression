<!-- uorc:non-authoritative -->

# Graph-free proposal streams

`Uorc.Search.FixedTemplates` stores a finite template tag, scalar parameters,
logical sequence handles and a syntax position. It retains no Graph, Rope,
Bytes or variable instruction collection. Static syntax emits the existing
`RuntimeProgram.Event` vocabulary. A literal control request names a logical
sequence, not a contiguous image. These families are discovery strategies;
the complete archive/program universe remains separate.

`Uorc.Search.TemplateStream` derives the literal length from the shared arena,
then uses actual `RuntimeSequence.beginSpan/step` transitions to traverse any
number of backing images. It prepays one declared source control/navigation
unit before each operation. The exact Heap ownership delta is returned
separately. This is not a physical CPU instruction count or a substitute for
RuntimeResources admission of Heap work.

A nonempty terminal sequence span transfers one owned image-root ticket.
`spanTicket` retains it while waiting for the full consumer acknowledgment.
After acknowledgment, stepped Heap release removes that ticket before the
next span is requested. Navigation cancellation and partially completed Heap
release are likewise resumed rather than dropped. Payload sequence roots
remain caller-owned dependencies throughout.

The seven closed `TemplateStreamControls` exercise real arena allocations,
one-image and two-image versions of the same logical literal, complete output,
navigation cancellation, terminal-ticket cancellation and non-advancement
before acknowledgment. Every tested state transition audits the complete
source root inventory. All seven reference-evaluate successfully. Separate
finite diagnostics compare exact literal bytes across partitions and observe
53 actual operation entries only after their source control credit is paid.

Planted source defects demonstrate failure: losing the terminal image ticket,
charging/advancing an unacknowledged waiting state, and allowing zero-credit
progress. Sources were restored byte-for-byte. Static-template tests also
reject a corrupted Scan capture lane. These are local source diagnostics, not
Lean proof, SDK acceptance or generated-native performance evidence.

Production integration still must bind actual admission completion to
acknowledgment, reserve the bounded producer control in the shared arena, join
its payload/span dependencies through registered owner roots, and pre-admit
its declared Heap work. No external checkpoint may resume this native state
without the separate exact identity/replay checks. The corresponding general
ownership, simulation and profile obligations are not asserted by these finite
controls.

## Authoritative operational task payment

`Search.TemplateWork` selects the shared `Core.RuntimeWork` operation before each actual sequence/control/Heap action. `Search.TemplateFunding` retains partial task payment without computing a future source state or arena. When the full fee is paid, it charges `RuntimeResources.Usage` before calling the actual operation, then observes the resulting arena. Excess offered credits are returned. Failure keeps the actual state, arena, ownership delta and committed usage. A task cannot silently take another partially paid task's credits.

The stream's small local transition counter, the declared operational task fee, and `Heap.delta` are different measures. None is physical CPU/RSS or canonical semantic membership. A concrete two-image three-byte control completed 101 actual operations with 79,808 declared task-work units and 79,808 consumed credits. A call-entry trace checked complete prepayment before each actual source call, including every sequence/Heap action beneath it; every tested ownership prefix audited. Eight closed source controls pass, and four planted underpayment/prepayment/failure-retention defects are rejected. These remain generic actual-source diagnostics, not generated-native or SDK proof qualification.

Actual full admission acknowledgment, the fixed owned control reservation, owner-root aggregation, and boundary take/reattach/split remain separate bindings. The controller must not treat the finite harness's simulated acknowledgment as production admission authority.

## Fixed template cursor ownership preparation

`Search.TemplateOwner` allocates the real shared control reservation and pins every fixed-template logical payload through registered search-owner metadata edges, one bounded action per step. It moves an existing search ticket into the aggregate without duplicating its root inventory. Literal data remains in any number of existing charged images, with no coalescing or graph AST. The ready token carries one arena plus basis/caller roots, usage and the fixed stream cursor. It is preparation, not admission success.

Five finite source controls cover one/two-image payloads, empty/existing search ownership, all tested prefix inventories, actual control reservation, payload incoming edges, and before/after allocation payment. Planted missing-reservation, borrowed-ticket move and skipped-pin defects are rejected. Preparation without an existing search owner consumed 1,472 declared task-work units and added 4,200 owned bytes including control/header/edge storage; preserving an existing search owner consumed 2,432 units and added 4,256 bytes. These small diagnostic figures are not native performance measurements.

Admission must consume the ready arena rather than retain an old preparation snapshot. Cleanup must restore any original search owner and stepwise release the producer aggregate only after actual full acknowledgment/cancellation. Those lifecycle operations are not silently certified by preparation.

## Actual retained-credit admission binding

`Search.TemplateAdmission` now binds prepared templates to the real
`RuntimeAdmissionCredit.fund` and `RuntimeAdmissionCreditBoundary` owners.
Only a fully acknowledged awaiting state may lend its arena. A produced
nonempty span's owned image ticket is moved into the registered search-owner
aggregate before reattachment, then remains there through the entire admission
operation and retained credit. After actual acknowledgment, bounded owner
splitting restores the producer root and image ticket before the stream can
acknowledge and release the span. The preparation arena is consumed, not kept
as a stale snapshot. Exclusive attached, detached, returning, splitting and
ready carriers contain only one arena.

Controller fees, producer funding and admission funding retain their separate
ledgers. Boundary operations already charge operational usage internally.
A successful terminal zero-fee admission observation consumes one positive
funding credit without increasing operational work, as explicitly classified
by `RuntimeAdmissionCredit.observationCredit`. These counters are not asserted
to be equal, and neither is canonical semantic membership or physical timing.
Failures retain the achieved owning carrier, counters and partial payment.

Fresh finite actual-source diagnostics after the one-based reference repair
completed the unchanged empty template in 769 wrapper steps and a three-byte
literal backed by two images in 880. The latter recorded 1,619,894 operational
work units and 1,619,895 total funding credits, including the named observation
credit. Every wrapper/boundary ownership prefix and sampled inner admission
prefix audited. Separate executable one/two-image full span-cycle controls use
5,681,937 and 7,958,035 generic evaluator terms, both within its unchanged
10-million diagnostic guard. These are not generated-native measurements,
SDK acceptance, universal ownership proofs, or archive eligibility results.

A planted early-admission-observation defect is rejected. Omitting span
aggregation or bypassing the owner split makes the guarded diagnostic fail to
finish within its evaluator allowance; this is retained incomplete negative
evidence, not falsely reported as a specific typed rejection. Restoring the
original source restores the two-image passing control.

Admitted readiness still retains the charged producer search aggregate.
Successful cleanup must restore the original search root and release the
producer reservation and pins; interrupted cancellation, wire reconstruction,
execution and canonical-cost refinement remain required bindings. The earlier
production-binding paragraphs describe these now-partially-closed obligations,
not an alternate simulated-acknowledgment implementation.

## Producer cleanup and restored handoff

`Search.TemplateRelease.start` consumes actual admitted readiness.
`step(cursor, credits)` performs at most one fully funded cleanup operation;
`released(cursor)` exposes `RuntimeProgram.Ready` only after complete cleanup.
An existing caller search owner is restored through registered owner splitting.
The actual control reservation is found through the fixed template dependency
chain and retained before producer pins are released. Its extra ticket remains
live throughout metadata/payload cleanup, then moves into the Heap release
cursor and is released last. No full Graph or literal snapshot survives in the
completed token. Partial payment and typed failures retain the achieved arena,
actual ownership inventory, cumulative usage and funding counters.

Fresh finite actual-source controls cover empty, two-image three-byte literal,
and two-image literal with an existing search owner. Cleanup takes respectively
6, 9 and 17 operations, charges 2,304, 3,840 and 5,888 operational work units,
and releases 4,144, 4,200 and 4,256 owned bytes. The admitted program and original
search ticket are preserved exactly. Every cleanup prefix audits, zero/partial
credit enters no Heap operation, and the control remains allocated through the
final transition. A work-limit failure retains the exact owned arena, usage and
fully paid pending task without performing the rejected Heap operation.

The actual-source stepped `TemplateReleaseControls` probe rejects planted
missing-control-retention, lost-original-search, and missing-operational-charge
defects; restoration passes. Its whole-trace generic evaluation exceeded the
unchanged ten-million-term diagnostic allowance, so the probe was independently
stepped at its source-defined bounded transitions. That incomplete whole-trace
observation is retained rather than represented as a passing larger guard.

These diagnostics used a recovered generic JSON evaluator derived from the
preserved runtime audit helper after transient scratch storage was replaced.
Its explicit ten-million-term per-call guard and source hash were recorded
with persistent local witnesses. Cumulative evaluator term counts and elapsed
diagnostic seconds are not operational work, generated-native timing, SDK
proof acceptance, semantic refinement, or archive eligibility. Full interrupted
cancellation and the owned wire/execution bindings remain required.
