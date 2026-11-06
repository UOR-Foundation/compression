<!-- uorc:non-authoritative -->

# Runtime decoder engine interface

`src/Uorc/Core/RuntimeMachineEngine.lex.tex` is authored LexLean source. It imports
`RuntimeMachine.prepare` for the symbolic opcode semantics and delegates sequence,
store, table, and byte construction to their runtime owner modules. It retains no
Graph, Rope, Trie, Bytes, copied capture list, or copied value list.

This document describes the current integration interface. Source checking and
kernel compilation are diagnostic evidence, not the repository's pinned SDK
acceptance boundary. End-to-end behavior and native cancellation qualification
remain under integration tests.

## State and transfer

The admitted `Ready` enters by move. The engine retains one arena, with fixed
program, caller-root, and usage shells; it retains no independent `Ready` or
arena snapshot. It has no input/EOF demand after admission. The terminal
`takeReady` move is the controller boundary; no active-machine detach, arbitrary
checkpoint validation, or controller restart protocol is introduced here.

- `start(ready, budget)` consumes the `Ready` root tickets and returns fixed state.
  The first tick allocates the engine control owner. Existing arena ownership and
  budget are checked by `RuntimeResources.observeArena`.
- `tick(state)` performs one bounded event. Pending events and primitive failures
  consume work. `checkedEventWork` quotes the shared `RuntimeWork` operational
  policy before `paidEvent` dispatches the operation. Builder copy quotes use
  `RuntimeWork.builderStepFee` before any image read or commit.
- `extend(state, budget)` revalidates usage and updates the arena ceiling for
  running, cancelling or suspended work. Valid active updates preserve their
  disposition; only resource suspension resumes. Failed, completed and cleaned
  states are returned exactly unchanged before budget validation, including when
  the proposed budget is invalid.
- `cancel(state)` selects cleanup. Cleanup ticks remain subject to the work budget;
  `extend` can resume a cleanup suspended for resources.
- `takeOutput(state)` transfers exactly one completed output root ticket into
  `OutputTransfer.handle`. Its returned state has `valueOwned = false` and a null
  completed handle, so a second transfer is rejected.
- Cleanup preserves the program, basis, and caller tickets supplied in `Ready`.
  After `cleaned`, `takeReady` consumes the terminal engine state and returns those
  tickets with the current arena/usage as `Result<RuntimeProgram.Ready, Failure>`.

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
| Fetch and prepare an executing instruction | `scalarCheck` |
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
The modeled 64-bit packed frame plus the cons spine is at most 704 bytes when
sum tags, option tags, booleans and bytes each occupy one word. This includes the
frame table shell, borrowed block and optional instruction metadata shells,
action, capture and release counters, iteration state, accumulator and its unit count, auxiliary units-table shell, and fixed
builder. The reserve includes the complete control record and spine, rather than
only a symbolic frame marker. Frame data and units-table roots, accumulator roots, and builder roots
remain separate tickets and are enumerated with multiplicity.

The fixed engine control uses `Heap.allocateControl(8192)`, including the extra
72-byte operation ledger and optional 72-byte frozen event plus its tag. Its fixed pending
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

The engine starts only sequence Index, Slice and Concat operations; its builder
starts Concat. It does not qualify Span consumption. An unexpected Span outcome
retains its transferred image ticket under `oldRoot` before protocol failure.

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
