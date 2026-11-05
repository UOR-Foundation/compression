# OwnedRead ordinary-image boundary

`OwnedReadImageScenarios.run` owns UC-WIRE-OWNED-READ-IMAGE-01 through -04.
The registered source owner was red before the reader change, green afterward,
red when the prior reader was planted in a disposable project, and green after
restoring the exact fixed reader bytes. The scenario source was unchanged across
all four runs.

Core `RuntimeImage.validateSpan` is the kind/range authority. It rejects a
control image with `ImageFailure.invalidShape`. `OwnedRead.step` now calls that
API on its actual image, offset and count, returns the validated descriptor's
image/offset/length fields, and maps the four Core image failures into its existing
failure algebra. `ArchiveIntegrity.byteAt` already delegates to
`RuntimeImage.readByte`; this checkpoint does not introduce another byte reader.

The six-module native projection contains Heap, HeapAudit, RuntimeImage,
RuntimeValue, OwnedRead and the new scenario owner. Controls exercise:

- Charged input/archive/basis/runtime images sliced at offset 1, length 2,
  preserving the actual image handle and span fields.
- Core `readSpanByte` values `20` and `30` (hex), the relative end-index `range`
  error, and a negative descriptor's `lengthOverflow` error.
- A negative-only control-image forge that changes only image kind while keeping
  backing bytes, length, entry weight and root counts. Core rejects it; the prior
  reader incorrectly exposed a span, while the fixed reader returns
  `invalidShape`.
- Malformed and overflowing leaf ranges, malformed cursor ranges/index, and a
  missing cursor handle, preserving typed `range`, `addressSpaceExhausted` and
  `heap` observations.

Negative `writeIndex` fixture mutations are confined to the scenario module.
They are not an uncharged production fallback or an admissible-image constructor.
Traversal fuel exhaustion is a separate test error, never a manufactured reader
failure. Fixture cleanup/root audits are local postconditions, not a general
retention theorem.

The accompanying public JSON records raw reader/scenario hashes, exact build IDs,
all six projected-source and generated Lean/document hashes for each run,
compiler/tool hashes, and command templates. Concrete private artifact locations
and command bindings are recorded separately and are not part of this public
checkpoint.

This is narrow candidate/native runtime evidence. It is not locked SDK acceptance,
`just vv`, production export, or a native/Rust physical-memory refinement claim.
Retained-source custody across handoffs, layout correspondence, and whole-stream
EOF proofs remain open under their respective owners. A local sequence cursor's
completion in these controls does not close those obligations.
