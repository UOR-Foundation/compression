<!-- uorc:non-authoritative -->

# Canonical semantic scope and operational realization

`Search.SemanticScope` and `Search.SemanticCheckpoint` introduce distinct, explicit source schemas. They do not change the meaning of existing `Identity.Snapshot`, `Replay.Seal`, or `Checkpoint.Record` records, whose resource fields describe the earlier operational diagnostic path.

The new semantic seal binds the actual ordered target bytes, object boundaries and basis objects; archive/machine/wire/source/objective identities; `CanonicalCost.ProfileId`; and the complete canonical semantic budget, including its logical-storage scope. Identity comparison is resumable and compares bytes directly. Neither a digest nor a producer-supplied success flag stands in for equality. Transport trees carrying the same bytes compare equal, while empty objects and basis order remain significant.

Operational work, owned-byte, output and frame admission limits belong to a separate `Realization`, together with operational addressing, implementation source and reference-owned refinement source. Changing that realization does not redefine semantic membership. It can make a particular execution unable to finish. Such a result remains incomplete and cannot exclude a certificate predecessor. Effort, proposal policies and wall time are also outside semantic membership.

A new checkpoint contains this semantic seal, its separate realization, its search policy, an untrusted claimed effort prefix, and optional archive bytes to recheck. Its checker has no result constructor for restored coverage or an accepted incumbent. An exact match requests replay of the claimed prefix from the initial state. A changed realization or policy requests replay from zero. A semantic change rejects the record. A legacy schema is explicitly rejected for reinterpretation. All incumbent bytes must traverse the actual independent wire/admission/execution/output checking path again.

Mathematical source Nat budget/header equality remains a reference identity operation. A bounded production certificate/checkpoint implementation must bind its actual canonical-natural representation, traversal work and target addressability; it may not call arbitrary-size Nat equality a constant physical event. Native UInt64 address exhaustion is an operational inability, not a semantic membership cap.

The canonical cost schedule and reference-owned event/storage bridge remain required, open release obligations. These identity/checkpoint modules neither prove the bridge nor convert current operational counters into canonical semantic evidence. Production semantic-budget replay and minimum certification must bind the actual ledger once that bridge is established; operational failure remains incomplete.

## Current evidence

There are 31 closed source controls for semantic-budget/storage differences, actual target/basis bytes, empty boundaries, equivalent transport partitions, legacy schema rejection, operational-realization changes, policy changes, and replay-only handling of arbitrary claimed prefixes. A generic semantic-JSON interpreter evaluates all 31 as true. Planted mutations that ignore semantic budgets, ignore operational envelopes, reinterpret legacy schema, or trust a changed-policy prefix are rejected by these controls. These are finite source diagnostics, not official LexLean/Lean proof, generated-native execution, or universal refinement qualification.
