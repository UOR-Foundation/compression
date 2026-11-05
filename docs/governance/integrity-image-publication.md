<!-- uorc:non-authoritative -->
# Digest image storage and cleanup boundary

This note records the historical `8188c3d` qualification with Core `8bb13182`. Its receipts retain that exact scope. [Current-source requalification](integrity-current-source-requalification.md) records the separate replay against restored Core and current Wire.

The authority is `Integrity.ImagePublicationLaws`. Actual successful
`Heap.commitImage arena reservation bytes` yields a successful lookup at the
returned handle whose node is an image of those supplied bytes, with a
non-control image kind. `commitImage_readByte` gives the corresponding actual
RuntimeImage read equation. No digest-equality or assumed storage postcondition
is a premise. The proof derives the effect from actual allocation and same-key
trie read/write equations.

This supplies the storage link after OwnedHash's actual `Profile.digest` and
commit call. The emitted image additionally needs preservation while the
control ticket is released, plus the actual publication transfer. The existing
`RetainedSourceLaws.NodeFrame committedArena publishedArena digest` relation is
the precise content requirement: successful lookups and equal nodes. It does
not follow from live roots or unchanged lengths alone.

Integrity proves the narrow actual cleanup transitions. `releaseControl`
calls `Heap.release arena cursor.control`. `controlDrain` calls
`Heap.releaseStep arena`. The local drain contract uses the actual pending list
`[control]`, pending count one and root-step flag, an actual control-reservation
lookup, and the actual non-control digest-image lookup. Successful release and
drain must preserve the digest node; the childless control drain must finish
with an empty queue. These are transition-effect obligations, not a replacement
for Core's ownership model. `OwnedControlLifetime` establishes the actual
OwnedHash control reservation's lifetime and the queue's provenance through
the preceding hash operations, and composes this storage effect with cleanup.

All sixteen source declarations passed scoped build, generated kernel checks,
exact axiom audit and leanchecker. An isolated actual commitImage defect kept
32-byte preflight but allocated an empty payload. The native observation was
`inputBytes=32; preflight=ok; storedBytes=0`; the unchanged storage proof rejected
it. Exact restoration stored 32 bytes and passed proof replay. Source, tool,
closure and mutation identities are under `validation/integrity/image-publication`.
Original product locks, Core implementation, verifier limits and NIST bytes are
unchanged. Full pinned-SDK and product acceptance remain separate.
