# Streaming program parser evidence

The implementation branch starts at integration commit
`86e9b2a2157dceaf3c26539b87b025b2b3789e36` (draft #78, stacked on #75).
The new authoritative modules are `Uorc.Wire.ProgramParse`,
`ProgramParseContract`, `ProgramParseScenarios`, and `FrameEventContract`.
No contributor branch, SDK pin, checksum construction, or universal policy file
is changed by this lane.

## Concrete integration API

`ProgramParse.begin(arena, usage)` creates a session. The driving roots are
`feed(budget, session, image : Handle, offset : UInt64, length : UInt64)`,
`advance(budget, session)`, `finishInput(budget, session)`,
`acknowledge(budget, session, Ack)`, `cancel(session)`, and
`cleanup(budget, session)`. Each driving result retains a `Session` and a
`Signal`. A work pause returns the exact session; the source checks U64 work
exhaustion separately from a resumable work allowance pause.

The fixed state holds counters, a checked absolute program position, a borrowed
image span, one pending `RuntimeProgram.Event`, and one charged wire owner.
`roots(session).wire` fits the existing `CallerRoots.wire : Handle` slot.
`State.image` and `State.pendingImage` are borrowed handles under that owner;
they are not extra external root tickets. The arena release cursor owns any
release already scheduled after the state clears its wire handle.

A pending literal remains under the wire owner until `Ack.takeLiteral` retains
an external image root and returns `Signal.transferred(handle)`. The admission
owner receives that ticket and the pending event together, then commits it to
charged runtime metadata or cleans it up. `Ack.consumed` on a literal is a
retained protocol failure. Feed allocates a charged wire owner edge before the
upstream span owner may release its ticket. Cancellation latches immediately;
cleanup remains prepaid and resumable. Both successful and failed actions
observe the resulting arena for current and peak logical owned bytes.

The event is the committed closed `RuntimeProgram.Event` algebra. Wire tags
0..38 are explicit and separate from `RuntimeOpcode.code`. Scan's observer
scalar/capture lane precedes its update scalar/capture lane. The seal event is
held until exact program EOF; it is syntax evidence, not an admission token.
Static validation, runtime metadata writes, execution, and publication are
owned by the other integration lanes.

There is no retained Graph, Rope, Bytes, List, Trie, or signature cache beside
the arena in the parser state. No literal payload copy occurs in the parser
source. Logical ticket accounting does not establish physical copy elision,
allocator behavior, stack measurements, or RSS of a generated backend.

## Frame and object boundary contract

`FrameEventContract` supplies typed archive and object events and an
`OwnedLengths` record containing a charged `RuntimeTable.Table`, count, and
aggregate output length. It specifies an acknowledged indexed length write for
every ordered object, including zero-length objects, before the next length is
accepted. The object cursor is fixed scalar state over that table. It introduces
no second table cursor or uncharged length list.

This module is a contract, not a production frame parser. Its source constants
are `diagnosticFormat = 2`, `streamingFormat = 3`, and `machineProfile = 1`.
Dispatch labels format 2 as the diagnostic reference and format 3 as awaiting
the separately reviewed integrity binding. No new format acceptance, checksum
identity, or archive-v1/profile02 compatibility is claimed. The enclosing frame
driver supplies program EOF exactly at body length, preserving integrity bytes
outside the program stream.

## Executed diagnostics

The host diagnostic compiler was the existing LexLean PR #393 commit
`3fe13b50e8d3b20e7e24d7fc8cd6b7a232c1471e`, with Lean 4.32.1.
The production lock was not changed. SDK `to_notation` authored the new files;
no handwritten compiler, runtime parser, or Lean semantics was substituted.

The exact new declaration payloads were retained in a diagnostic computational
dependency slice. Inherited theorem declarations were omitted, old data-form
dependencies were converted with the SDK to language-1.2 notation, and uncalled
dependency declarations were excluded. These transformations are diagnostic
limitations, not acceptance or general refinement evidence. The source hashes,
unchanged declaration hashes, build IDs, selected dependency inventory, and
runtime outputs are in the adjacent JSON evidence files.

Native Lean execution passed all 12 typed source scenario owners:

- 42 compact fixtures, covering all 39 wire opcodes, every one of their 527 split positions, empty chunks, empty literals, empty blocks, and multi-byte ULEBs.
- Both source literal carriers emitted by GraphEmit, with another 24 split positions.
- Normalized event comparison against the independent existing GraphParse result on all compact fixtures. GraphParse is imported only by the diagnostic scenario module.
- Nonminimal, overflow, truncation, invalid type/opcode/predicate, and trailing-byte failures, including failures after earlier acknowledged events.
- Cancellation while a literal is pending, retained cleanup ownership, cancellation followed by a cleanup work pause, wrong acknowledgement/feed order, duplicate EOF, and budget resume.
- Invalid spans/handles, arena budget mismatch, and U64 work exhaustion.

The syntax-only all-opcode fixtures are not claims that those programs pass
static admission. The empty literal and empty parameterized block use valid
local return distances. Empty graph syntax is tested separately from entry
admissibility.

The planted defect replaced the admission-ticket retain in
`ProgramParse.acknowledgePaid` with a null-handle retain. It type checked,
generated Lean, and compiled. `runTransferOwnership` then returned the intended
typed `ownership` failure with process exit 1. After exact restoration, all 12
owners passed again with exit 0. This did not rely on a compile failure or a
large ground `rfl` proof.

`node --check scripts/program-parser-validation.mjs`, its single-retain mutation
locator, and diagnostic canonical formatting checks passed. The committed
orchestrator has no diagnostic fallback: it checks the selected SDK, verifies
the exact source, executes the typed owner, checks the intended mutation failure,
and rebuilds/runs the restored control. Its full invocation here failed closed
with `UC-WIRE-PARSE: the digest-selected SDK is required`.

## Remaining qualification and blockers

1. Exact locked-SDK `just model-write`, targeted runtime gate, production export,
   certificate checks, and `just vv` remain pending. The locked SDK was not run. No acceptance result was manufactured from
   the diagnostic compiler.
2. At the recorded parser integration base, exact inherited-source checking
   with the diagnostic compiler failed before this parser at `Core.Resources`, `LLT4001`: `proof.definitions` is outside the
   closed JSON schema. The source is unchanged in this lane.
3. Unpruned native Lean compilation also failed in existing `Core.Bytes.valid`
   and `sliceUnchecked`: generated recursive equations refer to the unbound
   scrutinee `rope`. The compact reproduction is a structurally recursive match
   that calls another function on the original scrutinee inside a branch;
   the generated equation omits that scrutinee binder. The adjacent
   `recursive-scrutinee-repro.lex.tex` reproduced it: source build succeeds,
   then generated Lean fails with `Unknown identifier tree`. This is distinct from
   the known certificate-B #310 regression; no compiler fix or relaxed
   certificate check is included here.
4. General wire-to-event refinement, admission and execution integration,
   production frame/object demultiplexing, format-3 integrity dispatch, physical
   backend measurements, and inclusion of this targeted gate in complete
   product acceptance remain open obligations.

The parser checkpoint was published after fresh direct user approval: commit
`433fcdbabf583d94e0d70e0027dbb01e8a32c96c`, draft PR #80. Two earlier automatic
approval-review rejections did not execute. No alternative upload route was
used; the accepted action followed new direct approval. No merge or release
operation was performed.

The exact-SDK gate remains unrun. The observed diagnostic checks and failures
have known outcomes; they do not set release acceptance.
