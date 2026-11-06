<!-- uorc:non-authoritative -->
# Charged product comparison carrier

The authority is `Product.ComparisonOwner`, `Product.OwnedCompare` and the typed
controls in `Product.MachineBindingScenarios`.

`ComparisonOwner.start(terminal, budget, left, right)` moves one actual
`MachineBinding.Terminal`. Its owner sum contains either that terminal and one
fixed phase, or the usage left after handoff. The terminal supplies the sole
Arena, admitted program/caller roots and separately owned decoded output. The
operand handles and comparison spans are borrowed from those retained owners;
they do not create an extra outside ticket or retain another arena snapshot.
The enclosing source controller must supply reachable, pinned sequence operands.
These roots are internal typed transfers, never caller-provided JSON snapshots.

Start does not initialize a reader. Its first tick prepays the shared
`RuntimeWork.controlAllocate` fee and allocates an actual fixed control owner.
The 4096-byte reserve covers the packed fixed shell, two bounded readers/spans,
phase/disposition, one scalar observation plan and a delta receipt. It is a
logical capacity charge, not a physical RSS observation. Programs and payloads
remain separately charged in the same arena.

The left and right `OwnedRead.begin` observations occur on separate ticks, each
prepaid through `RuntimeWork.sequenceBegin`. A length mismatch therefore pays
both observations and still owns a real control object until cleanup. Neither
an unknown result nor an inherited cancelled/failed decoder initializes a
successful comparison.

During comparison, traversal steps retain the existing shared sequence-step
fee. The bounded arithmetic shell uses the shared control fee. Every attempted
image access is additionally prepaid with the actual arguments to
`RuntimeImage.readByteCost`; an error still consumes its own dispatch quote.
If the right quote is refused after a successful left read, `OwnedCompare`
retains the left scalar and checked geometry. Resume reads and charges only the
right side. A left-side failure pays no unattempted right access. Cancellation
clears the scalar plan and performs no further image reads. Pending observations
cannot satisfy `isEqual`, even if a terminal flag is malformed.

Each achieved heap event occupies `pendingDelta` until `takeDelta` transfers it.
Later ticks cannot overwrite an undelivered event. This is an internal serialized
controller step and requires no user approval. Resource refusal retains phase,
control, achieved usage and root custody. Extension resumes the same operation;
terminal heap failures do not restart. A failed initial allocation owns no new
control and can hand the original terminal back once. Release failures retain
their actual control or release-cursor custody.

Equality, byte mismatch, exact length mismatch, upstream failure, comparison
failure, incomplete work and cancellation remain distinct. Results select
control-only cleanup through `Heap.release` and bounded `releaseStep`. Once
release has begun, Heap's cursor owns the transferred ticket and the explicit
inventory omits it. Original/program/output tickets remain retained throughout.
`moveTerminal` transfers them once after cleanup and delta transfer; the source
carrier then contains no arena. Ordinary cancellation propagates the cancelled
flag into that returned terminal.

This pairwise owner compares retained runtime sequences. The archive controller
also has to route bounded file/spool rereads through the source ResourceIO
leases, including a small-memory raw path for large inputs; this owner does not
make full input materialization a format requirement. Pairwise equality is one
observation, not a successful archive Report, publication, scoped certificate or
global optimality claim. Current generated VM, native, proof and pinned SDK
qualification are recorded separately.
