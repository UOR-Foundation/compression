# Handle arena and runtime ownership

This local lane adds the runtime carrier and ownership substrate. The authoritative behavior is in `src/Uorc/Core/RuntimeValue.lex.tex`, `Heap.lex.tex`, and `RuntimeTable.lex.tex`. Mathematical `Types.Value`, `Store.Trie`, `Bytes.Rope`, graph syntax and independent reference semantics remain separate. The carrier contains no nested copy of those values.

## Integration API

`RuntimeValue.Handle` contains one `UInt64` ID. ID zero denotes an empty sequence, empty sparse store or empty table child; it is never allocated. IDs increase monotonically, including an explicit exhausted state after allocating the maximum ID. Equal payloads never imply equal IDs. Handles are arena-local: admission, basis, execution and retained checkpoints must share one arena context or explicitly re-admit their data. Handles from independent arenas must never be compared or mixed.

`RuntimeValue` has exactly five alternatives: `natural(UInt64)`, `byte(UInt8)`, `predicate(Bool)`, `sequence(Handle)` and `store(UInt8, Handle)`. `RuntimeType` and `typeOf` provide their five tags. The machine must map these tags to its checked mathematical value types at admission.

The Heap API is pure, and all failures use `HeapError`:

- `emptyArena(maxOwnedBytes) : Arena`
- `allocate(arena, node) : Result Allocation HeapError`
- `retain(arena, handle) : Result Transition HeapError`
- `release(arena, handle) : Result Transition HeapError`
- `releaseStep(arena) : Result Transition HeapError`
- `dropRetainedRoot(arena, handle) : Result Transition HeapError`
- `allocateMoved(arena, node, moveFirst, moveSecond) : Result Allocation HeapError`
- `lookup(arena, handle) : Result Entry HeapError`
- `sequenceInfo(arena, handle) : Result SequenceInfo HeapError`
- `totalOwnedBytes(arena) : Result UInt64 HeapError`
- `reserveImage(arena, kind, length) : Result Allocation HeapError`
- `commitImage(arena, reservation, bytes) : Result Allocation HeapError`

`Allocation` contains `arena`, `handle`, and `delta`. `Transition` contains `arena`, `delta`, and `complete`. The returned arena stores the release cursor; the API never accepts a detached, potentially stale cursor. Only the returned state should replace the current state. A failed operation returns no replacement state. Errors distinguish malformed handles, invalid nodes, count overflow/underflow, handle exhaustion, owned-byte overflow, budget exhaustion, busy/missing release, invalid roots and protocol failures.

`Delta` separately reports allocated/released bytes and entries, retained/dropped edges, retained/dropped roots, and logical ownership-operation work. It does not report CPU instructions, index traversal time or allocator operations.

Allocation creates one owned external root and retains each nonzero child edge, including a repeated child twice. `allocateMoved` additionally consumes the specified existing temporary root tickets after retaining the new parent edges. This keeps a newly built persistent path from leaving temporary roots behind. `dropRetainedRoot` is the bounded nonreclaiming root drop; its last-reference case is a typed protocol failure and must use resumable release.

Release transfers an existing root ticket into the cursor. Its first token has no additional dynamic charge, so release can begin at the exact budget limit. Each later pending edge token costs 16 modeled bytes. A release step handles exactly one reference: decrement, or remove the zero-count entry before scheduling its children. Allocation and retain are refused while release is active. Run one step per machine continuation event; do not hide a drain loop inside dispatch.

## Actual owners and indexed metadata

Arena nodes contain handles, never recursive mathematical ropes/tries. Image entries have an explicit `input`, `archive`, `basis` or `runtime` kind and retain the actual immutable `Bytes`. Sequence leaves store image ID, offset and length. Leaf bounds are checked without a wrapping offset addition. Sequence branches store length/height caches and child IDs; allocation validates both caches. Store leaves and branches retain child IDs. The mathematical store-shape/depth discipline belongs to the store operation/validation layer.

`slot(role, value)`, `rootBranch(role, left, right)` and `ownerRoot(role, target)` provide actual roots for frames, continuations, basis, outputs, checkpoints, input/archive owners, checked graphs, type/body/lifetime caches, admission, search, certificates, verification and wire state.

`metadataWord(kind, tag, word)`, `metadataLink(kind, tag, target)` and `metadataBranch(kind, left, right)` store indexed graph and cache records. They contain only fixed-width fields and owned handle edges. A metadata link may retain a literal image or another metadata record. Retaining an old reference AST beside these nodes would retain a second owner and is outside this representation.

`RuntimeTable.Table {kind, root}` is a common fixed64 index. Its kind is `slots(role)` or `metadata(kindId)`. `emptyTable`, `lookup(arena, table, UInt64)` and `put(arena, table, UInt64, leaf)` are the public operations. Slot leaves are `Node.slot`; metadata leaves are `Node.metadataLink` whose tag equals the index key. Lookup follows at most 64 arena child IDs. Put allocates at most 65 entries, retains unchanged sibling IDs and moves newly allocated subtree roots into parents. The caller retains the old table root until its last use and releases it explicitly. No value-equality search infers aliasing or frees an owner.

The graph, validation and machine workers must assign metadata kind/tag schemas, build these tables during bounded admission, discard the old AST/cache backing at the conversion boundary, and attach every simultaneously live owner to its partition/root. The arena is not a new instruction interpreter or lifetime-plan validator. Its metadata nodes provide the storage for those workers' indexed runtime carriers.

A block builder must call `reserveImage` before allocating/merging its new byte backing. `Node.imageReservation(kind, length)` charges `48 + length` immediately and contains no old payload. Its ticket is a unique external root; the API refuses retain and parent edges to reservation nodes. The pending continuation owns that root ticket. `commitImage` checks the ticket's uniqueness and exact byte length, removes the reservation, and allocates a fresh immutable image ID in one pure transition with the same byte charge. The old inputs stay live until explicitly released after the new builder owner is installed. Cancellation releases the reservation. An append operation must not be started first and charged afterward. This protocol does not assume `Bytes.append` mutates, uniquely owns or reuses either input buffer.

## Invariant and independent checks

For states created through the API from an empty arena:

1. Each nonzero child is a live, strictly older handle. Allocation checks this ordering and child existence. Retain/release never change an existing node. This establishes acyclicity; it is a required condition for reference-count reclamation.
2. Each entry's incoming count is its external root count plus all live parent-edge occurrences plus pending edge tokens targeting it. The initial pending root is already one of the external roots and is not counted twice.
3. Owned entry bytes are exactly the sum of immutable live entry weights. Dynamic charged ownership is those bytes plus 16 bytes per pending edge token. A pending root ticket is transferred, not newly created. A release never needs extra quota to start; removing a parent funds its bounded child scheduling.
4. A zero entry is removed before child tokens are scheduled. A retained alias or capture keeps the entry live until every corresponding reference is dropped.
5. Failure returns no replacement arena. Temporary persistent states constructed before a later failed check are not committed to the logical state.

`HeapAudit` independently inventories the actual fixed64 index, reconstructs parent-edge multiplicity, pending-token counts, acyclic ordering, entry totals and weights, and checks the dynamic budget. It also provides a copying mathematical sequence interpretation used against literal binary expected values. `HeapTests` checks every release transition in its alias/image sequence. `RuntimeTableTests` independently checks persistent table ownership and metadata owner conversion at snapshots, including key zero and maximum `UInt64`, then final reclamation. Equal-value allocations and independent equal images remain separately charged. Exact-budget reclamation and reservation commitment are checked without requiring spare payload quota. An unchanged-value table write remains an explicitly owned persistent path.

This is executable invariant evidence and scoped kernel evidence. It is not a completed general refinement theorem relating every runtime value/store/graph to the other workers' independent mathematical reference evaluator. That bridge must be composed during integration; mathematical reference modules must remain outside the production entry closure.

## Accounting and generated-code boundary

The source defines deterministic logical entry weights: image `48 + payload length`, sequence leaf `64`, sequence branch `72`, store leaf `40`, store branch `48`, slot `64`, root branch `56`, metadata word/link/branch `56`, owner root `48`, and uncommitted image reservation `48 + promised length`. These coefficients are a declared logical accounting model, not a Rust layout or RSS measurement. Image payload bytes are charged even when a literal borrows only a slice. A repeated live edge is a reference occurrence, not a second payload owner.

The entry index itself is a fixed-depth persistent index. Its generated representation, reference-count control blocks, allocator alignment, compiler temporaries, fixed arena/machine control records and externally retained prior arenas require separate physical accounting. A transition may borrow the previous arena while constructing its replacement. Source-level removal proves logical absence from the replacement, not immediate physical deallocation of every older borrowed/shared representation. Repeated retention of history is not silently excluded from any physical-memory claim; there is no such claim here.

## Verification and release scope

The new modules use official language 1.2 authoring notation with explicit executable closure markings and exact axiom sets. Runtime helpers use the traced `Quot.sound`/`propext` instances where required; carrier-only definitions have exact empty sets. The base project is still locked to language 1.1 and the older SDK. Main-project migration and locked SDK acceptance remain integration requirements. Neither the product locks nor template policy files were changed in this lane.

`tests/heap/prepare.mjs` copies the six exact source files into a fresh isolated diagnostic fixture and records their SHA-256 hashes. This permits official compiler diagnostics without changing the product locks. It is not an alternative release gate. The native test harness consumes generated implementation crates, copies them byte-exactly, adds only Boolean assertions in test files, and leaves the original bundle unchanged.

Use the commands and retained receipts in `docs/implementation/runtime-heap-evidence.md`. Full release acceptance remains the repository's locked-SDK `just vv`, including source-model/registry integration and the composed product closure.
