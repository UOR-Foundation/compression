<!-- uorc:non-authoritative -->
# Bounded output image delivery

Authorities are `Uorc.Product.WriteImage`, `WriteImageScenarios`, and the imported
`Uorc.Core.RuntimeImageCopy` contract. This note describes their frozen diagnostic
checkpoint; it is not an accepted export or operational host qualification.

`start(writer, span, ownership, maximum, ticket)` returns `Result Cursor Error`.
A successful transferred start moves one existing image ticket; a borrowed start
acquires one prepaid retain. Rejected starts leave input custody with the caller.
The cursor contains controls, handles, range/lease identities and a delta receipt,
with no Arena or Bytes. Each `tick(budget, arena, usage, cursor)` returns one actual
Arena/Usage lineage, actual Heap delta, cursor, and outcome.

The source pays `RuntimeImageCopy.quoteCost`, obtains `quoteChunk`, reserves a
real Heap staging-control ticket for the selected length, then pays `copyCost`
before `copyChunk`. Empty quotes skip copying. The shared 4096-byte limit bounds
one transfer chunk; larger spans loop through checked offsets and attachment
tickets. Large raw resources still require bounded input images and timely
release by their enclosing controller; retaining a large image is not a
small-memory claim.

`Outcome.attach(request, bytes)` hands one bounded payload to the generic adapter.
The adapter attaches an independently owned immutable lease and returns the exact
run/destination/ticket/extent correlation. It does not traverse Heap nodes or cast
ResourceRef into a Heap handle. Short writes reuse that same lease and advance
its remaining span. ResourceIO.Writer reserves destination Storage before I/O and
records actual acknowledged bytes, including failed partial writes.

Cancellation stays latched through late replies. Pending attachment/write effects
require exact stopped receipts or actual late completion; lease custody remains
until its exact release acknowledgement. Refusal leaves the same unresolved
immutable callback with its caller for retry. The image ticket stays pinned
through acknowledgement. Every actual Heap mutation creates a checked serial and
actual delta receipt. `acknowledgeDelta(cursor, run, serial, delta)` must match it
before another mutation. Checked identity exhaustion retains achieved owners.

After lease/control/image cleanup, `takeWriter(arena, usage, cursor, run)` transfers
`Outcome.settled(writer, sourceFailure, cancelled)` once. It preserves achieved
Storage. Settlement proves delivery/custody progress, not archive verification or
publication. Enclosing caller roots and unconsumed outcomes still require their
own custody inventory.

The frozen diagnostic has 53/53 VM controls, 24 compiled kernel modules, 162 exact
axiom policies in the new family, and five owning defect rejections. Its selected
Core files are the earlier four-Core `0ff59f51…` composition plus RuntimeImageCopy
from `8e7d815f…`, not the whole current integration. Actual full verify stopped at
`RuntimeByteBuilder.step`: certificate B was 6703926 bytes against 4194304, for
selected builder source `fc41954a…`. This does not describe the later factored
builder. Native ABI retention, NativeLibrary/Wasm, closed-archive operation,
pinned-SDK acceptance, physical RSS and whole-product peak bounds remain
unqualified. Logical Arena bytes, Storage, transport capacity and physical
observations remain separate.
