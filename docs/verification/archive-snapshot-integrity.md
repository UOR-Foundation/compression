# Charged immutable archive verification stages

`Wire.ArchiveSnapshot` drives the actual `ArchiveParse`, reserved `LengthStore`
and `OwnedRead` controls in one arena. It consumes an already charged immutable
sequence ticket, retains one current image ticket for borrowed parser input,
commits every ordered length including zero, and requires the exact one EOF.
`takeSnapshot` transfers the archive and lengths tickets once. It reports framing
completion, not integrity acceptance or Core Ready.

`Wire.ArchiveIntegrity` moves that snapshot and the same arena through the actual
versioned `Integrity.ArchiveBinding`. It rereads the exact preimage, excluding the
32 trailer bytes, then compares all trailer octets over borrowed spans. Each
feed retains a separate image ticket consumed by the hash. Trailer spans borrow
the immutable archive root. A mismatch retains the digest and all achieved
cleanup state. Cancellation initiates once and steps the actual returned cursor.
Its cleanup releases hash scratch/feed/digest tickets and retains the archive
and length table for enclosing recovery.

One controller action charges 64 logical work units before integrity work: the
largest imported single hash report is 35, and bounded scalar/read operations
fit within the remaining allowance. Internal hashing does not emit product
output bytes. No recursive Graph, Rope, Bytes cache, signature cache or growing
object-length list is retained by these controllers. The immutable archive
snapshot is deliberately reread, and those reads are prepaid. Logical ownership
moves do not establish that an immutable host implementation physically frees
all predecessor copies; that is a separate qualification obligation.

The source-owned controls executed both profiles over 98 compact split cases,
including every trailer boundary; corrupt trailers; retained cleanup; and 80
hash cancellation prefixes. The framing owner executed exact ordered lengths,
body fences, transfer audits, 150 cancellation prefixes, resource pause,
truncation and trailing-byte controls. Source owners first ran red. The basis
adapter's representative empty-length omission is rejected by its canonical
identity gate; see `owned-basis-controls.md`.

These are implemented verification stages. The closed orchestrator through
conditional basis preparation, actual admission/engine execution, ordered output
and publication custody is still being implemented. No old archive-v1/profile02
compatibility, complete source export, full refinement, physical-memory bound or
locked-SDK acceptance is claimed. The existing Integrity APIs retain their exact
source and declaration eligibility; no executable assertion is fabricated over
unmarked imported definitions. `just model-write` and full locked-SDK `just vv`
remain required for acceptance.

Exact compiler identity, source hashes, build IDs and controls are in
`archive-snapshot-integrity-diagnostic.json`. The separate proof-only Option
probe passes check/build but fails Lean verification with LLV7002; its result is
in `option-linear-arithmetic-diagnostic.json`. No verified theorem, replay or
axiom audit exists for that attempted proof.
