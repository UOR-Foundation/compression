<!-- uorc:non-authoritative -->
# Ordered Integrity execution contracts

The authorities are `Integrity.OwnedTranscript`, `ChargeContracts` and
`TranscriptControls`. They refine the existing reader/hash operations; they add
no archive controller, Product bridge, production byte buffer or wire field.

`StartedExecution` anchors actual `OwnedRead.begin` and `OwnedHash.startCharged`.
`OrderedExecution` relates sequential actual reader/hash calls, borrowed span
admission, byte admissions, failed attempts, cancellation and cleanup.
`ByteAdmission` requires the actual image read and child `Profile.beginUpdate`;
an arbitrary caller byte does not authenticate an admission. A reader event
in this relation requires the preceding hash input to be fully drained.
`OwnedHash` still accepts caller-selected spans; consumers must establish the
ordered relation for their executions.

`StartedView` derives a checked subrange from the actual reader origin without
allocating a slice. `CompletedView` requires actual selected-reader completion,
drained hash input and successful hash finalization. This proves the chosen
input boundary; Wire must establish the canonical preimage, stored digest and
whole-archive EOF on the same source. In particular, ArchiveBinding's zero
remaining-admission counter is insufficient while a final span is still pending.

`RetainedSnapshot` is an explicit consumer/Core premise: the retained root and
reachable nodes remain live and unchanged across arenas. It is not a new
runtime owner or a Boolean assertion of global Heap preservation. Ghost byte
lists, histories and natural counters occur only in propositions. The relation
does not yet prove its emitted digest equals canonical `Profile.hash` on that
byte list, nor establish origin authenticity or collision-free equality.

`ChargeContracts` uses the actual `RuntimeResources.charge` equations.
Successful work equals the prior natural work plus the charged event; sequential
basis/main events preserve the initial prefix. UInt64 overflow returns the
actual typed `workLimit` error. No failed replacement Usage is invented and no
counter is reset with `Heap.wordAdd`. Failed hash attempts and later cleanup
have separate work observations; a successful `charged.work` and its matching
`Transition.work` are charged once. These are hash-work contracts; uncharged
`OwnedRead.step` work and the complete controller ledger remain consumer-owned.
Post-operation accounting does not establish budget prepayment.
The universal `basis_main_work_exact` theorem establishes arithmetic composition
of two charges; it does not establish either event's purpose or completion order.

Twelve source-owned composed controls execute actual operations, including
pending-final-span EOF rejection, split headers, empty spans, cancellation,
prefix read failure, source order boundaries, basis-domain hash completion
followed by main failure, and late corruption. Fixture preparation is outside
their explicitly accumulated segments. Basis-domain completion is not a claim
about Wire's separate exact-basis identity implementation. Resetting the actual
failure-acceptance usage branch makes the prefix and basis/main controls fail.
The finite `basisThenMainFailureCumulative` control exercises basis completion
before the main attempt. `orderedReaderBoundary` exercises the pending/drain
guard; it does not construct a `CompletedView` witness or prove whole-archive EOF.
Changing the source reader origin or replacing checked work addition with an
overflow reset invalidates the corresponding unchanged kernel proof.

Scoped source builds, native kernel compilation, exact declaration audits and
leanchecker replay are recorded under `validation/integrity/transcript`.
The normal binding validation driver now selects these contracts and executes
the exact twelve controls; its complete SDK verification remains mandatory.
No known failed Heap certificate probe was repeated for this increment. Saved
compiler diagnostics are separate from original pinned-SDK and full archive
acceptance; existing locks, NIST response bytes and verifier limits are unchanged.
