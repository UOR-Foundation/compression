<!-- uorc:non-authoritative -->

# Owned wire emission evidence

This checkpoint adds `Uorc.Wire.ProgramEmit`, `FrameEmit`, `Output`,
`OwnedRead`, `EmitContract`, and their typed runtime scenario owners. It is
based on integration `3c8fca4c13ce0f50bb1b3b99c977bce2fa04d37f`. The parser
checkpoint remains [draft PR #80](https://github.com/UOR-Foundation/compression/pull/80),
source `433fcdbabf583d94e0d70e0027dbb01e8a32c96c`. This work is related to
graph wire issue #12, resource issue #17, and integration draft #78. Their
historical profile-02 terminology does not imply legacy compatibility.

## Concrete API

`ProgramEmit.begin(arena, usage, program : RuntimeProgram.Program, mode,
embedded)` creates an emission session. The driving operations are
`advance(budget, session)`, `acknowledge(budget, session, ack)`,
`cancel(session)`, and `cleanup(budget, session)`. Each operation returns the
achieved session and a typed signal. `Output.Mode` is `measure | stream`;
measurement follows the same byte control without publishing output events.
`State.wireBytes` is checked exact wire length in either mode. `embedded`
suppresses the program's final output boundary when a frame owns that boundary.

`Output.Event` is `byte(UInt8) | borrowedSpan(Handle, UInt64, UInt64) | end`.
There is at most one pending output. `Ack.consumed` means the sink has finished
its synchronous read or prepaid copy while the source owner remains live.
`Ack.takeSpan` retains an image ticket before returning `Signal.transferred`.
The receiver must inventory that ticket until it transfers or releases it.
Taking a scalar event as a span is a retained protocol failure.

The emitter reads the committed `RuntimeProgram.blockAt`, `instructionAt`,
`typeAt`, and `captureAt` interfaces. One borrowed block and instruction hold
fixed scalar/handle metadata. The 39 explicit wire mappings are independent of
`RuntimeOpcode.code`. Scan emits its observer scalars and first capture lane
before its update scalar and second lane. Immediate byte/predicate domains are
checked. Naturals use minimal ULEB, including the full operational U64 range.
Static admission and execution remain Core responsibilities.

`OwnedRead` walks charged immutable sequence nodes using a fixed cursor. It
allocates no graph, rope, byte buffer, stack, table cursor or signature cache.
Its explicitly named `Step.borrowedSpan` borrows from the enclosing live owner.
It does **not** call `RuntimeSequence.beginSpan/step`. A nonempty Core
`RuntimeSequence.Outcome.span` transfers an owned image ticket independently of
its terminal cursor; callers of that separate API must inventory the returned
ticket. Empty terminal cursor roots cannot erase that ownership fact.

## Frame preimage and integrity seam

`FrameEmit.begin(arena, usage, request, mode)` uses the same advance,
acknowledgment, cancellation and cleanup protocol. `FrameEmit.root(state)` is
the single wire owner to inventory in `CallerRoots.wire`;
`pendingEvent(state)` exposes either the frame output or its child output.
`Request` contains fixed binding data, `FrameEventContract.OwnedLengths`, scope,
raw/program body metadata, and a charged record handle.

The record uses `FrameEmit.frameMetadataKind = 1464422982` and three metadata
links: key 0 to the lengths table root, key 1 to the raw sequence root or program
block-index root, and key 2 to the conditional basis image or null. The emitter
checks those links before exposing output. This makes lifetime depend on real
charged edges rather than an asserted relationship between unrelated handles.
The caller keeps its record owner live until the first attach action retains
the emitter's `ownerRoot(wire, record)`.

The lengths table has metadata kind
`FrameEventContract.lengthsMetadataKind = 1464422981`. Its indexed leaf is
`metadataLink(kind, index, record)`, and that record's word at key 0 is the
length. Every ordered length, including every zero, is read by index and
emitted. Count, aggregate sum, body size, offsets and wire length use checked
U64 operational counters. Exhaustion is an operational refusal and does not
narrow the mathematical language of finite archives.

The integrity owner supplies `Binding` (four magic bytes, format, profile and
basis-identity width). This is data, not an accepted-format token. The emitter
outputs magic, format, profile, scope, representation, object count, ordered
lengths, body length, optional basis identity and exact body. `Output.end`
marks the **preimage** boundary. The integrity controller uses `wireBytes` and
composes its separately versioned trailer. Format 2 meanings are preserved;
new format 3 acceptance must come from the integrity owner's binding and tests.
No checksum algorithm or identity is selected here.

## Ownership and accounting

Program attachment retains a charged wire owner over the block index. Frame
attachment retains a charged owner over its frame record. Borrowed metadata and
pending spans stay under that owner until acknowledgment. Cancellation and
typed failures preserve the owner; cleanup releases it and drains the achieved
Heap release cursor one prepaid action at a time. Transferred span tickets are
separate external owners and survive emitter cleanup.

Producer controls contain no `Arena`. A driving session contains one Arena and
Usage. Frame composition stores only a child `ProgramEmit.State`; each child
operation receives the actual arena and usage and immediately returns their
achieved values. It never stores a second child session or predecessor arena.
The exact Core admission detach/reattach API is not yet present in the tested
integration source; this checkpoint does not invent it or manufacture a ready
token. Actual admission/controller composition remains an integration obligation.

Actions prepay bounded logical work; stream events reserve output before
exposure. Work/output pauses preserve cursor, achieved usage, pending output,
arena and cleanup roots. Arena observations record current and peak
`uorc.logical-owned-bytes/1` usage after successful and failed actions. These
logical ownership moves do not establish physical copy elision, linear runtime
storage, allocator behavior, stack usage or process RSS.

## Executed diagnostic evidence

The existing diagnostic compiler is LexLean PR #393 at
`3fe13b50e8d3b20e7e24d7fc8cd6b7a232c1471e`, executable SHA-256
`c479a5cf91b39cfef9806a8e4c6165450b2b7ac89e275afaf3a68c730f9edd78`,
with Lean 4.32.1. SDK `to_notation` authored the new language-1.2 source; no
handwritten parser, emitter or compiler substitutes for these declarations.

The adjacent JSON binds the exact source/declaration hashes, dependency slice,
compiler identities, build receipts, native owners and mutation outputs.
Inherited theorem declarations were omitted from this computational diagnostic
projection, old dependencies were converted with the SDK, and uncalled
dependency declarations were excluded. Every declaration in all seven new
source modules was retained unchanged. This projection does not establish
general refinement, theorem verification, production export or SDK acceptance.

The executed source owners cover:

- Measure and stream for all 39 opcodes, using compact authoritative vectors,
  with actual independent GraphParse-to-GraphEmit diagnostic checks.
- Empty literal, full U64 immediate, invalid byte/predicate after earlier
  acknowledged output, wrong acknowledgment, work/output pause and resume.
- Separate program, wire and returned image-ticket inventories; cancellation
  with a pending literal and retained cleanup ownership.
- Raw/frame preimages for empty collections, empty objects, ordered empty
  boundaries, multi-byte length 129, natural/literal program bodies and a
  conditional basis span.
- Frame sum mismatch, mismatched ownership link, missing record/length,
  invalid basis width/span/handle and a child emission failure.
- Frame cancellation with a pending program span, explicit ticket transfer,
  emitter cleanup while that ticket remains live, then its explicit release.

The planted defect changes only naturalConstant's wire tag from 0 to 1 in
`ProgramEmit.wireTag`. It must build and compile before
`ProgramEmitScenarios.runOpcodes` rejects it with typed `mismatch` and exit 1.
Exact restored source must run both complete owner groups with exit 0. The
receipt records the final observed red/green outcomes.

`scripts/wire-emitter-validation.mjs` is a fail-closed targeted orchestrator for
the locked SDK, not a substitute acceptance gate. Syntax checking and its exact
mutation locator were exercised. Its full invocation here refused with
`UC-WIRE-EMIT: the digest-selected SDK is required`.

## Remaining qualification

The exact locked SDK, `just model-write`, the SDK targeted gate, actual Rust
emitter export/certificates/replay, general wire refinement, Core admission and
search/controller integration, production frame parsing/object demultiplexing,
format-3 integrity composition and complete `just vv` remain unqualified. No
source lock, SDK pin, universal policy file or contributor implementation was
changed. No merge or release acceptance is claimed.

The separate narrow Heap consumer export result remains at
[the public receipt](https://github.com/UOR-Foundation/compression/blob/9e0b052064a18f84e6d79c47f0d9947c742c8500/docs/verification/heap-export-consumer-diagnostic.json).
It passed unchanged certificates A/B/E and artifact replay with the same
existing diagnostic compiler. That component result does not qualify the whole
SDK or this emitter.
