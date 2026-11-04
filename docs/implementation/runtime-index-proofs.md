# General MSB index bridge

RuntimeIndexProofs is a theorem-only additive LexLean1.2 module. It proves the actual Heap.readIndex/writeIndex same-key laws for all Nat depths, arbitrary Index trees and UInt64 keys: one theorem for arbitrary inserted Entry and one for deletion. No valid-tree, shape, depth<=64 or arena premise is used. Arbitrary malformed intermediate trees are handled because writes replace the selected path and compact preserves left/right projections. The ten laws contain no executable helper definitions.

The two constructor families cover Option.some and Option.none exhaustively. A single forall Option theorem is not supplied: the pinned semantic proof surface supports cases on Nat/List/Bool/document inductives, but not builtin Option. This is a surface limitation, not a mathematical premise or impossibility in LexLean Core.

Official fmt/check/build and pinned Lean kernel pass. All10 traced axiom sets match source exactly:5 empty and5 {Quot.sound,propext}; no sorry or custom axiom. RuntimeValue and fused Heap generated Lean are byte-identical to the qualified final23 fixture, justifying reused imported oleans.

The isolated negative gate changes only the actual writeIndex highBit route test to lowBit. Heap compiles, but both arbitrary-depth families fail at successor. An independent source-owned witness proves the mutant loses any Entry at depth1/key1. Restoring Heap makes both general laws pass and rejects the loss witness. Restored sources and all3 generated modules match the positive fixture.

This does not establish other-key preservation,64-bit route injectivity, index enumeration, freshness, successful-operation preservation, or the full arena invariant. No runtime/native rerun or production export/release claim is made. The source-bound public receipt is runtime-index-proof-evidence.json; raw commands and compiler logs remain in the local diagnostic fixture.
