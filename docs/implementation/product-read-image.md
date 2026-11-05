<!-- uorc:non-authoritative -->
# Bounded transport lease to charged image

The authority is `Product.ReadImage`, its typed `ReadImageScenarios` and the
projected `HostContract`. This is the input bridge for the registered UC-STG-01
storage/resource operation. It contains no archive parser, codec or generic host
implementation.

`ReadImage.start(request)` constructs a cursor for an actual pending
`ResourceIO.ReadRequest`. The request fixes run, source, ticket, offset and
maximum extent. The cursor contains metadata, phase, three handle slots and an
achieved delta receipt; it contains neither an Arena nor payload Bytes.

The host first returns `LeaseReady`: matching request identity, an immutable
external lease, observed length, EOF and failure. This callback describes an
extent without copying its bytes into the generated runtime. Foreign run,
source, offset, ticket or lease instance, oversize, nonterminal empty progress,
unaddressable length and actual read failure remain distinct typed outcomes.
Rejected incoming leases are returned explicitly for cleanup without replacing
existing custody.

`leaseReady(budget, arena, usage, cursor, metadata)` and subsequent
`tick(budget, arena, usage, cursor)` move the one actual arena and usage. A fixed
4096-byte control is allocated after its shared work quote is paid. The exact
image capacity is reserved before `Outcome.requestPayload` authorizes payload
materialization. This capacity and the transport quantum bound one chunk; they
place no total input, file type, object-count or format-size ceiling on the
product. External spool/staging storage and physical observations remain
separate from logical arena accounting. The host must also admit its actual
copy buffers and external resources.

Only then does `commit(budget, arena, usage, cursor, PayloadReply)` accept the
same immutable lease's bytes. It checks callback identity and exact length,
obtains `Heap.preflightImageCommit`, prepays the actual shared commit/copy quote,
and calls the real `Heap.commitImage`. The committed image owns its immutable
bytes independently of the external lease. Work refusal preserves the exact
reservation, achieved usage and unresolved immutable callback. Extension/retry
uses that callback without rereading the source or replacing the lease.

Every mutation returns `Transition { arena, usage, cursor, delta, outcome }`.
The controller delivers the actual Heap delta before `acknowledgeDelta`; pending
deltas block further mutations. This acknowledgment is internal orchestration.
It adds no user-facing ceremony. Heap or resource failure keeps the achieved
arena, usage, reservations and cleanup ownership. No state is reconstructed
from an arena diff or a second admission snapshot.

`takeImage(arena, usage, cursor, run)` transfers one nonempty image root ticket
and the external lease exactly once in `Outcome.transferred(metadata, span)`.
It clears both from cursor ownership. Empty EOF transfers no image ticket.
`ownedRoots`/`outcomeRoots` and `ownedLease`/`outcomeLeases` expose the two custody
sets with their real multiplicity. An empty cursor does not erase an unconsumed
outcome ticket. `ResourceRef` is an external lease identity, never a cast Heap
handle.

The archive controller must move the image ticket into its charged producer or
CallerRoots aggregate. A real host attachment then associates that actual image
with a lease for `ResourceIO.ReadReply`; full consumer acknowledgment controls
release. The bridge does not forge a Reader reply from a resource token. The
closed archive orchestration belongs to the Wire owner, whose published parser
and admission revision is `d27be3e8f7dba73440ec29fa8dba236275162ca7`. This product
increment does not duplicate or assert integration of that later source graph.

Cancellation is same-run and idempotent. An in-flight initial read or payload
remains pending until an actual late callback or exact matching stopped receipt.
False, stale and unknown stopped receipts cannot authorize release. Charged
control/image/reservation cleanup uses actual Heap release and drain steps.
External lease cleanup emits `releaseLease` and waits for an actual matching
`LeaseReleased(released = true)` receipt. A late payload after cancellation is
not committed. Cursor retirement also requires that every unconsumed transition
outcome has settled.

The generic adapter interface is self-contained in
`validation/product/host-contract.json`, generated from `HostContract`. Its
sixteen `image*` roots specify metadata, materialization, achieved event delivery,
transfer, cancellation, stopped/release receipts, inventory and disposition.
They are internal typed ownership transfers, not public JSON snapshot inputs.
The native/library/Wasm binding must separately demonstrate that lowering
failure and outer ComputeError retain achieved owners. Source-level Transition
custody does not establish this ABI property; the older 22-root artifact is
historical and cannot stand in for the new closure.

The output bridge needs the shared Core checked operation
`readSpanBytes(Arena, BorrowedSpan) -> Result Bytes ImageFailure` and a
non-materializing `readSpanBytesCost(Arena, BorrowedSpan) -> Result UInt64
ImageFailure`. Both must validate the current live image handle and checked
range. The controller holds its actual owner, pays the quote before bounded
copy, and uses real partial-write/storage receipts. No private Heap matcher,
scalar-loop materializer or whole-buffer fallback is introduced here. The Core
owner is implementing that shared interface.

The exact diagnostic closure checks 21 source modules and kernel-compiles 24
Lean modules including the official SDK runtime. All 39 typed VM controls pass;
four actual source defects fail at their named controls; direct kernel queries
match all 111 product axiom declarations. These cover prepayment/refusal,
same-callback resume, stale and foreign identity, capacity, empty EOF, wrong
payload length, read/payload failure, one-time transfer, byte survival,
repeated cancellation and achieved cleanup. Initial missing-owner failure and
all exact diagnostic inputs are retained with
`validation/product/payload-image-evidence.json`.

The normal registered test executes `scripts/product-image-runtime.mjs` and
requires actual compiler verification; no committed receipt satisfies it.
Full verification of this older Core closure stops at the unchanged
RuntimeByteBuilder certificate-size guard, as recorded in the receipt. That
failure does not measure the newer published Core factoring. Complete
preservation/export, native/browser flows, closed operation/publication and
pinned-SDK acceptance are separate outstanding qualification work. Production
locks and the required `just vv` boundary remain intact.
