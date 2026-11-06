<!-- uorc:non-authoritative -->
# Framing-backed inspection reports

Authorities are `Uorc.Product.InspectionReport`, its source scenarios, and the
imported ArchiveSnapshot/Profile owners. `start(run, wireTransition)` retains the
actual taken snapshot transition, its one Arena/Usage and archive/length tickets.
Measurements bind run, requested archive lease and borrowed archive root; they
supply I/O counters and optional physical observations, never correctness flags.

`complete(budget, currentControl, measurements, state)` checks current run ownership
and the actual taken snapshot. It reserves a real 4096-byte report control before
Profile.select or report construction, and prepays the shared projection work.
Actual allocation deltas require acknowledgement. Stale, waiting, refused and
failed attempts retain their source owners. `take(currentControl, state)` moves
one Envelope with report control, report and actual Wire carrier. `moveWire`
returns retained custody for enclosing cleanup on noncompletion.

Archive length and object count come from the framing snapshot. Transport,
external basis and basis transfer are scoped observations. Checker work is the
actual cumulative inspection-job Usage; this owner runs no decode, replay or
search. Logical peak Arena bytes, Storage and physical observations stay separate.
Physical None means unobserved.

The report records parsed framing. It does not promote framing identifiers to
hash observations, or establish integrity, complete reconstruction, execution,
original-byte equality, certificates, search exhaustion or publication. Its
six-octet archive prefix, profile octet and format/profile pair name framing
identities; they are not digest checks.

The frozen diagnostic selects 53 source files: 19 captured local Core, 19 Wire
from `2de910755e66a145891669d2d45a0c8154ea1b32`, 5 captured Integrity and 10 Product.
It uses old raw Control `405b7a83…` (diagnostic `892f5883…`), before current shared
record factoring. Its 19/19 VM controls, 56 compiled Lean modules, 63 exact
new-family axiom policies and three owning mutants qualify that composition.
All 53 generated Lean modules match the archived pre-annotation checkpoint.

The in-place mutant runner restored original sources; raw mutant input bytes
were not retained. Exact input hashes, generated mutant Lean/compiler source
renderings, transforms and observed failures are archived. No current factored
Control, whole-current graph, accepted export, native ABI, NativeLibrary/Wasm,
closed product operation or pinned-SDK acceptance follows from this checkpoint.
