<!-- uorc:non-authoritative -->
Product.TargetDescriptor creates a charged immutable descriptor for an ordered byte sequence. The descriptor owns the sequence graph and the actual ordered-length table through Heap edges. Its construction cursor contains no Arena or Bytes; each transition returns the one actual achieved Arena and Usage.

The [SDK declaration extraction](../../validation/product/target-descriptor-api.json) records the exact types. The consumer signatures are:

```text
Parts { sequence : Handle, lengths : FrameEventContract.OwnedLengths }
Snapshot { record : Handle, sequence : Handle, lengths : OwnedLengths }
Transition { arena : Arena, usage : Usage, cursor : Cursor,
             delta : Heap.Delta, outcome : Outcome }
DeltaTake = stale(Cursor) | empty(Cursor) | taken(Cursor, Heap.Delta)
View { usage : Usage, outcome : ViewOutcome }
ViewOutcome = refused(Resources.Error) | viewed(Result Snapshot Error)

begin(run : Api.Run, parts : Parts) -> Cursor
tick(budget : Budget, arena : Arena, usage : Usage, cursor : Cursor, run : Run) -> Transition
takeDelta(cursor : Cursor, run : Run) -> DeltaTake
cancel(cursor : Cursor, run : Run) -> Cursor
take(budget : Budget, arena : Arena, usage : Usage, cursor : Cursor, run : Run) -> Transition
extend(budget : Budget, arena : Arena, usage : Usage, cursor : Cursor, run : Run) -> Transition
ownedRoots(cursor : Cursor) -> List Handle
outcomeRoots(outcome : Outcome) -> List Handle
releaseDisposition(cursor : Cursor) -> Api.ReleaseDisposition
deltaTakeRoots(movement : DeltaTake) -> List Handle
deltaTakeDisposition(movement : DeltaTake) -> Api.ReleaseDisposition
view(arena : Arena, root : Handle) -> Result Snapshot Error
readView(budget : Budget, arena : Arena, usage : Usage, root : Handle) -> View
```

Arena, Usage and Budget are the shared Core types. OwnedLengths contains the actual RuntimeTable.Table, count:UInt64 and totalOutputBytes:UInt64. Outcome is advanced / ready / clean / awaitDelta / snapshot(Snapshot) / failed(Error) / cancelled / refused(Resources.Error) / rejected(Error). Error distinguishes wrong run, not ready, invalid descriptor/table kind, sum overflow/mismatch, sequence-length mismatch, and actual read/length/Heap failures. The extracted SDK declaration JSON supplies all fields and exact qualified types.

Parts moves up to two nonnull actual owned tickets, sequence and lengths.table.root. Root0 empty values contribute no ticket. Cursor.ownedRoots reports those two when their ownership flags hold, followed by control, countWord, totalWord, kindWord, sequenceLink, lengthsLink, header, metadata, links and record when nonnull. The shared addRoot filters nulls and preserves equal-handle multiplicity: two actual tickets to one handle remain two inventory entries. Protected or borrowed handles contribute no extra owned ticket.

Construction prepays every observation, allocation and cleanup step with shared RuntimeWork fees. It accepts actual slots(wire) or metadata(FrameEventContract.lengthsMetadataKind) length tables. It incrementally checks actual sequence length, every ordered length, checked cumulative sum and exact total; there is no fixed input or object-count cap. One charged RuntimeOwnerRoots control covers the fixed cursor. Nine metadata nodes encode count, total, table kind, sequence link, length-table link and four branches. The final record owns both source graphs through charged edges.

Each achieved mutation stores its actual Heap.Delta in pendingDelta. An observation refusal after mutation retains the achieved Arena, Usage, cursor and event. tick and take stop at awaitDelta until takeDelta moves that event once. Wrong-run takeDelta returns stale with the original cursor; matching empty returns empty; matching nonempty returns the event and a cursor with pendingDelta cleared. A second take on the returned cursor is empty. Transition.delta and pendingDelta describe the same event and must not be folded twice. deltaTakeDisposition(taken(...)) retains receipt custody even when the returned cursor has no roots.

Cleanup releases the twelve ticket positions in inventory order, preserving only the successful final record. An unfinished Heap release is owned by the actual arena release cursor; draining uses releaseStep before cleanup continues. Once ready, a paid take with no pending receipt clears record from the cursor, marks done and returns Snapshot. Snapshot owns only record; sequence and lengths are borrowed children pinned by that record. outcomeRoots(snapshot) therefore contains one nonnull record, while other outcomes contribute no roots. Those other outcomes still return their achieved cursor/Arena owners.

Cancellation retains failure and pending receipt state. When keepRecord was true it rewinds cleanup to0, so even cancellation after ready releases the retained record. Active release remains draining. A blocked cursor remains retained. Release disposition retains receipt custody for pending deltas, image tickets for nonempty root inventory, and Arena custody for draining/blocked release state. Failed, refused or cancelled outcomes do not establish complete cleanup. Resource extension preserves the actual arena/usage prefix while changing its budget limit.

readView prepays the bounded descriptor observation and returns achieved Usage plus a refused or viewed result. Raw view requires separate prepayment. It checks the fixed descriptor shape and actual table representation; it neither retains a ticket nor repeats the full incremental lengths validation. The record owner must remain pinned in the same arena lineage while its borrowed children are used. Logical arena charges do not establish physical RSS or native ABI retention.

The exact diagnostic includes34 selected source modules and37 Lean modules including the official SDK runtime. Its26 closed VM controls pass, five general begin/guard laws compile, all93 source declarations have exact direct-kernel axiom annotations, and three actual source mutants are rejected. Annotation-only changes leave all34 generated Lean files byte-identical. Mutants were executed on that preannotation computational baseline; exact inputs and equality evidence retain this scope. The typed missing-owner probe reports LLR3005. Registered public gate sources are included separately; the whole pinned gate was not run. This is no accepted production export, native/Wasm/operational ABI, closed-archive integration or release qualification.

Requested external consumer interfaces remain separate from this existing descriptor API. OrderedObjectCompare proposes Candidate{source:Handle,start:UInt64,length:UInt64,lengths:OwnedLengths} owning up to two nonnull tickets and RootContext{program:Option RuntimeProgram.Program,caller:CallerRoots,extraBasisTicket:Handle}. The context owns up to two program roots, seven caller slots and the separate basis ticket; protected roles are an enclosing noArena mirror, not another owner. Candidate lengths must leave caller.verification once. The requested range path prepays actual Core.RuntimeSequence.beginSlice(arena,root,start,count), with the actual start and count, and step/cancel/ownedRoots custody; it compares the actual zero-based slice through unchanged OwnedCompare and releases the temporary slice while returning original candidate tickets. These OrderedObjectCompare signatures are requested, not implemented by TargetDescriptor.

Wire now implements the required pre-clean transfer at [d32f55b](https://github.com/UOR-Foundation/compression/commit/d32f55b71b9da3a47ce3dc1c8b17784c2d8975ab). Its authoritative moveDecoded must run before ObjectDemux advances; decodedAvailable is only a hint. The moved carrier supplies the sole Arena/Usage, actual source/start/length and owned lengths, remaining program/caller/basis tickets, protected-role classification and semantic/admission/receipt metadata. It does not supply run, operational Budget or free admission-credit balance; Search retains those correlated descriptors and Product accepts them explicitly. Final Product cleanup must release each trial-owned occurrence once and return exactly the protected caller occurrences. In raw mode the candidate archive ticket is source; in program mode it remains caller.archive. An equal protected predecessor handle still denotes a separate ticket. The descriptor diagnostic does not qualify this Wire transfer, ordered comparison or final cleanup integration.

The [source-bound receipt](../../validation/product/target-descriptor-evidence.json) identifies the complete public diagnostic archive and its exact input, generated-output and qualification scope.
