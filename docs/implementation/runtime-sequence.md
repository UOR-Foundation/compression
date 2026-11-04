# Handle-backed runtime sequences

`RuntimeSequence.lex.tex` is the executable source. It imports RuntimeValue and Heap, retains no mathematical rope or copied payload, and uses no generated-code fallback. Its cursor is fixed control: 51 UInt64 words and one UInt8 byte of primitive payload, before outer-control overhead. Variable-depth traversal frames are charged arena metadata.

## API and ownership

- `length(arena, root) : Result UInt64 SequenceFailure`
- `beginConcat(arena, left, right)`
- `beginSlice(arena, root, start, count)`
- `beginIndex(arena, root, start)`
- `beginSpan(arena, root, start, count)`
- Each begin and `step(arena, cursor)` returns `Result SequenceTransition SequenceFailure`.

A transition contains the replacement arena/cursor, per-event `Heap.Delta`, one logical transition-work unit, and `pending`, `sequence(handle)`, `byte(value)` or `span({image,offset,length})`. Callers accumulate accepted deltas. This work count is separate from Heap ownership work, CPU instructions and fixed64-index traversal cost.

Begin captures the exact input IDs without consuming caller tickets. `capture`, `garbage` and `fresh` are owned external tickets whenever nonzero. `result` is owned exactly when `resultOwned(cursor)` is true; other cursor handles are borrowed. `ownedRoots(cursor)` supplies the nonzero ticket inventory for diagnostic audits. Add it to the independent caller inventory; exclude any root already transferred into Heap's release cursor because HeapAudit adds that ticket itself.

A nonzero terminal sequence or span image transfers one ticket to the caller and clears the cursor. Empty concat and full slice explicitly retain their known selected IDs. Equal payloads never establish alias identity. Span lookup returns the maximal first-leaf span bounded by count; advance by its length to stream the interval. Its owned image remains live after releasing the source sequence. Zero count returns null image and zero length. No operation copies payload bytes.

Cancellation may transfer capture/garbage/fresh plus result when `resultOwned`, replace the cursor with `emptyCursor()`, drain any active Heap release, then release those fixed cleanup slots. Keep only the returned arena/cursor lineage; never reuse an old cursor after accepting its replacement.

## Bounded transitions and failures

Metadata kind 513 is the charged spine owning operation-created temporaries; 514 encodes traversal frames; 515 captures inputs. Frames contain only fixed words and handle edges. Popping changes a borrowed handle; the charged owner keeps the old frame live until cleanup.

Concat descends the taller tree, records unchanged siblings, then rebuilds with single/double AVL rotations. A rotation plans at most three branches; a frame plans four metadata records. These are intrinsic fixed instruction bounds, not input caps. Planning allocates nothing. Each node allocation and each `allocateMoved` attachment to the temporary spine is a separate event: every begin/step performs at most one Heap allocation. Between them, the fresh root ticket remains observable in cursor.fresh. A later failure preserves that already-returned arena, delta and ticket; retry/cancel from the current prefix instead of rolling back to begin.

AVL balance is preserved for AVL inputs, so repeated concat from image leaves avoids linear-depth ropes. Arbitrary valid unbalanced inputs remain readable/sliceable; concat does not claim to normalize every old subtree. Slice emits fully contained subtrees by exact ID and partial leaves as new image-handle views, joining emitted pieces through the same machine. Index/span descend one child per event. Index reads directly from the image with a fixed 64-step scalar UInt64-to-Nat conversion.

Results are retained before cleanup releases temporaries and captures. Each cleanup event starts a release or handles exactly one pending reference. Separate start/drain phases reject an unrelated active release with heap(releaseInProgress). Temporary branches remain charged until operation cleanup, increasing peak live bytes. Production has no input-size/depth cap or hidden drain loop; only test drivers use finite fuel.

`SequenceFailure` distinguishes heap(HeapError), range, lengthOverflow, invalidShape and invalidCursor. Range checks subtract before adding; lengths/heights/offsets use checked additions. An error commits no replacement state for that event, while previously returned prefixes stay owned.

## Observed evidence and limits

Official language-1.2 linking, formatting, source build and Lean 4.32.1 compilation passed. All 60 definitions were individually traced: seventeen pure control/inventory helpers use []; the other 43 use exactly Quot.sound and propext. No custom axiom or sorry was introduced.

All nine runtime scenarios passed: binary concat/slice/index/span and alias behavior; 32 repeated appends with literal expected bytes and height <= 8; all five typed failure alternatives; foreign release; and exact-budget failed prefix. Every driver transition audits independent root-ticket inventory and rejects allocatedEntries > 1. Final reclamation reaches an empty arena. The 241-byte prefix scenario observes a successful 72-byte branch, then a refused 56-byte attachment, and audits/reclaims the fresh ticket. Overflow uses a 63-level shared binary DAG rather than a huge image. HeapAudit remains outside the production import closure.

Control source SHA-256: `3c7309525af5d48e88155caee9d98aa439a6d6dd7e59db4241000caf528fb642`; tests: `7c739a3d8363274276d87293069a38c47ac430a4dcf7aa59cea6a1a4e7b78acc`. A planted sliceWalk defect discarded cursor.start from the leaf-view offset. Mutant `4801f3cef5c1373a5782ee8440a3a587ec965c741b10128245bfe47be411d201` linked/compiled but exited 1 with binaryScenario failed; exact restoration rebuilt and passed. A compiler error or skipped assertion is not this mutation evidence.

The run used frozen diagnostic Heap SHA-256 `4e28ea84b2af3e9f73c9ae1bc79298ac9f77afd87031bda1501278cbdbab3b57`. Later arena/index changes require the composed integration rerun. Prepare a fresh fixture with tests/heap/prepare.mjs, copy these two modules, then run official lock, fmt --check and build on RuntimeSequenceTests. Compile emitted modules with exact generated dependencies; assert its suite in the pinned Lean runtime and print every definition's axioms.

This is scoped source/kernel/runtime evidence, not a general refinement theorem, accepted Rust export, or release acceptance. The known upstream certificate-B failure, product language-1.1 SDK migration and complete just vv remain integration boundaries. Product locks/template policy were unchanged.
