<!-- uorc:non-authoritative -->
# Owned integrity binding checkpoint

The authority is `Uorc.Integrity.Profile`, `OwnedHash`, `ArchiveBinding` and
`BindingFixtures`; `CollisionEvidence` and `BindingControls` contain independent
proof/validation evidence. This checkpoint is based on integration
`d402e3d39d17ac614bdb4b6d98cc2ea3073edb91` for diagnostic ownership dependencies,
then rebased onto `3c8fca4c13ce0f50bb1b3b99c977bce2fa04d37f`. Its Heap, RuntimeValue,
RuntimeSequence and HeapAudit sources remain byte-identical; all integration Core
files are preserved.

`Profile.select(format, machine)` explicitly selects CHAIN-v1 for 2/1 and
STREAM-MAX-v1 for 3/1. Other pairs reject. Archive hashing adds no extra purpose
domain: it hashes the exact preimage prefix and excludes the trailing 32 bytes.
The six fixed `UORC`, format and machine octets are checked across arbitrary span
partitions. All other grammar belongs to the separate parser/emitter authority.
Format-2 and CHAIN-v1 meanings remain unchanged; no released compatibility is
implied by retaining those historical identities.

The production interface uses `Core.Heap.Arena` and
`Core.RuntimeSequence.Span`, not an arbitrary input `Bytes` value:

- `OwnedHash.start(arena, kind, purpose)` charges bounded control storage first.
- `beginSpan(arena, cursor, span)` consumes a nonempty producer span's one owned
  image root ticket. The producer removes that ticket only after successful
  admission. The hash cursor holds it through update completion/cancellation.
  Empty spans own and transfer no ticket.
- `borrowSpan` explicitly retains an additional ticket when the caller keeps its
  own. It is not the consuming producer-result interface.
- `step`, `finalize` and `cancel` return the next arena/cursor, Heap delta,
  operational work, boundary-materialization counts and typed outcome.
- `ownedRoots(cursor)` and `outcomeRoots(outcome)` expose the exact live ticket
  aggregate. A digest outcome transfers one owned image ticket; the terminal
  cursor retains none. Excess roots must never be normalized away.

At most one input octet is materialized per step under the control charge.
Finalization reserves the charged 32-byte output image before beginning final
component finalization. Intermediate caches stay within the control envelope.
The image is committed, control storage is released, then its handle transfers
once. These are conservative logical control/storage charges and operational
work, not native RSS or semantic admissibility measurements. Boundary copy
counts do not claim to measure every physical implementation copy.

`ArchiveBinding` supplies a checked runtime length fence, span admission,
finalization/cancellation and an independent 32-byte image comparison. Its U64
counts describe admitted native storage, not a wire-level input limit; wire Nat
and mathematical stream domains remain distinct. The caller retains the same
immutable archive snapshot through parsing, hashing and stored-digest checking.
Digest mismatch leaves its already-transferred output handle caller-owned.

Input bytes, counted basis objects and checkpoint bytes have distinct
profile-specific domains in `Profile.prefix`. The legacy basis domain remains
`UORC-BASIS-V1` + NUL; format 3 uses `UORC-BASIS-F3-M1-v1` + NUL. Checkpoint
query comparison still checks full target/basis bytes, boundaries, scope and
operational envelope. A new supported-profile guard rejects unknown or
cross-format contexts. Certificate/checkpoint scope must also bind the shared
canonical semantic cost profile and interpretation; Core/search own those
shared declarations and refinement. This lane adds no certification metadata to
ordinary archives and makes no semantic-budget minimum claim.

Observed diagnostics: 20 independent Python hashlib/OpenSSL framing vectors,
60 span partitions, ten raw byte reconstructions through actual owned sequence
slicing/indexing, a real producer-span ticket handoff, cancellation, range and
format/length rejection all pass generated-model execution. The source-only
binding probe reports six passing controls. Collision evidence reports ten
passing controls and rejects the planted false-digest-equality defect. These
are finite controls, not universal transition, reconstruction or cryptographic
reduction proofs. Full pinned-SDK and integrated BinaryProgram acceptance remain
separate; current diagnostic conversion uses the saved official compiler's
language-1.2 authoring APIs in a separate copy, leaving repository locks intact.

The factored owned adapter contains eleven source helpers and nine kernel-checked
record-refinement lemmas. Expanding these helpers recovers all twenty original
computational declarations and original theorem statements exactly (audit
annotations excepted). Two existing general boundary lemmas check empty-span
ownership and single-ticket digest publication. These do not establish a full
transition-preservation theorem.

The five source-owned negatives reject prefix corruption across partitions,
stored-digest mismatch, truncated input and excess preimage admission, and cancel
a reserved output safely. A planted untracked extra root makes three negative
owners fail; restored observations match byte-for-byte. Digest mismatch and
excess admission retain the ticket with the correct caller/producer owner.

Current certified diagnostic export is blocked by `LLS8002`: certificate A for
`BindingControls.chargedHashAndCleanup` is 5,107,159 bytes, exceeding the unchanged
4,194,304-byte limit. Factoring reduced the former 5,203,654-byte individual
certificate, but did not qualify the export. The saved compiler/lock identities,
source digests, exact finite coverage and remaining proof boundaries are recorded
in `validation/integrity/binding-evidence.json`. No full pinned release acceptance
or integrated BinaryProgram result is claimed by this checkpoint.

The validation client is `scripts/integrity-binding-runtime.mjs`. It verifies the
selected authority, compiles generated modules and executes all fixtures and
source-owned controls. The importer tests also reject changed digest bytes,
truncated archives and missing fixtures. The acceptance command remains the
locked `just vv`; diagnostic authoring conversion is not a replacement for it.
