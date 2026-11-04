# Canonical semantic cost profile (provisional)

The source authority is Uorc.Core.CanonicalCost. This is a NEW profile, not an
interpretation of historical StepMachine, archive format1/2 or resource counters.
Its event schedule and its reference/owned refinement bridge are not yet frozen
or ordinarily verified. The current product must report checked reconstruction
and operational metrics, not a semantic-budget minimum certificate.

Identities:
- profile: uorc.canonical-reference-work/1
- representation: uorc.fixed4096-byte-leaves/1
- bridge: uorc.reference-owned-event-bridge/1
- storage unit: uorc.reference-live-binding-payload-bytes/1

Budget contains maxSemanticWork:Nat and storage:StorageScope. StorageScope is
unconstrained or liveBindingPayloadBytes(limit:Nat). All bounds are configured
quantities, not a fixed input eligibility limit. Usage separately carries
semanticWork, semanticStorageUnits, peakSemanticStorageUnits, outputBytes,
activeFrames, peakFrames, acceptedArchives and basisObjects, all mathematical
naturals. Operational RuntimeResources.work/ownedBytes remain separate and must
not be substituted into these fields or used as a semantic exclusion proof.

Scope is semantic execution. Parsing, admission, transport, representation
normalization, ownership implementation and integrity calculation are operational
work. The canonical EOF event is emitted only after exact archive EOF and
integrity have actually succeeded; it does not replace those checks. Conditional
basis objects must be completely decoded/checked first and share the same
semantic ledger with main execution. Each basisBoundary, its execution/output,
and its canonicalEof are charged. No model, parameter, seed, lookup or codec is
free: its archive bytes contribute to the size objective, its actual operation
sequence contributes to execution, and all materialization/verification remains
operationally charged.

The provisional positive event vocabulary distinguishes instruction dispatch,
parameter binding, reference reads, block entry/leave, loop observation/update,
byte reads/writes/comparisons, output boundary/byte, basis boundary, canonical EOF
and value-unit allocation/release. Each event currently costs1 abstract unit;
storeRead/storeWrite instruction events cost65 (one instruction and a fixed64
address-bit primitive fee). These are abstract costs, never host timing claims.
The bridge must account for every reference/capture binding, every sequence
comparison byte and every loop observation/update. A whole unbounded semantic
subloop cannot be hidden behind one event. Immutable sequence view/concatenation
construction has no implicit full-buffer scan; traversal is charged separately.
The exact event trace must be reviewed against the independent reference and
actual owned machine before the profile can be frozen.

Semantic storage counts live binding payloads, not physical sharing or allocator
layout. Natural/byte/predicate bindings weigh8/1/1 units. A sequence weighs its
logical byte length. A store weighs1 default byte plus9 units per represented
key/value leaf (8 address bytes plus1 value byte). Aliases are counted for each
live binding, intentionally differing from exact owned allocation accounting.
Program/archive bytes, source index metadata, control reservations and host
transient storage remain in operational ownership. Store entry counting is a
reference-only mathematical measure; production must maintain a proved cache or
incremental event relation, not perform a whole-store count in one scheduler
quantum. Release underflow and frame underflow are typed failures before commit.

The4096-byte representation is a canonical LEAF policy, not a total object cap.
Admission now normalizes offered literal spans through the source-owned native
byte builder: full4096-byte leaves, followed by largest-to-smallest power-of-two
residual leaves. It preserves the producer image owner through full span
acknowledgment and charges all reads/copies/ownership work operationally.
Source finite controls check ordered bytes, canonical residual shape and root
inventory for one-span versus split-span input. The general order/partition
refinement law remains an ordinary proof obligation.
Partition-equivalent output alone does not prove equal resource work. The bridge
must advance the semantic ledger alongside real transitions, without rerunning a
second complete decoder to discover the cost.

The source includes exact increment and list-composition laws. Their source
proofs are obligations pending ordinary checker/axiom audits; local AST shape or
finite scenario evaluation is not proof or SDK acceptance. A bridge identity or
externally constructed Usage record is not evidence that the bridge ran.
