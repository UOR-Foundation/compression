<!-- uorc:non-authoritative -->
# Logical reference event profile

Source authorities are `Uorc.Core.CanonicalCost`, `CanonicalBindings` and
`CanonicalSchedule`. The current typed profile is `logicalReferenceEvents1`,
identified by `uorc.logical-reference-events/1`. It is distinct from the published
provisional `canonicalReferenceWork1`. The old constructor remains recognizable
but `profileSupported` rejects it; its old checkpoints require fresh replay.
`budgetSupported` also rejects the old `liveBindingPayloadBytes` storage scope.
It does not reinterpret old evidence or modify ordinary archive format identities.

The value and storage identities are `uorc.logical-reference-values/1` and
`uorc.logical-live-binding-payload-units/1`; the event schedule identity is
`uorc.logical-reference-event-schedule/1`. The intended, still unproved runtime
bridge identity is `uorc.logical-reference-owned-binding-bridge/1`. These names
identify source contracts, not certificates that the bridge has executed.

This is a reference semantic execution measure. Parsing, static admission,
integrity, transport, normalization, allocation/index housekeeping and cleanup
remain operational `RuntimeResources` work and owned bytes. They never advance
this semantic ledger merely because a host operation occurred. Native CPU time,
RSS and transient ownership observations are separate again. The implementation
must currently report checked reconstruction with operational metrics, not a
semantic-budget minimum claim: the full owned-machine refinement is open.

## Events and storage

`Budget` holds a mathematical-natural `maxSemanticWork` and either unconstrained
storage or `liveBindingPayloadUnits(limit)`. There is no fixed total input cap.
`Usage` tracks work, current and peak storage units, output bytes, active and peak
frames, accepted archives and basis objects. An event is checked before semantic
commit; work/storage/underflow errors retain the preceding committed prefix.

Every instruction dispatch costs one abstract unit, except store read/write,
which have the explicitly weighted cost65 for a fixed64-bit-address primitive.
This is not a timing assertion or a count of the physical store tree. Parameter
binding, reference reads, frame entry/leave, loop observation/update, logical byte
read/write/comparison, output boundary/byte, basis boundary, EOF, and each binding
allocation/release event cost one. Allocation payload affects storage, not the
number of event units. The schedule reuses `RuntimeOpcode.spec` for all39 opcode
signatures and operand/capture roles; it has no independent fee table.

A live natural, byte or predicate binding weighs8,1 or1 logical units. A sequence
binding weighs its logical length. A store binding weighs1 plus9 per non-default
key/value exception. These are named weighted logical units, not an encoding-size
or allocator-byte assertion for arbitrary malformed reference carriers. Reachable
natural scalars obey the existing checked U64 language operations, including the
existing `sequenceLength` overflow rule; arbitrary-length sequences are not
restricted to make this measure convenient.

Each live alias counts its full logical binding weight. Caller bindings remain
live while borrowed captures are prepared; captures do not allocate a second
semantic binding until callee parameters are bound. Each callee parameter emits
`parameterBind` followed by allocation of its logical payload, in source order.
Frame entry precedes them. A fresh instruction result is allocated before the
last-use release bucket supplied by the existing lifetime authority. The return
reference is read, remaining non-return bindings are released, then the frame
leaves. The returned binding transfers its role into the caller without another
allocation. Duplicate aliases/captures are distinct bindings, not shared weights.
Physical handles, program bytes and metadata do not substitute for this measure.

Production store updates need a maintained, proved exception-count witness;
`referenceValueUnits` is a mathematical reference definition and must not be
implemented as an uncharged whole-store traversal in a bounded native step.

## Concrete instruction and loop schedule

Instruction prefixes emit dispatch and each local/block reference read in the
four fixed signature positions. Each actual capture reference emits a separate
read during call preparation; repeats bind fresh parameters in each callee frame.
Immediates do not add reference events. Scalar checked operations use their
fixed-width primitive dispatch. Immutable sequence concat/slice create views and
add no full-buffer traversal. A literal reads each logical payload byte once,
then allocates its result binding. Index emits a logical byte read only once a
valid index is observed. Equal-length comparison emits two reads and one byte
comparison for each examined pair, stopping at its first difference; unequal
lengths require no byte pair. A singleton emits one logical byte write. Basis
lookup allocates a new alias binding with its full payload units after its source
reference; it has no hidden network or model lookup.

Every loop condition, including the final false condition, emits `loopObserve`.
Every completed iteration emits `loopUpdate`. Tabulate starts an output binding
of zero units, calls the body with index and captures, then appends its returned
byte: one write, one output-growth unit allocation, and release of the returned
byte binding. Fold clones the initial state into a loop-state binding; each body
returns a binding which replaces the old state after releasing the old binding.
The final state transfers to the instruction result. Scan clones initial state
and creates an empty output binding; it calls the **observer**, appends/releases
its returned byte, then calls the **updater**, replaces old state, and completes
the iteration. Its final updater is not omitted. On exit, the final state binding
is released and the output binding transfers. Zero-count loops still perform
setup/final observation/teardown, without any body frame. These accumulator
bindings are explicit, not inferred from final Heap usage.

Each ordered output object emits its boundary even if empty, then one byte read
and output-byte event per logical byte. Each supplied basis object emits its
basis boundary and shares the same ledger for its actual decode and output.
Completed basis bindings transfer into the basis context and remain charged
until that context ends. `canonicalEof` can be committed only after actual exact
EOF and integrity acceptance; the schedule helper is not permission to invent
that observation. No model, parameter, seed or codec payload is free.

`CanonicalSchedule` realizes event lists and a retained-prefix reference event
cursor for the literal/parameter/frame spine. It does not evaluate another full
graph, own a Heap, or purport to enforce an externally forged trace. General
source laws connect its literal meaning to `ReferenceSemantics.evaluateInstruction`,
preserve meaning across committed events and role transfers, and express
additive binding inventories. Their ordinary proof/axiom qualification is pending.
The remaining bridge must attach these observations to every real owned-machine
transition, including failure prefixes and static last-use buckets, without
rerunning a duplicate decoder or accepting supplied event records as proof.

`CanonicalRuntimeRelation.bindingRelated` reuses the actual
`RuntimeMachineRefinement.valueCorresponds` denotation and separately requires
that the native per-binding unit witness lifts to the reference payload measure.
Role transfers and parameter aliases preserve this relation. Scalar general-law
candidates connect all natural/byte/predicate bindings to that existing runtime
denotation; no second Heap reader or graph evaluator is introduced. A raw cache
record or unit number is not evidence of the relation. In particular, extensional
store equality alone does not establish the cardinality of a malformed exception
list: the production store/count invariant and its update preservation are still
required. These definitions are proof-side, not executable production caches.

## Realization independence and evidence boundary

The4096-byte native builder policy remains `uorc.fixed4096-byte-leaves/1`, exposed
only as a realization/normalization identity. Full4096-byte leaves and ordered
power-of-two residual leaves are not a semantic input cap and do not enter this
profile's membership. Host span partition, rope balance, alias sharing, and
transport copies may change actual operational work, never logical event meaning.
The legacy `representationIdentity` accessor exists only for old provisional
callers; new semantic scopes must not bind it as part of the logical universe.

Source shape checks and generic AST evaluation are authoring diagnostics only.
No source theorem here is described as proved until ordinary checking and exact
axiom audits pass. No complete bridge or semantic-budget minimum is claimed.
