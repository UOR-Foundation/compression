# Event-ledger integration audit and prefix checkpoint

Audited engine: ad6a4507f5f4ac1b3637ed5b28b0003b8329295c2b14a3830bd20f83d768bf69.
Audited CanonicalSchedule: 2d70f4682cd814a8939848af805f4e71bcb70bedd927f00aa40fddf663f17ebe.
Schedule identity: uorc.logical-reference-event-schedule/1. Ledger authority is
RuntimeMachineSemantic, current logicalReferenceEvents1. This section records the earlier audit. The instruction/reference prefix is now implemented; the remaining seams below retain their stated obligations.

## Smallest unambiguous initial seam

Current phase3 fetch combines instructionAt and prepare. prepare can read operands,
check arithmetic, fetch basis, and fail semantically. Split this existing bounded
phase into metadata fetch; prepaid prefix cursor; then actual prepare. Store the
existing BorrowedInstruction in FrameRecord.instruction before prefix consumption.
Use RuntimeOpcode.spec(opcode).operand0..3 directly (fixed executable record), not
a copied39-opcode schedule or event list. Consume Semantic.Event.instruction first,
then one referenceRead for each local*/blockReference role in positional order.
Absent/immediate roles produce no event. The exact reference contract is
CanonicalSchedule.instructionPrefix -> operandPrefix -> operandEvents.
Maximum prefix is5 events: Scan dispatch plus4 reference operands. There are four
operand positions, not four total events. Checked divide/remainder failure occurs
after the entire prefix. Invalid/released protocol metadata cannot establish a
semantic exclusion; it is operational/incomplete. Admission/lifetime refinement
is still needed to justify prepaying reference reads from immutable role metadata.

Each event uses Semantic.commit/charge before source semantic effect, updates the
existing authoritative Ledger only on acceptance, and advances its fixed cursor
only once. Refusal retains exact ledger/cursor/arena/ownership prefix and invokes
no prepare or primitive. An operational refusal after accepted semantic prepay
must retain a paid continuation; retry must not charge the event again. Existing
RuntimeWork guards remain separate and precede the work of the ledger step itself.

## Phase map and available witnesses

| Current engine site | Exact schedule contract | Event before effect / witness |
| --- | --- | --- |
| phase3 fetch, before prepare | instructionPrefix(opcode) | instruction then spec-role referenceRead; immutable opcode/operand roles, no payload read needed |
| phase9 invoke/enter, before semantic callee entry | callPlan.entryEvents / CanonicalBindings.frameEntryEvents | blockEnter, then source-order parameter events; exact capture-read versus entry ordering needs clarification below |
| phase0 argument | CanonicalBindings.parameterEvents | parameterBind then allocateValueUnits before beginPut; coordinate fixedValueUnits(natural)=8; state prefix copies parent.accumulatorUnits |
| phase1 capture | captureReadEvents(count), then parameterEvents for the argument | one referenceRead before captureItem; units from parent localIndex and unitsAt; repeated captures count separately, each call resets cursor; observer first lane/updater second for Scan |
| phases27/28 witness bind | same earlier parameter binding, not a second event | native data/witness dual commit acknowledges one prepaid logical parameter; slotCount advances after both roots installed |
| phase5 result put / phase29 result witness bind | completedResult(binding, deadBindings), for fresh results only | allocateValueUnits(State.valueUnits) before logical result binding; no extra allocation for cache metadata or ticket movement; Apply/loop origin classification remains unresolved |
| phase7 selects bucket; phase8 lifetime before data erase | CanonicalBindings.lastUseEvents(checked bucket) | releaseValueUnits from unitsAt(current unitsTable,key), in existing bucket order; save units before beginErase; phase30 witness erase is the same logical release, not another event |
| phase12 returnValue; phases13/31/14 table retirement/pop | returnPlan / frameReturnEvents | return referenceRead; release non-return live bindings; blockLeave; returned role transfer emits no allocate/release. Current bulk root retirement lacks the required logical slot cursor |
| phase10 childReturned Apply | CanonicalBindings.move/transfer | no event for the move itself; State.valueUnits unchanged into parent result; surrounding result allocation rule requires Core clarification |
| phase9 finished Fold | same move/transfer | accumulatorUnits -> valueUnits, no allocation merely because ticket moves |
| phase10 updater result replacement | replaceLoopState(oldState), completedIteration | old accumulatorUnits supplies release; new returned State.valueUnits moves into accumulatorUnits; exact commit timing is part of Core clarification |
| phase4 loop setup; phase9 condition | outputAccumulatorStart/stateAccumulatorStart/loopCondition | output0 and state source units available; loopObserve includes final false condition, must precede entering/finishing branch |
| phases23/22 append/afterAppend; phase26 accumulator retirement | appendReturnedByte/completedIteration/scanFinish | byteWrite, allocate1, release1; old accumulatorUnits for state release. Builder chunks/physical allocation do not produce these logical events |
| phase4 literal/index/equal/singleton/basis paths | literalTraversal/indexObservation/comparisonObservation/singletonConstruction/basisLookupEvents | literal length, index validity, comparison position and byte-pair state are available; new per-byte cursor stages required before actual reads/writes, not on Outcome publication |
| mode3 publish / takeOutput | outputObjectEvents(length) | boundary even empty plus byteRead/outputByte per logical byte; publication currently only checks length and detaches handle, so it cannot prove an actual emitter byte acknowledgment |
| surrounding basis/EOF controller | basisObjectEvents/acceptedEofEvents | not inferred by Engine: Ready is already admitted and Engine has no input/EOF demand |

Loop condition detail: phase9 is also used for Scan's updater call. Charge
loopObserve for Tabulate element/finished, Fold update/finished, and Scan
observe/finished, never again for Scan update. completedIteration belongs after
Tabulate append or Fold/Scan updater replacement, not after Scan observer append.

For byte paths, Q.beginIndex validates range but actual byte read is in Q.step
phase5/indexWalk at a leaf; Outcome.byte is reported only after cleanup. A
post-outcome callback is too late for prepayment. Equal compares inside the right
byte callback, so comparison must be a separately prepaid continuation before
that callback. Literal dispatch currently only retains its immutable sequence;
it has no per-logical-byte traversal. Realization of literalTraversal therefore
needs an explicit Core-agreed logical byte cursor, not an inferred event total
from final length. Slice/Concat have no traversal events merely because native
balancing walks nodes. Store's65 instruction charge is once per logical store
read/write, never once per native fixed64 owner step.

## Exact unresolved schedule decisions

1. captureReadEvents and callPlan/frameEntryEvents are separate helpers; no source
   composition fixes whether all capture reads precede blockEnter, or how reads
   interleave with prefix/explicit parameter binding. Existing runtime enters a
   physical child first, then binds prefixes before reading captures. Physical
   frame control allocation is operational and cannot settle this semantic order.
2. completedResult always allocates its binding, while returnPlan transfers the
   returned binding without allocation. Apply, final Fold state, and completed
   Tabulate/Scan output must distinguish fresh binding from role transfer. Existing
   valueOwned and handle identity cannot distinguish these: scalars/empty sequence
   may have null handles and aliases count full independent binding weight.
3. frameReturnEvents(otherBindings) releases the supplied list in its order, but
   the runtime must derive the exact other live bindings, excluding returned key,
   from sparse slots/cache without copying a list. Current retireSlots retires an
   entire physical table, whose Heap release order is not this logical schedule.
4. appendReturnedByte explicitly allocates growth1 and releases returned byte1;
   final builder publication transfers an already-live output binding. Core must
   settle composition with instruction completion to avoid allocating final length
   twice, and distinguish old state release from new returned-state transfer.
5. Action helpers such as indexObservation/basisLookupEvents are not a complete
   failed-action driver. For example index emits no read on invalid index; basis
   lookup failure ordering needs the exact authority decision. The initial
   instruction/spec prefix itself is unambiguous and can be realized first.

## Fixed-width retained state and handoff requirements

- Semantic.Ledger is profile tag + eight UInt64 counters (modeled72bytes).
  Semantic.Budget is profile/max work/storage tag/max units (modeled32bytes).
  Use the existing types and checked charge, not CanonicalCost Nat Usage at runtime.
- Minimal prefix cursor needs a closed stage tag and bounded operand position;
  opcode is borrowed from the existing immutable instruction. Add count/index and
  a saved UInt64 payload only for actual capture/binding/byte/release stages. No
  retained SemanticEvent list, CanonicalSchedule.Cursor, Binding list, Nat, or
  reference Value. Existing8192 control and4096 frame reserves must be re-audited
  for whichever exact fixed fields are approved, with each frame cons still owned.
- Prefix+capture destinations, live-slot traversal, literal/comparison/output byte
  counters, and ledger work/storage/frame/output/archive/basis additions must be
  checked before effect. Avoid length+1/end sentinels at MAX. Numeric inability
  remains operational wordOverflow/addressSpaceExhausted, never a Ref bound.
- Use existing charged unitsTable, accumulatorUnits and valueUnits. Cache0 is a
  valid empty payload, distinct from missing. Cache/refinement and prefix-paid
  pending-effect state require actual relations, not assumed invariant booleans.
- Semantic budget ingress, extension/refusal API, and ledger handoff across
  Ready/output/basis scopes are not yet present in Engine. start/tick/extend use
  operational Budget only; takeReady returns operational Usage only. A controller
  contract must thread one semantic ledger across basis/decode/output/exact EOF
  without independent Arena snapshots or silently resetting scope at each Ready.
- Output/basis transfer preserves logical binding ownership until its context ends;
  native root release or takeOutput clearing valueUnits cannot be treated as the
  corresponding canonical release without that controller contract.
- Native Heap/Table/Sequence/Store/Builder housekeeping and cancellation cleanup
  are operational, not automatic logical events. Cancel/error preserve accepted
  semantic ledger prefix and paid cursor; no rollback claim or inferred release.

## Validation boundary

At the audited checkpoint the Engine had cache witnesses and operational prepayment, but no semantic Ledger field or event-consumption cursor. The following increment implements the first prefix seam; no complete semantic-budget acceptance is claimed.


## Common-source parser boundary control

The same-Graph Scan control exposed a separate controller composition problem:
ProgramParse.advancePaid returns pending before inspecting Heap.isReleasing.
Calling advance repeatedly to drain a release while an event is pending cannot
progress, although it still prepays work. Boundary.reattach correctly rejects a
foreign active release. The producer lease must retain the pending event and
finish the actual release cursor with prepaid release work and observed achieved
Usage before returning the single arena. Clearing or acknowledging the event
before admission, returning a second arena snapshot, or weakening foreignRelease
would conceal the problem. The source-owned control repair and its execution are
tracked separately from decoder arithmetic and canonical-event refinement.


## Implemented instruction/reference prefix

The Engine now owns semanticBudget, semanticLedger and semanticError in State,
plus a fixed UInt64 semanticPosition in each charged frame. Phase3 fetches immutable
instruction metadata; phase32 consumes dispatch and the four actual spec roles,
one bounded position at a time. Position5 performs prepare under its separately
prepaid operational scalar-check fee. Semantic refusal preserves the accepted
ledger and exact cursor; tick is inert until supported budget extension. Native
work refusal after accepted prefix payment retains position5 without charging
those semantic events again. Cancellation bypasses the semantic-error stop so it
can clean up while preserving the semantic prefix.

Production ingress is startWithSemantic(Ready, operationalBudget, semanticBudget,
ledger). The fresh standalone start helper remains for component diagnostics and
is excluded from the production request. Controllers must inspect semanticStatus
alongside the operational disposition. extendSemantic retries only work/storage
budget refusals; fixed-word/profile/invariant inability stays operational and
cannot exclude a predecessor from arbitrary-Nat semantic scope.

takeReady now returns ReadyTransfer containing ready, semanticBudget and
semanticLedger. Controllers unwrap .ready and carry the same ledger into the next
basis/decode/output/EOF scope. OutputTransfer already contains the State with its
ledger and transfers exactly one output ticket. These fields remain bounded native
controls; no reference Value, Nat counter or event list is retained. The existing
machine/frame control reserves remain charged, while physical layout and complete
controller ownership transfer require the accepted export/whole-product audit.

The general native role/prefix agreement and five-position-success laws kernel
check against the exact authoritative instructionPrefix/operandPrefix/operandEvents
bodies. Omitting block-reference events is rejected. The Engine is still counting
only this implemented prefix: the profile tag alone is not evidence of complete
canonical execution accounting, and no minimum-certificate eligibility claim follows.
Capture/entry, return/accumulator, live-slot, byte and controller scope composition
remain the Core-owned decisions described above.
