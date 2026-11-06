<!-- uorc:non-authoritative -->
# Bounded image transfer chunks

The authority is Uorc.Core.RuntimeImageCopy, its source scenarios and source-owned
proofs. RuntimeImage remains byte-identical at 7a3dbde8. This additive API uses
the existing ImageFailure and BorrowedSpan types.

| Operation | Contract |
| --- | --- |
| chunkLimit | UInt64 = 4096. One transfer chunk, not an all-input limit. |
| quoteCost(arena, span, relativeOffset, maximumBytes) | Authoritative logical event work 1, before calling the quote. |
| quoteChunk(arena, span, relativeOffset, maximumBytes) | Result BorrowedSpan ImageFailure; validates the entire input span before selecting a chunk. |
| copyCost(quotedSpan) | Result UInt64 ImageFailure; rejects length above 4096, then computes checked 1 + length. |
| copyChunk(arena, quotedSpan) | Result Bytes ImageFailure; revalidates local handle, kind, full range and bound before bounded Bytes.slice. |

A quote checks image lookup and kind, then checked full-span end/range, then
relativeOffset <= span.length. Its result keeps the image handle, uses checked
span.offset + relativeOffset, and has length
min(span.length - relativeOffset, maximumBytes, 4096). Quoting does not slice
or materialize bytes. Zero requests and offsets exactly at the span end are
legal and return an empty quote.

Existing heap failures propagate through ImageFailure.heap; disallowed node
kinds, including control images, produce invalidShape; out-of-range spans,
relative offsets and oversized copy requests produce range; checked arithmetic
failure produces lengthOverflow. Full-span validation happens even when the
requested chunk would be empty. Copy validates the full quoted range before its
chunk bound. copyCost checks the bound before addition, so UInt64.max length
produces range, rather than overflowing the work addition.

The caller must use the **same Arena lineage**. Numeric handles are arena-local.
A span from a foreign Arena can collide with an existing numeric handle in
another Arena; this carrier has no global identity token to detect that mistake.
Each quote/copy validates the handle against the Arena supplied for that call.

Product must charge quote work, obtain the nonmaterializing quote, reserve
**real Product staging capacity**, and only then call copy with the quoted span
and its authoritative copy work. Core neither receives nor invents a reservation
ticket and does not prove that Product obeyed this ordering. Both the logical
image owner and the actual native Arena/image backing must remain pinned
through copy and acknowledgement. Quoting transfers no image ownership.
Returned Bytes are a separate copied value.

Product should skip copy for a zero-length quote. Calling copy for it remains
legal and has logical cost 1. The inspected native runtime allocates a 16-byte
empty Rc header in that case; empty copy is not free.

## Scoped evidence

The source operation/scenario IDs were registered before implementation. The
missing-operation control rejected the import with LLR3005. The 10 source
scenarios pass: exact/empty copies, relative/full-range priority, overflow,
shape/handle failures, chunk bound, costs and revalidation after actual release.
Disabling the copy bound guard makes overflowScenario fail; restoring the
exact source restores all 10 controls.

All 48 declarations in the new Main/Test/Proof family have exact axiom policies
checked against generated Lean. The 17 general theorems establish selection
bounds for arbitrary UInt64 arguments, quote/copy Arena-index frame laws,
null/control/nonimage rejection and exact pre-call/endpoint costs. They are not
an end-to-end Product staging-reservation or all-input copy-refinement theorem.

One official SDK71 diagnostic render produced the source-owned allocation probe
and actual getters in one generated carrier universe. The rendered Main
checkpoint was 960dd73e; final Main is aa1305d4. Their official semantic trees
differ only in boundedLength's overdeclared axiom list, corrected from
[Quot.sound, propext] to actual [propext]. All computational declarations are
identical. The render and native run were not repeated.

The held-native-Arena probe observed 40 events on payloads of 1, 4096 and
1,048,576 bytes:

- 18 successful quotes and 3 invalid-full-span quotes had zero measured allocator
  calls or requested bytes.
- 18 successful copies each made one allocation: 16 bytes for empty payload,
  24 for one byte, and 4112 for 4096 bytes on the inspected host/runtime.
- A valid-range, 4097-byte oversized copy rejected with zero allocation.
- Returned bytes and typed errors matched the source scenarios.

The measured region includes argument header clones and getter temporaries.
Payload construction, actual Heap allocation setup, input-span construction,
printing, result inspection, returned-Bytes destruction and final Arena/image
destruction are excluded. Original native Arena/image owners remain live.
The probe does not execute or verify Product staging reservation.

Exact source/config/compiler/program/artifact hashes, policies and counter rows
are in runtime-image-copy-evidence.json and runtime-image-copy-allocations.csv.
The language-1.2 diagnostic configuration is preserved in
runtime-image-copy-diagnostic-config; reproduce it in an isolated directory
containing the six source modules named in the evidence. It does not change
the repository's release project or locks.

These are source, kernel, finite-control and unqualified generated-code
observations. No certificate/export, Wasm/NativeLibrary, pinned-SDK, timing, RSS
or whole-product peak/no-copy qualification follows from them.
