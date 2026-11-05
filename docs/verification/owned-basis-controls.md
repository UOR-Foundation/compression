# Owned basis controls: candidate/native diagnostic

This is a source-control/native diagnostic, not `just vv`, SDK acceptance,
production export, physical memory refinement, or release evidence.

`OwnedBasisContract` registers UC-WIRE-OWNED-BASIS-01 through -04 and names
`Uorc.Wire.OwnedBasisScenarios.run`. That owner ran red before implementation
(`implementationMissing`, process exit 1). The implementation subsequently
replaced the failing owner; no stub or disabling flag remains.

## Composition and ownership

`OwnedBasisSnapshot.begin` starts an empty `RuntimeTable.slots(basis)`.
`append` uses the Core reserved PutCursor and counts only its actual completed
result. The caller keeps its sequence owner through append acknowledgment or
cleanup. All empty objects occupy ordered slots. `seal` creates a fixed charged
metadata branch containing count word(tag 0) and table link(tag 1). The table's
direct root ticket moves into that link; it becomes a borrowed control.
This fixed shape is read through `OwnedBasisSnapshot.view`, not generic
`RuntimeMetadata.wordAt/linkAt` (which expect their 64-level trie).
`takeSnapshot` transfers the record ticket once and clears the builder owner.
Descriptor copies acquire no tickets and confer no program readiness.

The descriptor's provenance is the source-owned append/seal builder. `view`
checks its charged record shape, and hashing rejects inconsistent count/root
copies. It does not certify an arbitrary caller-fabricated heap as an append
history. The enclosing controller roots the record throughout hashing and
conditional admission, then retains the exact same table root as the separate
actual `Program.basis` ticket. Hashing never converts through Core.Basis,
GraphEmit, Graph, Trie, Rope, or a payload/length List.

`OwnedBasisHash` borrows that immutable snapshot and uses `OwnedRead` spans.
Its canonical input is shortest ULEB count, then every ordered shortest ULEB
length and exact object bytes. `OwnedHash(Purpose.basisObjects)` adds the
Profile-owned domain exactly once: legacy `UORC-BASIS-V1` plus NUL for format 2,
and Profile's format 3 domain. The caller selects the same Profile kind as the
archive. Each scalar octet reserves a charged one-byte image before constructing
and committing its bytes, and transfers one actual ticket into OwnedHash.
Object spans explicitly retain through `OwnedHash.borrowSpan` while the snapshot
keeps ownership. `takeDigest` transfers a retained digest once.

States contain fixed controls/handles and Core cursors; only Sessions carry the
single actual arena and usage. `roots(State)` inventories all owned table,
PutCursor, metadata, reservation/image, hash input/control/output, and digest
tickets. Lists returned by root inventories are temporary diagnostics, not
retained backing. Cleanup advances an already active Core release before
cancellation and accepts the actual completed table. Hash cancellation is
latched once, then progresses OwnedHash cleanup. Achieved arena/owners enter
State before post-operation accounting can refuse them.

Snapshot actions prepay 256 logical work; hash actions prepay 64. In the exact
OwnedHash source, `report.work = delta.work + overhead`: overhead is at most 33
(the 32-byte digest materialization plus one adapter event), while image commit
reports two work; the maximum one-call report is therefore 35. Other Heap
operations report one work. Scalar reservation/commit and one-byte materialization
fit within the same prepaid 64. OwnedHash reserves bounded profile control
storage before initialization (847 bytes for format 2, 593 for format 3).
These are logical source charges, not a complete native allocation/RSS bound.
Pauses preserve achieved state. U64 exhaustion is operational; no media, object,
or semantic wire-size cap is added.

## Validation

The full candidate linked-IR build and the reachable native closure build pass.
The existing diagnostic compiler and existing Lean 4.32.1 were used; neither
compiler was rebuilt. The runtime slice removes unreachable dependency
declarations, preserves every new module declaration, and records its selection
in `dependency-slice.json` in the scratch project.

The native source owner checks both Profile kinds against the independent pure
`Profile.hash` reference on these exact canonical inputs:

- Empty basis: `00`.
- Empty, `ab`, empty: `03 00 02 61 62 00`.
- One 128-byte object: `01 80 01`, then 128 octets `7f`.
- 128 empty objects: `80 01`, then 128 octets `00`.

Small fixture append/seal prefixes undergo cancellation/cleanup root audits and
zero-work pause checks. Hash ownership-phase changes undergo the same checks;
repeated cryptographic microsteps retain their phase ownership and receive pause
checks. The 128-object bulk fixture checks canonical output, work pauses, and the
final exact root inventory. Corrupt descriptor counts and duplicate snapshot and
digest transfers are rejected. The early compact empty/mixed controls pass
independently of the bulk cases.

A scratch-only source mutation omits zero-length object ULEB octets. It links and
compiles, but the same native owner exits 1 with `digest`, demonstrating that the
canonical reference gate detects the intended defect. The official source was
never mutated. The final official source owner is rerun after that mutation.

An earlier overbroad diagnostic that audited every cryptographic microstep was
manually stopped (exit 143); that was a stopped test process, not a failing source
assertion. It is excluded from passing evidence. Exact source hashes, build IDs,
exit codes and final logs are recorded in `owned-basis-controls-diagnostic.json`.
