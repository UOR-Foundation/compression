# Authored runtime integration interfaces

Generated from the source using official `lexlean::sdk::read_module`. Source remains authoritative. Cursors contain fixed scalar/handle fields; variable continuations are charged arena owners. The builder's final cancellation correction passes all 35 source-owned scenarios in the native Lean diagnostic; no complete product qualification is claimed.

## RuntimeSequence

```text
Span { image: Uorc.Core.RuntimeValue::Handle, offset: uint64, length: uint64 }
Outcome = pending | sequence(Uorc.Core.RuntimeValue::Handle) | byte(uint8) | span(::Span) | cancelled
SequenceTransition { arena: Uorc.Core.Heap::Arena, cursor: ::Cursor, delta: Uorc.Core.Heap::Delta, work: uint64, outcome: ::Outcome }
beginConcat(arena: Uorc.Core.Heap::Arena, left: Uorc.Core.RuntimeValue::Handle, right: Uorc.Core.RuntimeValue::Handle) -> Result<::SequenceTransition, ::SequenceFailure>
length(arena: Uorc.Core.Heap::Arena, root: Uorc.Core.RuntimeValue::Handle) -> Result<uint64, ::SequenceFailure>
beginSlice(arena: Uorc.Core.Heap::Arena, root: Uorc.Core.RuntimeValue::Handle, start: uint64, count: uint64) -> Result<::SequenceTransition, ::SequenceFailure>
beginIndex(arena: Uorc.Core.Heap::Arena, root: Uorc.Core.RuntimeValue::Handle, start: uint64) -> Result<::SequenceTransition, ::SequenceFailure>
beginSpan(arena: Uorc.Core.Heap::Arena, root: Uorc.Core.RuntimeValue::Handle, start: uint64, count: uint64) -> Result<::SequenceTransition, ::SequenceFailure>
resultOwned(cursor: ::Cursor) -> bool
emptyCursor() -> ::Cursor
step(arena: Uorc.Core.Heap::Arena, cursor: ::Cursor) -> Result<::SequenceTransition, ::SequenceFailure>
cancel(arena: Uorc.Core.Heap::Arena, cursor: ::Cursor) -> Result<::SequenceTransition, ::SequenceFailure>
ownedRoots(cursor: ::Cursor) -> list<Uorc.Core.RuntimeValue::Handle>
```

Sequence cancellation preserves the current arena/cursor lineage. Drive `step` until `cancelled`; cancellation covers a fresh allocation prefix, retained result and the operation's already-transferred release ticket. The cursor layout is unchanged. Serialize the arena's single release cursor: phases 6/11 own the release transferred by that exact sequence continuation. Do not independently advance or substitute it while retaining the continuation. A completed sequence/span ticket has already moved to the caller; cancelling the terminal empty cursor is an `invalidCursor` failure.

## RuntimeStore

```text
Outcome = pending | byte(uint8) | store(uint8, Uorc.Core.RuntimeValue::Handle) | cancelled
StoreTransition { arena: Uorc.Core.Heap::Arena, cursor: ::Cursor, delta: Uorc.Core.Heap::Delta, work: uint64, outcome: ::Outcome }
begin(arena: Uorc.Core.Heap::Arena, defaultByte: uint8, root: Uorc.Core.RuntimeValue::Handle, key: uint64, operation: ::Operation) -> Result<::StoreTransition, ::StoreFailure>
beginRead(arena: Uorc.Core.Heap::Arena, defaultByte: uint8, root: Uorc.Core.RuntimeValue::Handle, key: uint64) -> Result<::StoreTransition, ::StoreFailure>
beginWrite(arena: Uorc.Core.Heap::Arena, defaultByte: uint8, root: Uorc.Core.RuntimeValue::Handle, key: uint64, newByte: uint8) -> Result<::StoreTransition, ::StoreFailure>
step(arena: Uorc.Core.Heap::Arena, cursor: ::Cursor) -> Result<::StoreTransition, ::StoreFailure>
cancel(arena: Uorc.Core.Heap::Arena, cursor: ::Cursor) -> Result<::StoreTransition, ::StoreFailure>
ownedRoots(cursor: ::Cursor) -> list<Uorc.Core.RuntimeValue::Handle>
```

## RuntimeByteBuilder

```text
Outcome = pending | builder(::Builder) | sequence(Uorc.Core.RuntimeValue::Handle) | cancelled
BuilderTransition { arena: Uorc.Core.Heap::Arena, cursor: ::Cursor, delta: Uorc.Core.Heap::Delta, work: uint64, copied: uint64, outcome: ::Outcome }
emptyCursor() -> ::Cursor
begin(arena: Uorc.Core.Heap::Arena) -> Result<::BuilderTransition, ::BuilderFailure>
beginAppendOwned(arena: Uorc.Core.Heap::Arena, builder: ::Builder, byteValue: uint8) -> Result<::BuilderTransition, ::BuilderFailure>
beginFinishOwned(arena: Uorc.Core.Heap::Arena, builder: ::Builder) -> Result<::BuilderTransition, ::BuilderFailure>
cancel(arena: Uorc.Core.Heap::Arena, cursor: ::Cursor) -> Result<::BuilderTransition, ::BuilderFailure>
step(arena: Uorc.Core.Heap::Arena, cursor: ::Cursor) -> Result<::BuilderTransition, ::BuilderFailure>
builderRoots(builder: ::Builder, roots: list<Uorc.Core.RuntimeValue::Handle>) -> list<Uorc.Core.RuntimeValue::Handle>
ownedRoots(cursor: ::Cursor) -> list<Uorc.Core.RuntimeValue::Handle>
```

## RuntimeTable

```text
Outcome = pending | complete(::Table) | cancelled | failed(Uorc.Core.Heap::HeapError)
PutTransition { arena: Uorc.Core.Heap::Arena, cursor: option<::PutCursor>, outcome: ::PutOutcome, delta: Uorc.Core.Heap::Delta, peakOwnedBytes: uint64 }
ownedRoots(s: ::PutTransition) -> list<Uorc.Core.RuntimeValue::Handle>
step(s: ::PutTransition) -> ::PutTransition
cancel(s: ::PutTransition) -> ::PutTransition
beginPut(arena: Uorc.Core.Heap::Arena, table: ::Table, key: uint64, leaf: ::PutLeaf) -> Result<::PutTransition, Uorc.Core.Heap::HeapError>
beginErase(arena: Uorc.Core.Heap::Arena, table: ::Table, key: uint64) -> Result<::PutTransition, Uorc.Core.Heap::HeapError>
```


## Foundation ownership helpers

`Heap.allocateControl(arena, bytes)` allocates a retainable fixed-record capacity owner with modeled charge `48 + bytes`; it is not an image and cannot be committed into one. No padding Bytes are created. `preflightImageCommit(arena, reservation, expectedLength)` returns the charged weight after validating every typed commit precondition, before bytes are copied. `withdrawRootRelease(arena)` transfers the still-unprocessed single cursor-root ticket back to its caller; it is rejected after a release step. These helpers return checked `Result` values and do not establish physical RSS equivalence.

The final composed snapshot passes canonical formatting, official source check/build, kernel compilation of all fourteen modules, all 406 exact definition/theorem axiom policies and all seven suite predicates in the native Lean diagnostic. This includes sequence cancellation, the twelve store scenarios, seven table-cursor scenarios and all 35 builder scenarios. The superseded builder interpreter was stopped only after its native pass. Actual SDK lowering succeeds for all 21 production roots; the nineteen runtime closures exclude mathematical backing, HeapAudit and atomic table writes. See `runtime-composed-evidence.json` and `runtime-production-closure.json` for source hashes and scoped results. Known Rust preservation qualification remains owned by LexLean #310/#393. The 30-line reproducer and prior failure receipts are preserved. No release or issue-completion claim is made.
