<!-- uorc:non-authoritative -->

# Owned general candidate byte enumeration

The source contract is `Search.OwnedEnumeration.contract` and the operational
entry is `Search.EnumerationFunding.fund`. This is the general byte producer
for the open UC-SYN-01 capability. It is not a claim that the complete compressor
or its canonical membership bridge is qualified.

The producer starts with the empty word, then enumerates every byte word in
unsigned shortlex order. It scans the existing immutable sequence for the last
byte below 255, copies the unchanged prefix, increments that byte and fills the
suffix with zero. An all-255 word grows to the next all-zero length. Both scans
and the byte builder have resumable, prepaid source-owned events. There is no
candidate-count, alphabet, syntax or mathematical candidate-length cap. UInt64
address exhaustion preserves an unresolved predecessor, rather than completing
the search universe.

Cursor backing consists only of fixed fields, actual handles and the shared
RuntimeByteBuilder cursor. The single Arena is external. RuntimeImage scalar
access borrows from the live current-sequence owner. No Graph, Rope or list is
retained as production backing. The computed root inventory includes the extra
outstanding sequence ticket with its actual multiplicity. Empty/null tickets
are valid only in the explicit outstanding phase with the matching owner,
sequence and length. Every producer generation allocates a fresh control owner.

Funding charges accepted logical credits before the event can execute. A
partial payment preserves the unexecuted primitive; a budget-refused payment
consumes no offered credit. A primitive failure consumes the attempt's paid
quote while preserving the same source and root prefix. Every transition
reports its actual Arena, achieved Usage and release custody. These quotes are
logical task fees, not CPU or RSS measurements. Cancellation abandons unused
prepayment without refunding already charged work and drains actual ownership.
The consumer must return the shared Arena and detach borrowed uses first.

Acknowledgment releases custody only. It does not establish parsing,
admission, execution, fresh archive reconstruction, semantic membership,
exclusion or coverage. The production controller must bind exact current
SemanticScope.Query (logicalReferenceScope1/logicalReferenceEvents1), keep
operational realization separate and retain an unresolved predecessor until
its canonical membership decision is established. A program event or admitted
program is not an archive certificate. Efficient learned/structural proposals
may run alongside this fallback; ordinary best-known output need not wait for
exhaustive minimum certification.

## Qualification and remaining obligations

`python3 tests/enumeration/diagnostic_test.py` evaluates the exact new source IR
and byte-bound official parsed Runtime owners. It is a generic diagnostic,
not native generated execution, a Lean proof or pinned SDK acceptance. It covers
empty and all 256 one-byte words, carry across lengths, all successor ownership
prefix cancellation, wrong-owner acknowledgments, zero credits, budget refusal,
split event prepayment, real allocation refusal/retry and length overflow.

Required source/general obligations include shortlex successor correctness,
no repetition/omission for arbitrary finite words, simulation of every owned
prefix and its root multiset, refinement to the admitted wire/canonical scope,
and a fair dovetail controller. The registered caller/search-owner handoff,
actual parser/admission consumer, native generated roots and full official
release gate remain required work; none is supplied by this diagnostic.

The separate `python3 tests/enumeration/mutations.py` gate plants four defects
in the loaded source IR: omitted successor byte, first instead of last pivot,
refusal advancing the predecessor and a stale empty-ticket acknowledgment.
Each intended diagnostic rejects its defect and passes again after restoration.
The source files remain unchanged throughout these diagnostic controls.
