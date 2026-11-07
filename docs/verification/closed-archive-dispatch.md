<!-- uorc:non-authoritative -->
# Owned whole-archive dispatch

The production source boundary is `Uorc.Wire.ClosedArchive`. Its input is an
already charged immutable **sequence ticket**, not a recursive Graph/Rope and
not externally leased archive chunks. The controller owns the archive reader;
Search must not insert a second archive reader or a TemplateAdmission shortcut.

```text
begin(run, budget, arena, usage, archive, basis) -> Result State Failure
beginWithCaller(run, budget, arena, usage, archive, basis, caller)
  -> Result State Failure
advance(state, admissionCredits) -> Transition
pending(state) -> Option FrameEventContract.ObjectEvent
acknowledge(state, ObjectDemux.Ack) -> Transition
readSequenceStep(state, OwnedRead.Cursor) -> SequenceRead
readByte(state, image, absoluteOffset) -> ByteRead
publicationFailed(state) -> State
cancel(state) -> State
cleanup(state, admissionCredits) -> Transition
takeClean(state) -> CleanMovement
fundInputWork(state, additionalWork) -> Result State Failure
takeOwnershipRefusal(state) -> RefusalMovement
```

These signatures use the exact source `Product.Api.Run`, `RuntimeResources`
Budget/Usage, Heap Arena, RuntimeValue Handle, optional OwnedBasisSnapshot
Snapshot and RuntimeProgram CallerRoots types. Transition returns the achieved
State and remaining admission credits. Its Owner sum contains the single active
arena-bearing child. Input, parser and admission do not retain separate arenas.

`beginWithCaller` moves the arena and one archive sequence ticket into the trial.
Protected input/search/checkpoint/certificate tickets travel with it. Archive,
verification and wire are reserved vacant caller roles at entry. Any basis
descriptor is borrowed under the supplied protected roots. Convenience `begin`
instead owns its supplied standalone basis record ticket. The controller never
accepts a supplied list as proof of custody: it incrementally checks actual Heap
index entries and root multiplicity against the archive ticket, standalone basis
record when applicable, and the fixed seven caller fields. Each handle check is
prepaid, including gaps and failed prefixes. A final checked ticket total rejects
proposed future handles outside the visited range. There is no retained growing
root list. Incoming active release is refused without releasing roots.

Equal handles are counted by ticket role. For Search's two copies of one
candidate sequence, archive owns the transferred copy and protected search owns
the current copy. The initial heap must contain two roots. Successful cleanup
releases exactly the archive copy, leaving one protected current copy. It never
deduplicates those facts. Protected roots can themselves reach arbitrarily many
charged objects; the seven-role inventory is not an input/object-count cap.

`advance` performs actual ArchiveSnapshot framing and ordered length storage,
versioned ArchiveBinding verification over the exact preimage, canonical owned
basis identity verification for conditional scope, body replay through actual
ProgramAdmission, Product.MachineBinding execution, and ArchiveObjects output.
Raw representation follows the same framing, integrity and ordered output path
without fabricating a program. Empty objects retain ordered begin/end events.
The seal event carries total output bytes. Ordinary archives gain no certificate
fields and no old archive-v1/profile02 compatibility claim.

A pending output span stays rooted until acknowledgment. `consumed` requires the
sink's completed prepaid copy or full comparison. `takeSpan` transfers an extra
owned image ticket in the outcome; the consumer must inventory and eventually
release/transfer it independently. `readSequenceStep` and `readByte` permit a
comparison actor to borrow expected and decoded bytes while the controller keeps
the sole arena, returning updated Usage in State. Search must correlate candidate
identity, compare every ordered boundary and byte, observe original EOF, and only
then acknowledge consumption. `decoded` is a decode result, not archive membership
or a bounded/global-optimality certificate.

`publicationFailed` freezes the pending owner and event. Cancellation drains
actual parser/store/hash/admission/machine/output cursors. A detached producer's
active input release is drained before reattachment. Parser calls are shared as
one transition per step rather than recomputed separately for session and signal.
Core admission uses actual Ready and cancellation results; no successful token is
constructed by this controller. Admission credits, achieved Usage and machine
overflow receipts remain retained across handoffs.

`takeClean` transfers one CleanSession containing Arena, Usage, exact protected
CallerRoots, admission ledger and overflow receipt, leaving an Arena-free handedOff
state. The consumed archive ticket and trial length table are fully drained;
protected tickets are returned, not released. A repeated take is rejected. An
ownership refusal instead permits `takeOwnershipRefusal`: the actual Arena and
achieved Usage move intact, with explicitly **unvalidated** proposed-root hints
and the audit position. It releases nothing and asserts no ownership certificate.
An initial audit work pause remains pending and can resume with `fundInputWork`.
Neither pause, cleanup delay nor ownership refusal is a semantic exclusion.
Other failed stages retain their actual typed child failure for the caller's
semantic/operational classification; `failed` alone is not an exclusion witness.

The source uses operational U64 addresses/counters and reports refusal at that
boundary. It does not redefine the mathematical finite wire language. Logical
ownership and work accounting do not establish host RSS bounds or physical
reclamation of old immutable copies. No uncharged AST, signature cache, length
list or fallback parser is retained beside the arena.

Native controls and exact source/dependency hashes are in
`closed-archive-dispatch-diagnostic.json`; caller custody has its own receipt.
The checked source currently composes the earlier Integrity implementation while
using the exact shared RuntimeImage accessor. New Integrity transcript/charge and
retained-source lemmas at d8aade12 are separate imported work. Their consumer
snapshot/node-content and canonical-digest refinement obligations are not assumed
as global facts here. Full production export, physical qualification, general
refinement, model regeneration and locked-SDK `just vv` remain acceptance gates.

## Current shared dependency boundary

This checkpoint uses exact Core runtime sources from ad70ad9a and the existing
Product.MachineBinding closure from 8221f4d7. It preserves RuntimeResources Usage,
the admission-credit Ledger and machine overflow receipts for that closure.
The newer Machine interface at 7cf135da returns
`ReadyTransfer {ready, semanticBudget, semanticLedger}` from `takeReady`, and
exposes `startWithSemantic(ready, budget, semanticBudget, semanticLedger)`.
The existing Product binding expects a plain Ready. Updating that binding's
start/finishDrain/Terminal/moveTerminal and the wire terminal carriers to pass
these exact semantic values is a **pending integration obligation**, not a
verified compatibility claim. Reconstructing them from Usage counters or
resetting them at output, basis or EOF is not permitted. This checkpoint does
not claim the newer combined Machine control job passed.
