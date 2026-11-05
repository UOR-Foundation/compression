# Runtime decoder engine interface

`src/Uorc/Core/RuntimeMachineEngine.lex.tex` is authored LexLean source. It imports
`RuntimeMachine.prepare` for the symbolic opcode semantics and delegates sequence,
store, table, and byte construction to their runtime owner modules. It retains no
Graph, Rope, Trie, Bytes, copied capture list, or copied value list.

This document describes the current integration interface. Source checking and
kernel compilation are diagnostic evidence, not the repository's pinned SDK
acceptance boundary. End-to-end behavior and native cancellation qualification
remain under integration tests.

The Core-29 composition checkpoint is source-checked and built, but its ordinary
full-proof kernel attempt stops at the unchanged ReferenceSemantics leaf-index
proof. It has no new composition runtime or accepted-compiler qualification.
The earlier complete102-control native result binds Engine6be96651 and remains
historical for this newer source. Exact scope is in semantic-composition-source.json.

## State and transfer

The admitted `Ready` enters by move. The engine retains one arena, with fixed
program, caller-root, and usage shells; it retains no independent `Ready` or
arena snapshot. It has no input/EOF demand after admission. The terminal
`takeReady` move is the controller boundary; no active-machine detach, arbitrary
checkpoint validation, or controller restart protocol is introduced here.

- `startWithSemantic(ready, operationalBudget, semanticBudget, semanticLedger)`
  consumes the `Ready` tickets and preserves the supplied semantic ledger. This is
  typed ingress, not proof that a supplied ledger came from a valid execution.
  The first paid startup attempt checks profile, storage policy, ledger shape and
  current limits before allocating engine control. Existing arena ownership and
  operational budget are checked by `RuntimeResources.observeArena`.
- `start(ready, budget)` is a standalone component compatibility constructor. It
  explicitly creates a fresh current-profile ledger and unconstrained semantic
  budget with UInt64 maximum work. It has no production annotation and must not
  reset a controller's ongoing basis/decode/output scope. UInt64 representability
  is operational capacity, not a mathematical candidate bound.
- `tick(state)` performs one bounded event. Pending events and primitive failures
  consume work. `checkedEventWork` quotes the shared `RuntimeWork` operational
  policy before `paidEvent` dispatches the operation. Builder copy quotes use
  `RuntimeWork.builderStepFee` before any image read or commit.
- `extend(state, budget)` revalidates usage and updates the arena ceiling for
  running, cancelling or suspended work. Valid active updates preserve their
  disposition; only resource suspension resumes. Failed, completed and cleaned
  states are returned exactly unchanged before budget validation, including when
  the proposed budget is invalid.
- `semanticStatus(state)` exposes the independent typed semantic refusal channel.
  A controller must inspect it as well as `disposition`. A retained
  `Disposition.running` does not imply progress when this channel holds an error.
  Ordinary ticks then return the exact state without further work or effects.
- `extendSemantic(state, budget)` checks the proposed budget against the accepted
  prefix. Work/storage budget refusals can retry; word, profile, scope and
  invariant errors remain operationally incomplete and are not resumed. Terminal
  and cancelling states remain exact. This never rolls back the ledger or cursor.
- `cancel(state)` preempts a semantic refusal and selects cleanup. Cleanup
  preserves the semantic ledger and refusal, and remains subject to the work budget;
  `extend` can resume a cleanup suspended for resources.
- `takeOutput(state)` transfers exactly one completed output root ticket into
  `OutputTransfer.handle`. Its returned state has `valueOwned = false` and a null
  completed handle, so a second transfer is rejected.
- Cleanup preserves the program, basis, and caller tickets supplied in `Ready`.
  After `cleaned`, `takeReady` consumes the terminal engine state and returns those
  tickets with the current arena/usage as
  `Result<ReadyTransfer, Failure>`. The transfer contains `ready`, `semanticBudget`
  and `semanticLedger`; there is no parallel transfer which silently drops the
  semantic ledger. `OutputTransfer.state` already carries the authoritative ledger,
  so output transfer introduces no separate ledger copy.

`startTransferred(transfer, operationalBudget)` moves an actual `ReadyTransfer`
into `startWithSemantic` with the same semantic budget and ledger. It introduces
no arena, ledger reset or new controller.

`ownedRoots` is a transient inventory for `HeapAudit.auditRoots`: aliases appear
once per outside ticket. Heap's release cursor owns a transferred ticket, and the
Heap auditor enumerates that ticket itself. The engine inventory omits it.

`State.delta` contains the current tick's heap effect. `State.operationDelta`
retains the cumulative heap effect of the current or most recently accepted
native operation. Sequence, store and builder events compose this prefix with
checked additions. Table transitions retain their owner's cumulative delta in
`TablePending`; the per-tick report contains its increase. Failed native events
preserve the achieved arena, cursor and earlier successful allocation prefix.

If composition itself overflows, the engine freezes the preceding prefix in
`operationDelta` and the achieved event in `overflowEvent`, then reports typed
`program(operational.addressSpaceExhausted)`. Cancellation preserves both. Subsequent
cleanup still reports each event through `State.delta`; it does not claim a
flattened cumulative UInt64 total while the overflow receipt is frozen.
`takeOverflowReceipt` transfers `{prefix, event, state}`, clears the frozen receipt
and starts a new operation ledger. `takeReady` requires that receipt to have been
consumed. This fixed receipt preserves the accepted effect; it is no rollback
claim.

## Operational fee policy

The fee authority is `RuntimeWork.profileIdentity`, currently
`uorc.operational-task-fees/1`. Engine code calls that module's `fee` and
`builderStepFee`; it does not copy a second table of numeric tariffs. Quotes
inspect only fixed state tags, counters and borrowed metadata. They never run an
operation speculatively to discover its fee.

| Event | Shared operation or checked composition |
| --- | --- |
| Allocate engine control | `controlAllocate` |
| Enter frame | `indexedStart + controlAllocate` |
| Fetch immutable instruction metadata | `indexedStart` |
| Advance one semantic prefix position | `control` |
| Prepare after the complete paid prefix | `scalarCheck` |
| Prefix parameter type lookup and install | `indexedStart + tableMutation` |
| Capture validation, witness lookup and install | `captureCheck + tableMutation + indexedStart` |
| Initial/last-use bucket metadata lookup | `indexedStart` |
| Release-bucket slot and witness lookup, data erase | `indexedStart + tableMutation + indexedStart` |
| Install instruction result / pending data or witness table event | `tableMutation` |
| Action payload witness reads before dispatch | Action fee plus `indexedStart` |
| Erase witness after data erase | `indexedStart + tableMutation` |
| Retain returned local and read its witness | `indexedStart + heapRetain + indexedStart` |
| Publish output length | `sequenceLength` |
| Sequence begin, step, first cancellation | Corresponding `sequenceBegin`, `sequenceStep`, `sequenceCancel` |
| Compare initial sequence lengths | `sequenceLength + sequenceLength` |
| Store begin and subsequent owner steps | `indexedStart` |
| First Store cancellation flag transition | `control` |
| Builder begin, append/finish start, first cancellation | `builderControl` |
| Subsequent builder owner step | Checked `builderStepFee(cursor)` |
| Machine root release / release drain | `heapRelease` / `heapReleaseStep` |
| Pure ticket move / scheduler and iteration control | `smallControl` / `control` |
| Tick not dispatched because of disposition | `idle` |

Store's fixed-64 owner steps deliberately use the shared `indexedStart` flat
tariff. This is an operational policy choice, not a physical instruction bound.
At builder phase 2 the copied bound is one; at phase 4 it is `cursor.length`.
The checked quote is `builderControl + 2 * copiedBound`, as defined by
`RuntimeWork`. An unrepresentable quote pays only `smallControl` before typed
operational failure. Ordinary failed and pending attempts pay their selected fee.

Admission work already present in `Ready.usage.work` remains the cumulative
operational baseline. Heap ownership work in `delta.work`, consumed controller
credits, this operational Usage and the separate provisional canonical-cost
ledger are different quantities. The engine does not introduce a canonical-cost
ledger or a new mathematical-reference bound.

Sequence length, builder length, Store work-counter, engine counter, copy-quote
and operation-ledger exhaustion are operational `addressSpaceExhausted` failures.
For example, a compact shared concatenation DAG can have mathematical length
`2^64` without materializing that many bytes; a later one-byte slice may be valid
in the independent reference even though the handle runtime cannot represent an
intermediate length. Such a candidate is not semantically excluded. Original
Heap error details remain under `Failure.heap`.

## Capacity owners

Each retained frame cons has its own actual `Heap.allocateControl(4096)` owner.
The modeled 64-bit packed frame plus the cons spine is at most 816 bytes when
sum tags, option tags, booleans and bytes each occupy one word. This includes the
frame table shell, borrowed block and optional instruction metadata shells,
action, capture and release counters, iteration state, accumulator and its unit count, auxiliary units-table shell, semantic position, pinned caller roots, completion
origin, retirement and observation cursors, and fixed
builder. The reserve includes the complete control record and spine, rather than
only a symbolic frame marker. Frame data and units-table roots, accumulator roots, and builder roots
remain separate tickets and are enumerated with multiplicity.

The fixed engine control uses `Heap.allocateControl(8192)`, including the extra
72-byte operation ledger and optional 72-byte frozen event plus its tag.
The semantic ingress adds a modeled72-byte profile-tagged ledger,32-byte budget
and16-byte optional error; these fixed fields remain under the same8192 reserve. Its fixed pending
union includes the 64-entry table path, sequence plan, or builder cursor. A
`TablePending` retains cursor/outcome/cumulative delta/peak only. The sole arena is
`State.arena`; a table transition borrows that arena when passed to the owner API.
These charges describe the modeled packed representation, not physical RSS.

## Scheduling and lifetimes

Frame phases are fixed numeric tags in the authored source:

| Tags | Meaning |
| --- | --- |
| 0–2 | Prefix arguments, captures, and initial parameter releases |
| 3–8 | Fetch, prepare, execute, install result, and release last-use slots |
| 9–10, 15 | Invoke, consume child result, and wait for child |
| 12–14 | Retain return value, retire slots, and pop frame |
| 19–23 | Start, append, finish, and advance a byte builder |
| 24–25 | Compare left and right sequence bytes through resumable indexing |
| 26 | Retire a completed continuation's accumulator ticket |
| 27–29 | Install prefix, capture and instruction payload-unit witnesses |
| 30 | Erase a released binding's payload-unit witness |
| 31 | Retire the frame's units-table root before popping |
| 32 | Consume instruction/spec-role prefix; position5 prepares |

The engine forwards historical one-based local and backward-block distances
unchanged to the shared readers. Slot, instruction, capture-array and release
bucket positions remain zero-based table positions. Captures borrow the parent's
immutable instruction record and install one item at a time. Prefix parameters are coordinate first, accumulator second when
applicable. Canonical Scan operand 2 and capture lane A belong to the observer; operand 3
and lane B belong to the updater. Internally target A remains the updater and
target B the observer. The engine invokes the observer with the first lane, then
the updater with the second lane, including the final update. Fold and Tabulate
continue to use the first lane. `RuntimeProgram.initialReleases` supplies the
unused-parameter bucket; `releasesAfter` uses the zero-based instruction index.
Each released slot is erased with `RuntimeTable.beginErase`.

A new slot-table root is installed before the old root ticket enters resumable
release. Return and output values hold their own ticket before retiring the
frame's slots. Slot and instruction increments are checked before instruction execution and
before `RuntimeTable.beginPut`; capture and release-bucket increments are checked
before their corresponding table mutations. Commit uses checked increments rather
than `Heap.wordAdd`'s fallback. A finished frame with `nextInstruction` equal to
`instructionCount` enters return directly, including the UInt64 maximum boundary.
Builder copy-cost arithmetic is also checked before effects; an invalid cost pays
the shared `smallControl` fee before typed operational failure. A refused frame entry leaves the
parent at its retryable invocation phase; allocating the child control and entering the child are one successful
transition.

Sequence views delegate Slice/Concat; actual byte observations use native
`beginSpan` and `RuntimeImage.readSpanByte`. A nonempty span transfers one image
root ticket into the frame's observation descriptor and owner flag. Empty spans
transfer no owner. Metadata descriptors alone do not pin images. Each image read
is quoted and prepaid before the actual accessor. Observation images are retired
through `oldRoot`, with native cancellation retaining all pending owners.
Normal `RuntimeSequence.step` is guarded by nonempty frames, a quiescent old-root
slot and no outstanding observation image before it can return another owner.
Cancelled native cursors can still drain without a frame. Unexpected nonempty
outcomes preserve their transferred root at this guarded boundary; empty outcomes
never trigger an invented image release.

Sequence, store, table, and builder cancellation use their native owner
protocols. The engine clears a pending cursor only after the owner reports its
terminal cancelled outcome. A native cursor retains an in-progress release
cursor until that owner drains it.

The engine's typed failures are `RuntimeMachine.Failure`. The generated
`ComputeError` channel and unknown host fatal failures remain separate; no
rollback guarantee is asserted for those channels. Input/EOF interpretation
belongs to the surrounding controller.


## Charged payload-unit witnesses

Every frame owns an auxiliary `RuntimeTable` of natural-valued witnesses under
`RootRole.typeCache`. `FrameRecord.unitsTable` has its own root ticket;
`accumulatorUnits` and `State.valueUnits` move with their corresponding value.
A data-table put installs its root before beginning a separate witness-table
put. The logical slot count advances only after both commits. A last-use erase
similarly erases data first, then its witness before advancing the bucket index.
Both native cursors retain their achieved allocation and cleanup state on refusal.
Cancellation drains pending mutations and then retires both frame tables.

Witness reads use the shared one-based `RuntimeProgram.localIndex` and
`RuntimeMachineSemantic.unitsAt`; a missing or malformed witness is an operational
invariant failure. Empty sequence units are the valid value zero. Scalar units
come from `RuntimeMachineSemantic.fixedValueUnits`; sequence constants and basis
values use bounded cached sequence length. Alias, choose, parameters, captures,
returns and loop accumulators copy their source witness. Builder completion uses
the accepted builder byte count. These are charged runtime metadata witnesses;
the global relation to the provisional reference-cost profile remains an open
invariant, and no canonical event ledger is emitted by this integration.

A Store write reads `RuntimeStore.leafByte` only at genuine descent depth zero,
then checks `storeUnitsAfter` before the native owner can allocate. Its predicted
units commit only if that native step succeeds. Refusal preserves the previous
units, so retry cannot count an insertion twice. This uses a bounded leaf read,
not a scan of the store. The shared `indexedStart` Store tariff includes these
fixed owner reads as an operational policy.

## Exposed-event peak envelope

After the event fee is paid, `RuntimePeak.beginEvent` reads the actual pre-event
arena metric. `completeMeasuredEvent` retains that start and prior history with
`failedEvent`, then folds `completeEvent` over the achieved arena immediately,
before a queued old-root ticket is released on a later tick. Native table
`peakOwnedBytes` is also retained, since a table event can contain an intermediate
peak above its final arena. Metric errors preserve achieved state and history
and report their typed Heap error; they never substitute zero.

The imported Sequence and ByteBuilder endpoint argument is conditional on valid,
source-created and audited owner lineage. Store's separate source walkthrough has
one allocation, retain, release or release step in each exposed event: descent
observes or allocates one leaf/frame, ascent allocates one branch, cleanup drains
one release, and cancellation selects flags or drains one release step. Its work
counter guard precedes these effects. The outer engine defers ownership release
to later ticks. These facts explain the logical ownership metric envelope; they
do not establish host RSS, an arbitrary forged cursor theorem, or a universal
physical-memory bound.


## Prepaid instruction prefix

Phase3 fetches immutable instruction metadata and stores it without calling
`RuntimeMachine.prepare`. Phase32 then advances `FrameRecord.semanticPosition`:
position0 consumes the instruction event; positions1 through4 select the actual
`RuntimeOpcode.spec` operand0 through operand3 roles through the shared
`RuntimeMachineSemantic.prefixEvent`; position5 calls `prepare`. Local and block
references emit `referenceRead`; immediate and absent positions advance without
an event. Scan has five events including dispatch. No copied opcode event table,
event list, mathematical-natural cursor or reference-value backing is retained.

Each event calls the existing `RuntimeMachineSemantic.commit` before prepare or
its semantic effects. Rejection retains the exact accepted ledger, cursor,
instruction metadata, arena and ownership state. An accepted position advances
once. A later operational refusal preserves the paid continuation, including
position5 before prepare, so retry never charges those semantic events again.
All cursor/counter additions are checked before effects. A numeric exhaustion
is operationally incomplete and cannot exclude an otherwise valid reference
candidate. Cancellation and native cleanup do not synthesize semantic releases.

The graph-execution consumer now extends that prefix through the exact
`CanonicalCallComposition`, `CanonicalComposition`, `CanonicalRetirement` and
`CanonicalActionComposition` contracts from Core29. Output/basis context and
verified EOF observations still belong to the real controller/reader handoff;
they are not inferred from a final outcome. No complete product budget
admissibility or cache/reference invariant is claimed by these consumer bodies.


## Core29 composition cursors

Capture preparation runs once per instruction before call entry or loop setup,
including zero iterations. Two separate native retain events pin the immutable
caller slot root and its units-cache root, with the original slot count. If the
second retain fails, the achieved first arena, delta prefix and ticket remain
owned. Capture references are then consumed one bounded item at a time; Scan
prepares observer lane A before updater lane B. Each invocation physically rereads
those pinned roots, without another semantic capture read. Both pins are explicit
outside tickets in `ownedRoots` and are released before result installation or
through cancellation cleanup. Instruction metadata remains borrowed under the
owned immutable program root.

Every invocation prepays `blockEnter`, then each source-order parameter's
`parameterBind` and payload allocation before its data/cache dual commit. Main
entry has its own paid continuation across operational frame refusal. Completion
uses the actual Core `CompletionOrigin` enum: ordinary results allocate their full
cached payload once; Apply and final Fold/Tabulate/Scan results transfer their
existing binding. Handle equality, null handles and alias sharing never infer
origin. Last-use releases follow the existing checked bucket and cached units.

Return first prepays the reference read and holds the returned value and key.
A fixed retirement cursor visits ascending logical slot keys, skips the returned
key before lookup, and skips holes. It saves each live payload before release
prepayment, then emits `blockLeave` before bulk physical table retirement.
Retirement uses no copied binding list and no maximum-plus-one sentinel.

Loop setup pays the actual accumulator events; Scan creates output0 before its
state copy. Conditions include the final false condition. Observer append pays
write/growth/returned-byte release before native append; Scan's updater follows
that append, and its completed-iteration event follows state replacement.
Fold's final state and Tabulate/Scan's final output transfer without a second
length allocation. Scan releases its final state before output transfer.

Literal traversal prepays and actually observes each logical byte through an
owned image span. Index range failure emits no byte event; an in-range attempt
prepays one read before span acquisition and the actual image accessor. Basis
resolves and stores its operand before the extra lookup-reference event, then
calls the factored actual `RuntimeMachine.prepareBasis` lookup. Missing basis
entries retain that paid reference. Equal-length comparisons pay left read,
right read, then comparison before computing equality, stopping at the first
difference; unequal lengths have no pair events. Singleton prepays one write.
Native tree traversal, image pinning, builder chunks and physical cleanup do not
synthesize logical events.

The11 appended frame fields are two pinned roots, original slot count,
preparation lane, Core completion origin, retirement index, returned key, pending
units, borrowed span, observation-owner flag and span position. Their modeled
104 bytes increase the complete frame/spine bound from712 to816, still under the
existing4096 charged control owner. No State field, event list, reference Value,
Nat cursor or capture list is introduced. These remain logical packed charges;
a physical representation bound awaits accepted backend measurement.

This checkpoint passes official source linking/build against the actual full
Core29 authority declarations. The known unmodified
`ReferenceSemantics.leaf_index_step_refines_independent_sequence` proof failure
blocks ordinary dependency kernel and matched new consumer execution until the
separate Core repair. Source checking alone is not kernel qualification or an
accepted Rust/SDK export. Preimplementation controls already reject the earlier
zero-loop capture, missing fresh-allocation and missing index-read behavior;
matched runtime replay for this composition remains pending.
