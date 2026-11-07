<!-- uorc:non-authoritative -->
# Closed archive caller custody: candidate/native controls

`Uorc.Wire.ClosedArchiveCallerScenarios` is the source owner for
UC-WIRE-CLOSED-CALLER-01 through -07. The basic owner ran red before its
implementation; the extended owner also ran red after registering the additional
scenarios and before implementing their assertions. No failing stub remains.

The isolated diagnostic uses the existing candidate compiler and Lean 4.32.1.
Neither compiler is rebuilt. This is source-control/native diagnostic evidence,
not `just vv`, locked SDK acceptance, production export, or a native physical
allocation/RSS refinement proof.

## Actual custody and reads

Each fixture starts with the valid raw/program archive from
`ClosedArchiveRunScenarios.wire`, charged through
`ArchiveSnapshotScenarios.setupBytes`. The separate original sequence is then
allocated into that exact same arena and supplied as protected
`CallerRoots.input`. The archive ticket and that arena move into the actual
`ClosedArchive.beginWithCaller` operation.

The shared-candidate cases call the actual `Heap.retain` on the archive sequence.
Its same handle is passed both as the transferred archive candidate and as
protected `CallerRoots.search`, with two actual root tickets. This is role
multiplicity, not deduplication or two descriptor copies without a retain.
A separate input original remains protected throughout.

Output progress carries the actual `ClosedArchive.State`, fixed scalar counters,
and borrowed `OwnedRead` controls. Original traversal uses `readSequenceStep`;
every original and output byte uses `readByte`. Each updated State is threaded
into the next operation before acknowledging the actual pending output span.
No read API returns an independent arena snapshot. Tests check ordered begin/end
boundaries, including raw empty objects, exact bytes and original EOF.

After actual cleanup, `takeClean` must return a State whose owner is
`handedOff(Usage)` and the actual `CleanSession`. Its caller inventory must contain
exactly the protected original; shared cases also contain the candidate handle
with exactly one remaining root ticket. Duplicate `takeClean` must retain the
handed-off State. Only after that handoff are the remaining actual tickets
released and their heap releases drained; the final root audit is empty.

## Matrix

- Raw and program reconstruction with a separate protected original.
- Raw and program reconstruction with the real two-ticket archive/search alias,
  returning one candidate ticket plus the separate original.
- Missing second candidate ticket: typed ownership refusal before parsing.
- An unrelated live sequence root omitted from the supplied caller inventory:
  typed ownership refusal before parsing.
- A fabricated protected certificate handle beyond `arena.nextHandle`: typed
  ownership refusal, exercising the final audited-ticket total check.
- Zero-work initial audit: exact work-limit pause, unchanged usage fields, audit
  cursor, audited-ticket counter, arena scalars and actual root inventory.
  `fundInputWork` changes only maxWork; the same custody then resumes through
  actual reconstruction, reads, cleanup and handoff.
- Cancellation while that audit is paused cannot bypass custody validation.
  Funding then permits actual audit and cancelled cleanup to return protected
  tickets. An early program-prefix cancellation checks the same final custody.
- Wrong archive hash after valid custody yields `archiveIntegrity`, rather than
  ownership refusal, and actual cleanup still returns the protected tickets.

Ownership refusals must not produce a clean handoff. Tests require failed cleanup
and then move the actual `OwnershipRefusal` exactly once. Its proposed caller and
basis fields are explicitly unvalidated metadata. Tests audit and release only
known actual fixture tickets from the returned arena; they never treat a forged
or omitted proposal as proof. Duplicate refusal movement and clean movement of
that handed-off State must reject. These refusal cases also finish empty after
releasing their actual tickets.

Expected-root lists exist only as ephemeral `HeapAudit` postcondition assertions.
They are never passed into the production custody audit or retained as input,
length, object or graph metadata. Production audit behavior is exercised through
its own prepaid historical-entry loop and scalar multiplicity counter.

## Gate defect and exact snapshot

A separate scratch-only mutation disables the protected-input skip in
`ClosedArchive.rootAt`. It links and compiles, but the same source owner fails
with `ownership` when cleanup releases the protected original. Official source is
not mutated. The final official-source owner is rerun after this defect check.

The final qualifier refresh includes ProgramAdmission's shared parser transition
change: advance/feed/EOF each compute one parser transition and use its session
and signal. No physical copy-elision claim is inferred.

The accompanying JSON receipt records source hashes, the copied dependency
snapshot, candidate build IDs, native results, the planted defect result, and the
final restored result. Runtime generation preserves the selected new source
modules and slices only unreachable dependency declarations; it does not replace
them with handwritten Lean behavior.
