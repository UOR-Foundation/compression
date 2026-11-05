<!-- uorc:non-authoritative -->
# Actual completed-view canonical refinement

The authority is `Integrity.CanonicalRefinement`. Its input bridge now derives
the profile computation from actual `StartedView`, `OrderedExecution` and
`Attempt` transitions ending at a ready boundary. It covers real arena/cursor
continuity, charged errors that stutter, zero-admission quiet steps, borrowed
spans, pending singleton updates, input drain and cancellation. The consumer
does not supply an independently stitched sequence of fresh arenas or states.

`canonical_of_completed_view` uses actual `CompletedView` provenance: selected
reader EOF, drained hash input and the same actual finalization transition.
The publication theorem additionally follows a successful actual finish,
commit and publication suffix. It derives the commit's bytes and returned
handle from the actual call and combines the proved image storage effect with
the output node frame. No premise asserts the desired canonical digest equality.

The independent existential success of the executable canonical `Profile.hash`
driver is still explicit in this checkpoint. `ProfileFuelLaws` is proving its
chosen-driver sufficient-fuel bounds. The output node frame during cleanup is
also explicit; Integrity owns the actual control-reservation lifetime and
cleanup discharge. The ordinary archive has no new proof field or input limit.

The result compares the digest with the bytes actually admitted by the ordered
trace. Relating those completed-view observations to an initial source layout
requires the reader's genuine sequence-shape and retained-source invariants.
Publication alone does not establish EOF: a caller may finalize a shorter
observed prefix. `CompletedView` supplies the required boundary independently.

All 113 declarations passed scoped source build, generated kernel checks,
exact axiom audits and leanchecker. Deleting the actual OwnedHash purpose-prefix
update emits the runtime module but invalidates the unchanged start bridge;
exact restoration passes. Current source, 22 dependency identities, alpha-only
authoring renames and exact mutation receipts are under
`validation/integrity/canonical-refinement`. This is not full original pinned-SDK
or product acceptance, a global hash-injectivity claim, or a replacement for
independent byte reconstruction.
