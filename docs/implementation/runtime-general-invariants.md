# General ownership and representation proof scope

This records the scope of the source proofs, not a second semantic authority or
a completed product refinement claim. The e87 runtime APIs and their completed
seven-suite/406-policy checkpoint remain the baseline. New proof-only modules
do not enter a runtime closure.

## General statements implemented

`RuntimeAuditProofs.countsRows_reflect` proves, for arbitrary `rows`, `all` and
arena, that the actual Boolean `HeapAudit.countsRows` succeeds exactly when the
independent propositional `CountsInvariant` holds. Each row has positive
unsigned incoming count, and its mathematical incoming count equals its root
count plus parent-edge occurrences plus pending edge tokens. The initial
transferred release-root token is excluded from pending-edge accounting.

`RuntimeAuditProofs.rootRows_reflect` similarly equates the actual root checker
with `RootsInvariant`, for arbitrary rows, caller roots and arena. Each root
count equals caller-ticket occurrences plus the pending initial root ticket
when `rootStep` is true. These are structural List-induction proofs, with no
finite test domain or valid-arena premise. They do not prove that a supplied row
list is the complete index enumeration, or that `wordToNat` implements the
mathematical unsigned conversion. Three positivity/count reflection lemmas use
the exactly declared standard `Classical.choice` axiom in addition to
`Quot.sound` and `propext`; the runtime's prior 406 policies are unchanged.

`RuntimeObservationProofs.lookupFrame` proves actual lookup is independent of
arena bookkeeping outside its index. `interpretSequenceFrame` proves the same
for the independent sequence interpreter, for **every Nat fuel**, arena and
handle, by induction on fuel. Its proof-only `observationArena` keeps the exact
resident index; its zero bookkeeping is not asserted to be a live valid arena.
This proves an observation frame law, not sufficient interpretation fuel or
operational preservation of the index. Dropping the index in an isolated
defect makes the owning lookup and interpretation theorems fail kernel checking.

`RuntimeOwnershipProofs` supplies 28 quantified laws over real Heap operations.
They cover arbitrary-index null/active/idle protocols; symbolic singleton-root
lookup, release and withdrawal; separate root/edge retain counts; all byte and
budget values for allocating/reclaiming a store leaf; and a full retained-image
retain/release/releaseStep roundtrip. The latter composes explicit step lemmas.
The singleton families quantify their payloads, index/counter parameters and
budgets; they do not establish successful-operation preservation for every
possible arena graph. The unrestricted all-arena success-frame obligation is
still open.

`RuntimeSequenceProofs` supplies 12 quantified local bridge laws: null and
null-left plan identity/interpretation, exact cancellation ticket-field
partition, result-ID preservation, idle terminal transfer and cancellation,
and empty-concat capture through actual `Heap.allocate`. The terminal families
use an explicitly idle arena and appropriate phase cursor. They do not license
clearing an active release cursor or prove all-phase reachability/termination.
Ownership interpretation also needs the independent root auditor to accept
the state and the complete caller inventory.

## Minimal remaining general obligations

The additive RuntimeIndexProofs now proves actual readIndex/writeIndex same-key
insertion and deletion for every Nat depth, arbitrary Index, UInt64 key and
Entry, without a shape premise. Its two constructor families exhaust Option
cases. Ten general theorems and an actual bit-order mutation gate pass; see
[the exact scope](runtime-index-proofs.md). This closes the same-key bridge,
while distinct-key frame, bit injectivity and complete enumeration remain open.

The global invariant can be carried by existing `Heap.Arena`/`RuntimeValue`
and the independent `HeapAudit` roots, with the following obligations composed:

1. **Complete representation.** Prove `scanIndex` enumerates exactly the unique
   live handles observed by fixed64 lookup, including MSB read/write agreement.
   Every nonzero child exists and is older than its parent. Node tags, image
   bounds, length/height caches, sparse-store depth and table kind/depth satisfy
   their source representation rules. Fresh allocation never reuses an ID.
   Roots: `Heap.readIndex`, `writeIndex`, `lookup`, `validNode`, `nodeChildren`,
   `HeapAudit.scanIndex`, `acyclicRows`, `nodesRows`.
2. **Global ownership and accounting preservation.** Lift the proved count/root
   reflections to the complete index, prove weight/totals/cursor reflection,
   then prove the invariant initially and after every real Heap operation.
   Counts include edge multiplicity and the distinct pending-root protocol;
   entry bytes equal the sum of live weights, and pending edge tokens carry
   their declared charge. Errors preserve the caller's prior state, while
   successful prefixes remain charged. Roots: `HeapAudit.auditRoots`,
   `countsRows`, `rootRows`, `weightsRows`, `sumWeights`; `Heap.allocate`,
   `retainEntry`, `allocateMoved`, `reserveImage`, `commitImage`, `release`,
   `releaseStep`, `dropRetainedRoot`, `withdrawRootRelease`.
3. **Representation refinement.** Supply adequate fuel from well-founded
   handle order and relate sequences, stores, tables and runtime values to the
   independent mathematical authorities. Prove concat/slice/index/span,
   pointwise default-store reads/writes and real table erasure implement those
   meanings, including alias and unchanged-write cases. Roots:
   `HeapAudit.interpretSequence`, `RuntimeSequence.begin*`/`step`,
   `RuntimeStore.beginRead`/`beginWrite`/`step`, `RuntimeTable.lookup`/
   `beginPut`/`beginErase`/`step`, and the machine's independent reference bridge.
4. **Continuation and returned-owner composition.** Prove each reachable
   continuation phase owns exactly its documented tickets; cancellation and
   typed failures preserve that partition and eventual reclamation. Compose
   program, basis, slots, continuation, input/archive, search, wire, checkpoint
   and returned-outcome owners, plus charged fixed control records. Roots:
   each primitive's `ownedRoots`, builder `builderRoots`, the new
   `RuntimeSequenceTickets.outcomeRoot`/`outcomeRoots`, and RuntimeProgram/
   RuntimeMachine inventory roots owned by their integration lane. A nonempty
   terminal span contributes one image ticket independently of the now-empty
   cursor and the pinned original sequence root.

Pre-call resource quotes are a separate interface obligation. e87 already has
`Heap.nodeWeight` for owned-entry byte admission, but no dedicated pre-call
logical event-work API. The additive `RuntimeCost` module distinguishes unit
event work, cumulative table deltas, ownership-operation work, copying and
reservation bounds. A quote's arithmetic fact is not a proved universal binding
to an actual successful primitive result. Failed structured-proof candidates
for that binding are not retained as theorems or reported as acceptance.

## Physical realization boundary

The inspected official renderer uses shallow Rc sharing. This does not measure
index/control-block overhead, payload copies, transient coexistence, or retained
old arenas. A measured native retention/copy bound must name the accepted
generated artifact and caller-history policy. Logical entry sums and the
source-hashed structural index-write audit cannot establish RSS or timing.
Accepted export, physical measurement and full locked-SDK release acceptance
remain separate from these source/kernel proof results.
