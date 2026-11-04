<!-- uorc:non-authoritative -->

# Operational task work

Uorc.Core.RuntimeWork owns uorc.operational-task-fees/1. These are explicit
abstract fees for bounded implementation tasks. They are not exact host CPU,
allocator traffic, index-write counts, or the canonical semantic certificate
budget. Native transition.work=1 is not a replacement for these task charges.
Actual Heap.delta ownership events, index visits/writes, copied bytes, wall time
and RSS remain separate observations and must not be silently discarded or
relabelled as this abstract fee.

A producer/controller prepays fee(operation) before invoking the corresponding
bounded action. The same action is not executed first to discover its fee.
Partial funding retains only the current arena/state. FeeWithCopy checks the
base fee plus2×known copied bytes before a copy; overflow is operational inability,
not malformed input or semantic-budget exclusion. BuilderStepFee derives the
known copy bound from the actual native cursor phase (singleton or merge),
without constructing an output or copying any bytes.

Current source fees include control/ack64; sequence length/begin/step/cancel4096;
heap retain/release/releaseStep256; owner pin/join1024; owner split start/step256;
control allocation256; table/metadata mutation1024; indexed-start4096; scalar
checking32768; capture checking16384; lifetime planning16384; seal checking8192;
admission initialization1024/offer1; producer take/reattach4096 and return256;
builder control4096 and bounded builder step12288 (4096 base plus twice the
known maximum4096-byte leaf copy).
These fixed fees bundle explicitly bounded fixed64 access/control operations.
They do not assert an independently proved inequality against literal processor
instructions. Byte copies require their separate pre-call bound; an unbounded
whole-buffer callback cannot be hidden under a fixed task fee.

Admission, owner joins/splits, cancellation and producer boundaries use these
shared roots rather than independent copied constants. Search must charge native
navigation and cleanup through this policy, not merely its local transition1
counter. Operational limits may stop a run and retain its work/roots; they never
establish exclusion from the sealed canonical semantic universe. A changed fee
policy must remain explicitly bound to a realization/run policy, not silently
change a semantic minimum statement.
