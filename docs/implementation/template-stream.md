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
