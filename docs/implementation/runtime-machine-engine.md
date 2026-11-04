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
  consume work. Builder image commit events preflight the bytes copied as work.
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
`program(static.addressSpaceExhausted)`. Cancellation preserves both. Subsequent
cleanup still reports each event through `State.delta`; it does not claim a
flattened cumulative UInt64 total while the overflow receipt is frozen.
`takeOverflowReceipt` transfers `{prefix, event, state}`, clears the frozen receipt
and starts a new operation ledger. `takeReady` requires that receipt to have been
consumed. This fixed receipt preserves the accepted effect; it is no rollback
claim.

## Capacity owners

Each retained frame cons has its own actual `Heap.allocateControl(4096)` owner.
The modeled 64-bit packed frame plus the cons spine is at most 672 bytes when
sum tags, option tags, booleans and bytes each occupy one word. This includes the
frame table shell, borrowed block and optional instruction metadata shells,
action, capture and release counters, iteration state, accumulator, and fixed
builder. The reserve includes the complete control record and spine, rather than
only a symbolic frame marker. Frame roots, accumulator roots, and builder roots
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

Captures borrow the parent's immutable instruction record and install one item
at a time. Prefix parameters are coordinate first, accumulator second when
applicable. Scan invokes the observer from capture lane B before the updater from
lane A, and runs the final updater. `RuntimeProgram.initialReleases` supplies the
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
one control-work tick before typed failure. A refused frame entry leaves the
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
