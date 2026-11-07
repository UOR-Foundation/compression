<!-- uorc:non-authoritative -->

# Handle-backed runtime sequences

`RuntimeSequence.lex.tex` is the executable source. It imports RuntimeValue and Heap, retains no mathematical rope or copied payload, and uses no generated-code fallback. Its cursor is fixed control: 51 UInt64 words and one UInt8 byte of primitive payload, before outer-control overhead. Variable-depth traversal frames are charged arena metadata.

## API and ownership

- `length(arena, root) : Result UInt64 SequenceFailure`
- `beginConcat(arena, left, right)`
- `beginSlice(arena, root, start, count)`
- `beginIndex(arena, root, start)`
- `beginSpan(arena, root, start, count)`
- `cancel(arena, cursor)` begins resumable cancellation.
- Each begin, cancel and `step(arena, cursor)` returns `Result SequenceTransition SequenceFailure`.

A transition contains the replacement arena/cursor, per-event `Heap.Delta`, one logical transition-work unit, and `pending`, `sequence(handle)`, `byte(value)`, `span({image,offset,length})` or `cancelled`. Callers accumulate accepted deltas. This work count is separate from Heap ownership work, CPU instructions and fixed64-index traversal cost.

Begin captures the exact input IDs without consuming caller tickets. `capture`, `garbage` and `fresh` are owned external tickets whenever nonzero. `result` is owned exactly when `resultOwned(cursor)` is true; other cursor handles are borrowed. `ownedRoots(cursor)` supplies the nonzero ticket inventory for diagnostic audits. Add it to the independent caller inventory; exclude any root already transferred into Heap's release cursor because HeapAudit adds that ticket itself.

A nonzero terminal sequence or span image transfers one ticket to the caller and clears the cursor. Empty concat and full slice explicitly retain their known selected IDs. Equal payloads never establish alias identity. Span lookup returns the maximal first-leaf span bounded by count; advance by its length to stream the interval. Its owned image remains live after releasing the source sequence. Zero count returns null image and zero length. No operation copies payload bytes.

`cancel` starts source-owned cleanup; drive `step` until `cancelled`. It releases fresh, garbage, capture and any produced owned result, one reference per event, then clears the cursor. Repeated cancellation requests are idempotent. A failed cancel/step leaves the caller's exact prior arena/cursor available, including achieved allocation prefixes and tickets. Terminal phase 10 rejects cancellation because any outcome ticket has already transferred to the caller. No fields or owned handles were added: purposes 4..7 mark cancellation and phases 20..28 perform cleanup. `resultOwned` and `ownedRoots` include its produced-result ticket until transfer into Heap's release cursor.

Use the returned cursor paired with its returned arena. During own-release phases 6/11, that arena's embedded release cursor is the ticket transferred by that same sequence lineage. Cancellation drains it before clearing or redirecting borrowed fields, and never inventories it twice. The caller must not independently advance, replace or substitute that pending release while retaining the sequence cursor. This serialized pairing is a precondition; the fixed cursor stores no second release identity. Independent foreign releases can be started in other phases, but must finish before sequence work resumes; active foreign releases are rejected with `heap(releaseInProgress)`. A larger owner may still transfer the four documented ticket fields into its own fixed cleanup slots, clear the nested cursor with `emptyCursor()`, and take responsibility for any already-transferred Heap release.

## Bounded transitions and failures

Metadata kind 513 is the charged spine owning operation-created temporaries; 514 encodes traversal frames; 515 captures inputs. Frames contain only fixed words and handle edges. Popping changes a borrowed handle; the charged owner keeps the old frame live until cleanup.

Concat descends the taller tree, records unchanged siblings, then rebuilds with single/double AVL rotations. A rotation plans at most three branches; a frame plans four metadata records. These are intrinsic fixed instruction bounds, not input caps. Planning allocates nothing. Each node allocation and each `allocateMoved` attachment to the temporary spine is a separate event: every begin/step performs at most one Heap allocation. Between them, the fresh root ticket remains observable in cursor.fresh. A later failure preserves that already-returned arena, delta and ticket; retry/cancel from the current prefix instead of rolling back to begin.

AVL balance is preserved for AVL inputs, so repeated concat from image leaves avoids linear-depth ropes. Arbitrary valid unbalanced inputs remain readable/sliceable; concat does not claim to normalize every old subtree. Slice emits fully contained subtrees by exact ID and partial leaves as new image-handle views, joining emitted pieces through the same machine. Index/span descend one child per event. Index reads directly from the image with a fixed 64-step scalar UInt64-to-Nat conversion.

Results are retained before cleanup releases temporaries and captures. Each cleanup event starts a release or handles exactly one pending reference. Separate start/drain phases reject an unrelated active release with heap(releaseInProgress). Temporary branches remain charged until operation cleanup, increasing peak live bytes. Production has no input-size/depth cap or hidden drain loop; only test drivers use finite fuel.

`SequenceFailure` distinguishes heap(HeapError), range, lengthOverflow, invalidShape and invalidCursor. Range checks subtract before adding; lengths/heights/offsets use checked additions. An error commits no replacement state for that event, while previously returned prefixes stay owned.

## Observed evidence and limits

Official language-1.2 linking, formatting, source build and Lean 4.32.1 compilation passed. All 85 definitions were individually traced: 22 pure control/inventory helpers use []; the other 63 use exactly Quot.sound and propext. No custom axiom or sorry was introduced.

All thirteen runtime scenarios passed against the final foundation across the scoped runs: binary concat/slice/index/span and alias behavior; 32 repeated appends with literal expected bytes and height <= 8; all five typed failure alternatives; foreign release; exact-budget failed prefix; and cancellation. The phase-coverage gate cancels every state on representative concat, slice, index and span paths and requires all twelve normal phases and all nine cancellation phases. It audits every intermediate state, including own pending release phases 6/11 and retained produced results. The foreign-release gate checks exact releaseInProgress at phases 7/8/9, audits and drains that foreign ticket, then resumes cancellation with the unchanged sequence cursor. Every driver transition audits independent root-ticket inventory and rejects allocatedEntries > 1. Final reclamation reaches an empty arena. The 241-byte prefix scenario observes a successful 72-byte branch, then a refused 56-byte attachment, and audits/reclaims the fresh ticket through public cancellation. Overflow uses a 63-level shared binary DAG rather than a huge image. HeapAudit remains outside the production import closure.

Control source SHA-256: `c352ecaf1eb161df6df7c6f0fed7fc7868d00e4fc9fe3590271d40604785a29e`; tests: `09a3bf982a3288a4b5969b782fd06ee165d3af469767350f8c26a9e0264d643c`. A planted prepareCancel defect discarded a produced owned result ticket. Mutant `ae97377110a8fe1b432183a18a5bccb5c95268c8e71488cd88981545ab25f874` linked/compiled but exited 1 with foreignCleanupScenario failed; restoring the exact control hash rebuilt and returned foreignCleanupScenario passed with exit 0. The experiment used an isolated fixture; repository controls remained unchanged. A compiler error or skipped assertion is not this mutation evidence.

The scoped runs used the MSB/control/backing-guard foundation with Heap SHA-256 `8f3880d42cc0294356bfbee1aa18e7e95fa4f9bfdcda63199523a366d304ae05`. The final fourteen-module native diagnostic also passes with the subsequent `withdrawRootRelease` foundation at Heap SHA-256 `8bb1318242ce855b266a7b0f076a0c49f28c7ddd742bd902c937534cfb32fe81`; `runtime-composed-evidence.json` binds that complete snapshot. The whole product machine still requires its integration qualification. Prepare a fresh fixture with tests/heap/prepare.mjs, then run official lock, fmt --check and build. Compile emitted modules with exact generated dependencies; assert its suite in the pinned Lean runtime and print every definition's axioms.

This is scoped source/kernel/runtime evidence, not a general refinement theorem, accepted Rust export, or release acceptance. The known upstream certificate-B failure, product language-1.1 SDK migration and complete just vv remain integration boundaries. Product locks/template policy were unchanged.
