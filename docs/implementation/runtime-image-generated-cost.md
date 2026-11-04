# Independent RuntimeImage generated-cost review

The exact new accessor does **not copy the image payload** in the reviewed
SDK71 Std-generated path. Intermediate Entry/Node/image clones retain an
`Rc<[u8]>` backing through bounded header copies. Byte indexing and conversion
have a structural operation count bounded by the 64-bit index/word width,
independent of image length. This is diagnostic generated-code evidence, not an
accepted certificate/export or whole-product peak-refinement claim.

Reviewed RuntimeImage source SHA-256:
`7a3dbde81e2e3e08a3e59f0d2ab6a6f0f3f6fab5714991ddecd65894230ef77f`.
Heap is frozen at `ee41c3fb11c1f60fc767448760e8020d934c253fc31eec2146413ec1a98b0e1b`.
The existing official SDK rlib is byte-identical to the retained SDK71 artifact
(`71defb4c663aaec5b9a3aab04cbc6a03e7106227`). Shared source was not edited by this
review.

| Exact source root | Official lowering | Std Rust render / compile | Reachable functions |
| --- | --- | --- | ---: |
| `RuntimeImage.readByte` | success | success / success | 11 |
| `RuntimeImage.readSpanByte` | success | success / success | 13 |
| `RuntimeImage.imageLength` | success | success / success | 10 |

Only Heap, RuntimeValue and RuntimeImage were linked for these scoped roots.
The raw lowered calculus, layout, eligibility report, generated Rust and compiler logs are retained privately. This public checkpoint includes hash-only receipts and the exact source; the line references below identify those private, hash-bound evidence artifacts without publishing them.

## Exact generated path

The line references below refer to `readByte/program.rs` unless specified.

- Lines 646–654 define `#[derive(Clone)] pub struct Bytes(Rc<[u8]>)` and
  `Bytes::index(&self, i: u64)`. Indexing performs `usize::try_from(i)` followed by
  `self.0.get(at).copied()`. The primitive wrapper at 768–769 borrows that Bytes
  value for indexing.
- Lines 796–826 define Arena as `Adt0`, Index as `Adt1`, Entry as `Adt2`, and Node
  as `Adt3`. Index branches contain two `Rc<Adt1>` values. Entry contains a Node
  and three scalar counters; `Node.image` is `Adt3::C0(Adt4, Bytes)`. Thus the
  image-payload clone behavior is established separately from recursive Index
  sharing.
- `readByte` is `f0` at 900–914. It calls Heap lookup `f1` at 916–937, which calls
  `f4(64, ...)`. `f4`, at 1008–1057, is a tail loop over at most 64 index levels.
  Lines 1039–1041 clone the two child contents before choosing one. A branch
  clone retains Rc child pointers; a leaf clone can clone Entry/Node/Bytes
  headers, including the shared image Rc. Neither recursively clones the image
  backing.
- `nodeByte` is `f2` at 940–996. Its image branch moves the Bytes header into
  `index_bytes` after rejecting control kind. Lines 947–950 call
  `f6(64, absoluteOffset)`, then index one byte.
- `wordToNatAux` is `f6` at 1080–1097. Std Nat is rendered as `u64`; each of its
  64 levels uses a scalar bit/shift plus checked `nat_mul`/`nat_add`. Those
  primitives at 109–132 are checked u64 operations. There is no unary walk to
  the numeric offset and no image-length loop.
- `readSpanByte/program.rs` 905–962 adds only a scalar relative-index comparison
  and checked offset addition before calling `readByte`. It performs one image
  lookup, not a fresh span-length validation on every byte.
- `imageLength/program.rs` 940–956 reads the Rc slice length header and calls
  `wordOfNat`; 1079–1080 fixes that conversion to 64 levels, with its recursion
  at 1113 onward. It returns a scalar length, not Bytes.

There are no allocator or payload-copy operations in the emitted reachable
`f0...` functions for these three roots. Their used byte primitives borrow the
backing. The renderer's general runtime prelude also contains `Bytes::append`,
`Bytes::slice`, `Bytes::lit`, List builders and formatting routines that allocate;
none is called by these getter closures. Whole-file searches for those prelude
names alone would give the wrong conclusion.

## Small held-owner allocation probe

The probe used a temporary **source-owned LexLean setup helper**, not a
handwritten Arena. It calls actual `Heap.allocate(emptyArena(16777216),
Node.image(runtime, bytes))`. Official lowering/rendering produced both this
setup and the same `RuntimeImage.readByte` definition in a diagnostic program.
The Rust counter harness calls generated setup first, retains its returned
native Arena/image owner across every read, and measures only the generated
getter plus argument header clones. Payload construction, Heap setup, printing
and final owner destruction occur with counting disabled.

| Payload bytes | Offsets exercised | Samples | alloc / zeroed / realloc / dealloc / requested bytes |
| ---: | --- | ---: | --- |
| 1 | zero, last, end, UInt64.max | 4 | all zero |
| 1024 | zero, last, end, UInt64.max | 4 | all zero |
| 1048576 | zero, last, end, UInt64.max | 4 | all zero |

All valid reads returned the expected scalar byte; end/max reads returned the
source's range error. Exact observations are in
`getter-allocations.csv`. The test-only `GlobalAlloc` counter
lives in a separate harness; the generated library was left byte-exact and
continues to forbid unsafe code. Setup, generated program/layout and the counter harness are retained privately; their exact hashes are in `receipts.json`.

Holding the original native Arena/image owner is part of this measurement and
cost conclusion. Consuming the final native Arena owner can reclaim its index
and backing; that destruction was deliberately outside the getter event. A
numeric logical root counter alone is not a separate native Rc owner.

No SDK build, Library retry, old native suites, certificate acceptance, package
export, timing measurement, global RSS claim, or whole-product peak refinement
was performed. Public hashes, root/function maps, API and probe limits are in `receipts.json`; exact command arrays and raw compiler evidence remain private. New proof/test qualification remains separate from this immediate source/probe checkpoint.
