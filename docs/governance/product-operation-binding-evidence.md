<!-- uorc:non-authoritative -->
# Product operation binding evidence

The authoritative new source is `Uorc.Product.MachineBinding`,
`Uorc.Product.OwnedCompare` and `Uorc.Product.MachineBindingScenarios`.
This checkpoint is based on public integration `f947c988c6769fac2d94a850afee19015d18653c`.

`MachineBinding.start/tick/extend/cancel` call the actual owned
`RuntimeMachineEngine` roots. Its owner sum retains exactly one engine or Ready
carrier. On completion it transfers the single output ticket, keeps that ticket
in its root inventory while engine cleanup runs, then obtains the achieved
Ready through `takeReady`. `moveTerminal` transfers those actual owners once and
leaves a carrier with no arena or output ticket. Cancellation, typed execution
failure, suspension and frozen overflow receipts remain distinct. Completion
alone does not produce a successful archive Report.

`OwnedCompare` walks the actual immutable sequence roots through `Wire.OwnedRead`
and compares every byte through the runtime image reader. Its fixed state holds
borrowed cursors/spans; enclosing owners remain live. Length mismatch, byte
mismatch, invalid source shape, counter exhaustion and cancellation stay typed.
`tick` prepays the declared source work before comparison. No digest match is
used as byte equality, and no Graph or mathematical Rope backs these operations.

The source-owned runtime controls construct real input images and sequence
owners, complete actual `RuntimeAdmission` before starting the machine, retain
one-based backward distances, audit real root inventories, exercise terminal
handoff/cancellation/stale runs, and independently compare all 256 byte values.
They also include same-length byte mismatch, length mismatch and wrong-shape
negative controls. Their finite harness limits are test allowances, not product
or format limits.

The isolated official diagnostic conversion/check built the complete 35-module
binding closure. The complete generated Lean closure kernel-compiles and executes. Eleven of
twelve actual binding controls pass. The all-256-byte admission/engine/comparison
control currently fails and is being isolated without reducing its input or
expectation. Mutation qualification is still pending; this checkpoint does not
assert full binding acceptance. The
previously published product-control receipt has been rerun after the narrow
nested binder repairs: 112/112 controls and all four defects passed/rejected as
intended, with its own exact source hashes. That component receipt does not
substitute for execution of the new binding or the full product.

The closed `RuntimeAdmissionCreditBoundary` and `RuntimeMachineEngine` APIs are
now present in the integration source. The next product path connects the real
owned wire producer/parser to that credit boundary and the machine, then expands
the archive-specific operations using the production frame/demultiplexer,
format-3 integrity adapter and owned search producers. Those source owners are
under active integration; there is no unavailable operation stub or fabricated
completion Report here.

The generic persistent host still needs the actual complete typed export and
constructor closure, including distinct Invocation/Update handling and
source-owned resource custody/release disposition. The earlier `sameResource`
export establishes only that extracted root and its configured prefix. Full
native/browser operation, keyboard/focus/publication-race qualification and the
exact pinned SDK gate remain release obligations. Repository locks and universal
policy files have not been changed.
