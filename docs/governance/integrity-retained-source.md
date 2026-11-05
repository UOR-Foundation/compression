<!-- uorc:non-authoritative -->
# Retained source accessor evidence

The authorities are `Integrity.OwnedTranscript.RetainedSnapshot` and
`Integrity.RetainedSourceLaws`. This checkpoint supplies accessor equations;
it does not establish a Wire session's ownership or parser invariant.

`NodeFrame initial current image` requires two successful actual Heap lookups
and equality of their nodes. It preserves `RuntimeImage.readByte`,
`imageLength`, `validateSpan` and `readSpanByte`, including typed errors.
`retained_nodeFrame` obtains this premise from `RetainedSnapshot` and actual
initial reachability from the selected root to the image. The live owner,
current reachability and unchanged initially reachable nodes are separate
requirements of that relation. Root membership or equal lengths do not prove it.

For the Wire consumer inspected at `2de910755e66a145891669d2d45a0c8154ea1b32`,
the minimal outstanding instantiation is:

- Inventory the actual archive ticket consumed by `ArchiveSnapshot.begin` and
  transferred by `takeSnapshot`, including real borrowed-span retain/release
  transitions. Establish the retained-source relation for each achieved arena.
- Establish the initial sequence shape, acyclic navigation and cached-length
  consistency, with validated leaf spans. Preserve the sequence layout and
  image nodes through the consumer's actual mutations.
- Bind the snapshot fields to the actual parser result and checked ranges.
  The selected preimage is `[0, preimageLength)` and the stored digest is
  `[preimageLength, archiveLength)` of that same archive root, with checked
  `archiveLength = preimageLength + 32` and the parser's body boundaries.
- Instantiate the actual reader origin and selected range from
  `OwnedTranscript.StartedView`. Establish selected completion with drained
  hash input, and independently establish accepted parser EOF and completion
  of the whole archive reader.
- Preserve the achieved arena, exact root tickets and attempted-work prefix
  on failure or cancellation. A canceled operation cannot supply a later
  successful publication witness.

`ArchiveIntegrity.byteAt` at that inspected commit still performs a direct
Heap lookup, image match and byte index. Its consumer needs non-control image
shape and index-conversion equivalence to use the RuntimeImage equations.
These equations do not already prove the direct consumer's refinement.

The same `NodeFrame` premise can preserve a committed digest image while its
control state is released. A complete publication bridge must also connect
actual `Profile.digest`, `Heap.commitImage` and the transferred digest ticket;
the node frame alone does not establish that connection.

All eleven declarations passed scoped source build, generated kernel checks,
exact axiom audit and leanchecker. Removing actual node equality from
`NodeFrame` emits source but causes the accessor proof to fail in the kernel;
exact restoration passes. Four finite mutation/restoration controls change
late preimage or stored-checksum bytes while roots, ownership audit, lengths,
span validity and the first byte remain unchanged. Their actual late reads
detect the changes. Evidence and identities are under
`validation/integrity/retained-source`. This is separate from full pinned-SDK
acceptance and does not repeat the known failed preservation probes.
