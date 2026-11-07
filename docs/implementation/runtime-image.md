<!-- uorc:non-authoritative -->

# Checked scalar image access

RuntimeImage is the shared handle-backed scalar reader for consumers that
already pin an immutable image. It imports only Heap and RuntimeValue. Source
SHA256: `7a3dbde81e2e3e08a3e59f0d2ab6a6f0f3f6fab5714991ddecd65894230ef77f`.

| Source API | Request | Result |
| --- | --- | --- |
| readByte | Arena, image Handle, absoluteOffset UInt64 | Result UInt8 ImageFailure |
| imageLength | Arena, image Handle | Result UInt64 ImageFailure |
| validateSpan | Arena, image Handle, offset UInt64, length UInt64 | Result BorrowedSpan ImageFailure |
| readSpanByte | Arena, BorrowedSpan, relativeIndex UInt64 | Result UInt8 ImageFailure |

The exact production root for scalar access is
`Uorc.Uorc.Core.RuntimeImage.readByte`; the other names share that prefix.
ImageFailure is heap(HeapError), invalidShape, range or lengthOverflow.
BorrowedSpan contains exactly image Handle, offset UInt64 and length UInt64.
It has no Bytes payload, retained root ticket or ownership operation.

The caller must keep the image/sequence owner and originating arena alive
through the read. Handles are arena-local; a borrowed descriptor does not pin
its backing. For an owned RuntimeSequence span outcome, retain its separate transferred image ticket
until the consumer acknowledges all reads, then explicitly move or release it.
These accessors perform no Heap.retain, release, image transfer or allocation.
They return only a scalar or fixed descriptor; they do not add an input limit.

readByte wraps the actual fixed64 Heap.lookup and rejects non-image nodes,
reservations and control-kind payload images. Null/absent handles preserve the
lookup error under ImageFailure.heap. A legal payload image is indexed using a
fixed64 numeric conversion; an offset outside its actual Bytes gives range.

validateSpan first checks lookup, image kind and representable image length.
It then checks offset+length: overflow is lengthOverflow, and a nonoverflowing
end beyond actual image length is range. Success returns the exact original
handle, offset and length. Empty spans at the image end are allowed. Image
identity comes from the input handle; payload equality is not used.

readSpanByte first requires relativeIndex<span.length, then checks addition to
span.offset, then performs actual readByte. It does not repeat whole-span
validation on every byte. A manually forged descriptor still cannot read past
the actual backing; validateSpan is the API for checking its complete extent.
Relative range errors precede lookup errors in this operation.

Each API has a matching Cost function with identical arguments and UInt64
result. Every quote is one logical accessor dispatch, charged before success
or typed error. This is source-owned operational accounting, not an instruction
count, physical allocation quote or elapsed-time measurement. The declared SDK
effect envelope is conservative; it does not imply an actual Heap allocation.

Official formatting/source build, generated Lean kernel and all 15 exact axiom
policies pass; see [the production-source receipt](runtime-image-production-evidence.json).
Only the three getter roots were independently lowered/rendered and compiled
for the scoped generated-cost review. Actual generated Entry/Bytes clones use
Rc-backed headers. A source-owned setup and held-arena probe measures zero
getter allocator events in twelve byte/range cases across three payload sizes.
See [the line-cited review](runtime-image-generated-cost.md),
[hash receipts](runtime-image-generated-cost-evidence.json) and
[all observations](runtime-image-getter-allocations.csv).

This observation applies to the exact measured getter with a held native owner.
Setup, backing construction and final-owner destruction are excluded. It does
not qualify every byte-reading path, whole-product memory, timing, accepted
certificates/export or the pinned SDK release.

The additive RuntimeImageProofs and RuntimeImageTests now pass source/kernel
checks and all52 additional exact declaration policies. The safe proof subset
covers real-API index frames, four pre-call cost laws, scalar/control/null
semantics, general relative-bound/end rejection and explicit resolution
equations. Nine source-owned scenarios cover binary bytes, wide/end offsets,
span translation/overflow, empty images, identity and caller-pinned reads.
Removing the relative bound guard makes their suite fail; byte-exact restoration
passes. The [additive receipt](runtime-image-additive-evidence.json) records all67
production/proof/test declaration policies and their separate scope. Universal
successful Option/Result projection, general numeric-conversion refinement and
the full Heap invariant remain unproved; failed attempts are excluded.

The exact language1.2 [diagnostic configuration snapshots](../../tests/heap/diagnostic-projects/README.md)
are retained beside this evidence and the private generated artifacts. The
product's locked language1.1 configuration is unchanged. Direct rust-std root
annotations do not restrict caller targets transitively; NativeLibrary/Core-Wasm
closure/signature/IR/ABI acceptance requires independent qualification.
