<!-- uorc:non-authoritative -->

# Byte-derived linear model proposals

Authoritative behavior lives in `Uorc.Search.LinearFit` and its source-owned
controls, not this implementation note. The strategy identity is
`uorc.search.byte-linear-fit/1`. It is a discovery strategy, not an archive
universe restriction, decoder authority or minimum certificate.

The fitting cursor retains only fixed scalars. Observations are actual bytes,
not caller-injected model parameters. It tracks a wrapping affine prediction
and the byte-state family `next = (a * previous + c) mod 256`. After the first
two bytes, each multiplier determines its increment. Surviving multipliers
are represented by a residue and power-of-two modulus. A constraint that
cannot distinguish this coset is checked directly; otherwise each explicit
filter step checks one multiplier. No observation scans the previous input
or invokes a whole candidate-family callback. Offered bytes are acknowledged
only after filtering and the count/previous-byte commit finish. EOF rejects
pending observations, and proposals require an explicit completed EOF.

Every fitting operation declares the shared operational control-task fee.
Actual input traversal, checked scalar access, pinned owners, the cursor's
storage reservation, scheduling and cleanup are separate operational tasks.
These fixed-scalar source definitions are not yet the owned observation
controller: binding them to the authoritative scalar image accessor and an
actual reserved owner remains required. The host-width count failure is a
typed operational inability, not a semantic input cap or predecessor exclusion.
Learned proposals are ordinary FixedTemplates; only complete emitted archive
measurement and fresh parsing/admission/execution/original comparison may
make one an eligible incumbent.

## Retained diagnostic evidence

The generic semantic-JSON evaluator interpreted all 19 closed source controls
as true. They cover wrapped affine values, learned nonlinear state parameters,
ambiguous even cosets, constants, contradictions, exact cursor and proposal
partition equality, pending/finished protocol errors, invalid scalar metadata,
count exhaustion and checked arithmetic failure. A scratch independent oracle
enumerated all 256 coefficients after every observed prefix, including a
sequence for every multiplier. It checked 310 cases and 3,955 prefixes,
reconstructed each emitted description independently, and compared exact
cursor/logical-event prefixes across empty, singleton and uneven delivery
partitions. Maximum observed filtering plus commit was 257 events for one
byte, not a per-input or semantic bound.

Three planted changes to constraint formation, solution comparison and emitted
parameters were rejected, then restored. The initial solution-comparison
mutation witness had equal difference and target and did not distinguish the
defect; the retained log records this ineffective witness before replacing it
with the nonlinear state witness. No source behavior or guard was relaxed.

The evaluator retained its 10,000,000-term per-root-call guard. Its ordinary
Python recursion limit could not evaluate five recursively composed fixture
terms; those failures remain recorded. Re-execution with a declared 50,000
Python stack limit evaluated the same source and unchanged term guard. The
independent oracle driver separately invokes each bounded source event. Its
35,617,532 total generic term evaluations and 17.17 diagnostic seconds are
interpreter observations, not canonical work, native wall-time or acceptance
benchmarks. The evaluator identity was
`7b8df5cb4fa819571db745fcc77f72baf2e38fe1f4c5d5d21b0169715f06fbc8`.

All generic tools/logs and mutation witnesses remain in persistent scratch,
not product execution or the published source. These are finite diagnostics,
not SDK/kernel verification, generated-native feasibility, a general coset
correctness proof or full archive eligibility. Those obligations remain open.
