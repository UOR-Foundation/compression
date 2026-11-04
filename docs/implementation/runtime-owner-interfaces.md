# Authored runtime integration interfaces

Generated from the current source using official `lexlean::sdk::read_module`. Source remains authoritative. Cursors contain fixed scalar/handle fields; variable continuations are charged arena owners. The builder runtime suite remains in progress; no complete product qualification is claimed.

## RuntimeSequence

```text
Span { image: Uorc.Core.RuntimeValue::Handle, offset: uint64, length: uint64 }
Outcome = pending | sequence(Uorc.Core.RuntimeValue::Handle) | byte(uint8) | span(::Span)
SequenceTransition { arena: Uorc.Core.Heap::Arena, cursor: ::Cursor, delta: Uorc.Core.Heap::Delta, work: uint64, outcome: ::Outcome }
beginConcat(arena: Uorc.Core.Heap::Arena, left: Uorc.Core.RuntimeValue::Handle, right: Uorc.Core.RuntimeValue::Handle) -> Result<::SequenceTransition, ::SequenceFailure>
length(arena: Uorc.Core.Heap::Arena, root: Uorc.Core.RuntimeValue::Handle) -> Result<uint64, ::SequenceFailure>
beginSlice(arena: Uorc.Core.Heap::Arena, root: Uorc.Core.RuntimeValue::Handle, start: uint64, count: uint64) -> Result<::SequenceTransition, ::SequenceFailure>
beginIndex(arena: Uorc.Core.Heap::Arena, root: Uorc.Core.RuntimeValue::Handle, start: uint64) -> Result<::SequenceTransition, ::SequenceFailure>
beginSpan(arena: Uorc.Core.Heap::Arena, root: Uorc.Core.RuntimeValue::Handle, start: uint64, count: uint64) -> Result<::SequenceTransition, ::SequenceFailure>
resultOwned(cursor: ::Cursor) -> bool
emptyCursor() -> ::Cursor
step(arena: Uorc.Core.Heap::Arena, cursor: ::Cursor) -> Result<::SequenceTransition, ::SequenceFailure>
ownedRoots(cursor: ::Cursor) -> list<Uorc.Core.RuntimeValue::Handle>
```

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

Status checkpoint: sequence nine-scenario, store twelve-scenario and table-cursor seven-scenario official Lean runtime checks pass on their recorded fixtures. Builder source/kernel/exact-axiom checks pass; its 32-case runtime command is still running, and cancellation at initial final-control release is being corrected through `withdrawRootRelease`. Known Rust preservation qualification remains owned by LexLean #310/#393. The 30-line reproducer and prior failure receipts are preserved. No release or issue-completion claim is made.
