# Handle-backed sparse store

Capability `runtime.store.handle-backed.v01` is defined by `src/Uorc/Core/RuntimeStore.lex.tex`; its executable acceptance scenarios are in `RuntimeStoreTests.lex.tex`. The production closure imports only `RuntimeValue` and `Heap`. It does not import or retain mathematical `Core.Store`, `Trie`, or `ReferenceStore` values.

The source operations are:

```
beginRead(arena, defaultByte : UInt8, root : Handle, key : UInt64)
beginWrite(arena, defaultByte : UInt8, root : Handle, key : UInt64, newByte : UInt8)
step(arena, cursor : Cursor)
cancel(arena, cursor : Cursor)
  : Result StoreTransition StoreFailure
ownedRoots(cursor : Cursor) : List Handle
```

`StoreTransition` contains `arena`, `cursor`, `delta : Heap.Delta`, `work : UInt64`, and `outcome`. Outcomes are `pending`, `byte(UInt8)`, `store(UInt8, Handle)`, or `cancelled`. Each successful transition charges one logical dispatch event in `work` and checked cumulative `cursor.logicalWork`. Heap ownership work and allocation/reclamation changes remain separately reported in `delta`; this is logical work, not a CPU or elapsed-time measurement. Source failures are `heap(HeapError)`, `invalidShape`, `invalidCursor`, and `workOverflow`. Heap failures preserve their original count, ownership, budget and release-protocol categories.

Only the returned arena/cursor pair replaces the current state. A failure returns no replacement arena, so the input pair remains available for cancellation or diagnosis. A completed cursor must not be stepped or cancelled again. Cancellation starts a fresh checked cleanup-event counter, so an exhausted operation counter cannot prevent ownership cleanup; every cleanup transition still reports its own logical work and heap delta. Cursors and handles are local to the single arena context; the caller must not forge, copy as an additional owner, replay, or mix cursors from different snapshots. The public first-order records are a machine integration representation, not an untrusted serialization format.

## Addressing and shape

The sparse store consumes the least-significant key bit first, independently of the arena's internal handle index and the runtime table index. It uses `Heap.lowBit` and `Heap.nextKey`, preserving the mathematical `Core.Store` addressing convention even when the internal indices change. The initial depth is exactly 64. A null child reads the supplied default byte. Allocated exception leaves occur only after all 64 bits have been consumed; a visited branch at leaf depth or leaf at branch depth produces `invalidShape`. An allocated default-valued exception leaf is noncanonical and rejected. Each operation validates its visited path; it is not a whole-store admission validator for unvisited siblings.

Changed writes allocate a fresh leaf for a nondefault byte or a null subtree for a default byte. They rebuild one ancestor per event, preserving the IDs of unchanged siblings and compacting a pair of null children to null. Leaves with equal byte payloads are never compared to infer identity. After observing equality at the addressed byte, an unchanged write transfers the explicitly retained capture of the known original root to its result.

## Ownership and charged continuations

`beginRead` and `beginWrite` borrow the caller's root and retain one separate capture ticket. `Cursor.captured` owns that ticket through the operation; `current` is a borrowed descendant reachable from the capture. The caller can release its original ticket while the operation runs. An actual result store owns its returned root ticket. The terminal cursor clears every ticket transferred to the result.

Write descent allocates at most one `Heap.Node.metadataBranch` per edge. Internal metadata kinds `256` and `257` record left and right descent respectively. The first edge owns the unchanged sibling and the second owns the previous frame. Each frame costs the heap's declared 56 modeled bytes. Allocation moves the previous frame's temporary root into the new frame. The path has at most 64 frames, and the cursor contains only a fixed number of scalar fields and handles. There is no free dynamic list of values, siblings or mathematical tries. The small list returned by `ownedRoots` is an auditing inventory of at most five fixed handle fields, with at most four populated in reachable states, not production backing for the path.

Unwinding uses separate events to retain the previous frame root, allocate at most one replacement store branch while moving the new subtree root into it, and transfer the consumed frame to `Heap.release`. The fixed `previous` and `retired` handle fields preserve the tickets between these events. Later events each perform exactly one `Heap.releaseStep`. No store event allocates more than one heap entry. Work overflow and the moved temporary root ticket are checked before allocation; no fallible heap operation follows the allocation in that event. The capture keeps unchanged store descendants live, and the retained previous-frame ticket prevents release from silently consuming the remaining continuation. The new subtree root remains owned until its parent consumes it or the terminal result receives it.

Cancellation releases the charged path, any retired frame and retained previous-frame ticket, the partial result subtree, and the captured root in separate phases. It also drains any release already transferred to the arena cursor, one release step per event. A shape/budget failure can therefore be cleaned up without an unbounded synchronous drain. Allocation and retain are not attempted while a heap release is active.

`ownedRoots` enumerates the capture, path, subtree, previous-frame and retired-frame tickets still owned by the operation, omitting null handles. Reachable states hold at most four simultaneous tickets; the two frame temporaries are never simultaneously populated. Qualification tests combine that inventory with an independently maintained caller inventory and an explicit terminal result ticket. `HeapAudit.auditRoots` adds the arena's transferred pending root itself; the store cursor clears its corresponding field at release start to avoid counting the ticket twice.

## Validation scope

The source scenarios check literal expected bytes for persistent writes, missing keys, key zero and `UInt64.max`, default erasure and empty compaction, alias preservation, exact root identity on unchanged writes, capture survival after the caller releases its ticket, cancellation, early-leaf and branch-at-leaf-depth shape rejection, budget failure, count overflow, work overflow followed by cancellation, cancellation during active frame and capture release, terminal-cursor protocol rejection, a branch-budget failure after preserving the already allocated 64-frame/leaf prefix, and final reclamation to zero. The test runner audits the independent external ticket inventory at every continuation and release event.

This is finite execution and scoped kernel evidence, not a general refinement theorem between every runtime store and the mathematical reference semantics. Whole-store admission validation, composition into the machine's owner partitions, physical allocator/RSS accounting, product model/registry integration, locked-SDK migration and complete `just vv` remain integration obligations. The product locks are unchanged. Full Rust production export remains subject to the separately tracked official LexLean preservation-certificate-B regression; no compiler check is bypassed and no accepted generated Rust execution is claimed here.

## Diagnostic evidence

The isolated language-1.2 fixture uses the official LexLean 0.4.0 compiler at source commit `71defb4c663aaec5b9a3aab04cbc6a03e7106227` and Lean 4.32.1. Official source linking, build and canonical formatting pass. SDK `read_module` followed by `write_module` reproduces both authoritative source files byte-for-byte. No product lock or SDK implementation is replaced.

The generated Lean definitions and the two source-owned closed overflow conformance theorems elaborate in the Lean kernel. Exact axiom tracing matches all 67 declaration policies: each is either empty or exactly `{Quot.sound, propext}`. No `sorry`, custom axiom or additional trusted assertion is used. Twelve source scenarios execute successfully through Lean's native compiler and runtime. The native harness only imports the SDK-generated test module, evaluates its scenario definitions and fails the process if one returns false; it adds no store behavior.

The representative planted defect changes the source default-erasure comparison from `newByte == cursor.defaultByte` to `newByte == 0`. It still links, but the executable `persistentScenario` gate returns false and the assertion process exits 1. Restoring the byte-identical control source restores all twelve scenario results. Kernel elaboration, exact axiom tracing, executable scenario results, and Rust production export are separate evidence levels.

The source test `prefixBudgetScenario` sets a 3624-byte heap budget: 64 charged path frames at 56 bytes plus the 40-byte exception leaf. The next 48-byte branch allocation is refused. The source test independently audits the preserved 65-entry arena and all continuation root tickets before cancelling it to zero. Thus the failure does not erase the achieved allocation prefix or require an uncharged dynamic cleanup list.

The final stable-foundation diagnostic run has semantic ID `f1f2012933d5fd8e3fdf0a9fdd38247aa1f5f9bf5b459a429bfaeec57907cd47` and source ID `a4c40316cc9dfe8016c767be4f541a6a01239d3c2b97402c1f327e984b02e405`. It re-elaborates the SDK-generated Heap, HeapAudit, RuntimeStore and RuntimeStoreTests modules, compiles those exact Lean modules to native code, and passes all twelve scenarios with the final foundation, including the MSB arena index and moved-root multiplicity preflight. The dependency refresh requires no store source or behavioral ABI change. Sparse-store key addressing remains LSB-first.

| Source/evidence | SHA-256 |
| --- | --- |
| RuntimeStore control and restoration | `833f2fb91be50ad892dbca9f60973300e9b2626dcd3e5650b6a7db8950004ee6` |
| RuntimeStoreTests | `680b88d23503c07a2b23dce78d67832356168b85f117128457b4a17273b42410` |
| Planted default-comparison defect | `ddf8ab47d3a8c51c037d726edbfe4628bdda5a98fca1420cf27b9ae11ef724f0` |
| Integrated Heap snapshot | `8f3880d42cc0294356bfbee1aa18e7e95fa4f9bfdcda63199523a366d304ae05` |
| Integrated HeapAudit snapshot | `9e777b30e67fc05df81d23fbd0674d06c65f606742370622ae9c5abbbcaa794e` |
