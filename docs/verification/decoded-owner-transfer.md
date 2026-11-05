# Pre-clean decoded owner transfer

The closed controller now composes exact Product `MachineBinding` source at
`8fff69085cd89e15285df26af5508e33dbc5e4c3` with Machine Engine/Semantic at
`7cf135da579abb7f3012184ccc9c253496259f99`. `CanonicalCost` is the exact shared
`29d25233a2565157dc0bd39acc497f620c567788` source; RuntimePeak is exact 7cf source.
No contributor branch, product lock or compiler identity is changed.

Production trial entry is explicit:

```
beginWithSemantic(run, operationalBudget, semanticBudget, semanticLedger,
                  arena, usage, archiveTicket, basis) -> Result State Failure
beginWithCallerSemantic(run, operationalBudget, semanticBudget, semanticLedger,
                        arena, usage, archiveTicket, basis, protectedCaller)
                        -> Result State Failure
advance(state, admissionCredits) -> Transition
moveDecoded(state) -> DecodedMovement
```

The two legacy entry helpers do not invent a semantic session. They remain usable
for diagnostic framing/raw dispatch, but cannot start the migrated admitted
machine without explicit semantic custody. `beginWithCallerSemantic` consumes
exactly one candidate ticket, with protected input/search/checkpoint/certificate
roles retaining their actual incoming tickets. Identical handles remain distinct
ticket occurrences. Its prepaid joint audit checks the actual Heap inventory.

`decodedAvailable` is a phase hint for the first objects state. Call `moveDecoded`
before asking the old ObjectDemux event consumer to advance. The authoritative
move checks actual unconsumed demux controls and empty scratch roots, the exact
length-table root in caller.verification, source-reader correspondence, achieved
Heap release state, cancellation, failure, semantic refusal and receipt barriers.
It neither allocates nor copies payload. A retained result carries the complete
unchanged State and a typed DecodedRefusal.

A successful `DecodedMovement.moved(state, session)` moves the sole Arena and
achieved Usage. `session` contains:

- The owned decoded source sequence ticket, `start` and `length`. Raw archives
  return their body range under the archive sequence; program output returns its
  actual output sequence and range beginning at zero.
- Actual `FrameEventContract.OwnedLengths`, including ordered empty objects.
  Its table ticket moves from caller.verification, which is cleared in the
  returned caller inventory. It is not duplicated by a descriptor alias.
- Remaining program and caller owners, protected-role classification, basis
  descriptor and any distinct basis ticket. `protected` describes the subset of
  caller tickets to return; it is not a second inventory to concatenate.
- The actual admission credit ledger, machine receipt field and owned semantic
  Budget/Ledger, plus format/profile/scope/representation metadata.

The returned State is handed off and contains no Arena, source owner, length
owner, program, caller/protected tickets, basis ticket, receipt or semantic
session. A second move cannot succeed. No stale Arena is detached or recreated.
Cleanup must not run before this handoff when Product needs decoded owners.
Product owns charged range normalization, TargetDescriptor, complete ordered
length/byte comparison and final cleanup. Decoded custody alone is not a
membership verdict; source EOF, target EOF and full comparison remain required.

The migrated machine consumes an actual `ReadyTransfer`, starts with its exact
semantic Budget/Ledger and returns that achieved transfer. Typed semantic
suspension is retained; work extension delegates to Product while it owns the
machine. Wire-owned scalar observations use Core's actual commit operation;
refusal preserves the accepted prefix and blocks further advance until funding
or cancellation. No semantic ledger is reconstructed from operational Usage.
Automatic complete output/basis/comparison/EOF event coverage is still an
integration obligation and is not claimed by this custody increment.

UC-WIRE-CLOSED-SEMANTIC-01 through -04 and UC-WIRE-DECODED-MOVE-01 through -05 have
source-owned typed runtime owners. Initial source failure caught the obsolete
Ready-to-ReadyTransfer call. Removing only moveDecoded from the disposable
projection rejects the new owner with LLT4001 missing function. New native
controls are being completed; the first semantic run exposed an empty fixture,
then a cancellation cleanup fuel failure caused by the fixture operational
budget disagreeing with its charged Arena budget. Both fixture errors are
corrected; current native controls remain pending. These observations are
recorded as failures, not passing acceptance. The older 1da29c8 native receipts remain bound to their older
Core/Product closure. No locked-SDK, whole-product, export, general refinement or
physical-memory acceptance is asserted.

The shared image-access boundary now supplies validated ordinary image spans and
uses the same byte accessor as Integrity. Initial archive layout, preserved
reachable-node contents and whole-stream EOF correspondence remain consumer
proof obligations for Integrity's completed-view publication theorem.
