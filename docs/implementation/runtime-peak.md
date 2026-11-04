<!-- uorc:non-authoritative -->

# Logical owned-byte event peaks

For the source versions below, Sequence and ByteBuilder expose no interior logical-byte peak above their event endpoints. Their operations are increasing, constant, decreasing, or reservation replacement with a valley. A peak field in their transitions or Heap.Delta is therefore unnecessary. This finding concerns `Heap.totalOwnedBytes`, not physical RSS, allocator/index storage, generated temporaries or retained arena history.

## Consumer API

`RuntimePeak.beginEvent(arena)` returns Heap.totalOwnedBytes, preserving its typed errors. `completeEvent(previousPeak,beforeBytes,afterArena)` returns the maximum of the historical peak, saved pre-event charge and returned arena's charge. `failedEvent(previousPeak,beforeBytes)` retains the maximum of history and pre-event charge. The helpers neither modify the arena nor compare against its quota; a historical peak is an observation, not newly owned bytes or a budget. Comparison uses the official unsigned UInt64 order and performs no wrapping addition.

Call beginEvent on the actual current arena, execute exactly one exposed begin/step/cancel, then fold its successful returned arena before any further ownership operation. On error preserve the prior arena/cursor and fold failedEvent. Do not replace a failed totalOwnedBytes calculation with zero. Include starts and finishes: sampling only final successful outputs misses reservations and earlier allocation prefixes.

The endpoint rule applies at these event boundaries. An outer wrapper that calls an increasing event and then releases owners before returning must fold the intermediate event peak itself, including on later failure. It must also fold RuntimeTable.peakOwnedBytes where applicable. This document does not justify taking only the outer wrapper's start/final maximum.

## Charge and release bounds

Write O for arena.ownedBytes, n for pendingCount and B for totalOwnedBytes:

- rootStep=true: B=O; its one transferred root ticket has no extra charge.
- rootStep=false: B=O+16*n.

The domain is valid source-created or independently audited arena/cursor lineage: pendingCount matches pending references; rootStep has exactly one pending root; entry weights match the node accounting; counts/children are valid; and checked arithmetic succeeds. A malformed manually forged Arena need not satisfy the following bounds.

Let w be a removed entry's weight and k its nonzero child-reference occurrences, counting a repeated child twice. Source nodes have k<=2 and w>=40. The independent HeapAudit reference list has arity<=2 and its referenceWeight is at least40; it also funds16 bytes for each raw reference, even before null filtering.

| One releaseStep | Change in B |
| --- | --- |
| Root reference survives | 0 |
| Edge reference survives | -16 |
| Initial root entry is deleted | -w+16*k <= 0 |
| Edge entry is deleted | -w+16*(k-1) <= 0 |

A deletion removes the entry before scheduling children. Any natural removal/pop/enqueue phases form a valley followed by an endpoint no larger than the start. Child scheduling needs no extra peak allowance. Heap.release itself only transfers a root into the uncharged initial cursor; withdrawRootRelease transfers that same ticket back and is byte-constant. Retain/dropRetainedRoot modify counts without reclaiming entries and are byte-constant.

## Actual Sequence paths

| Event/path | Ownership operation | Byte shape |
| --- | --- | --- |
| All four begin functions | One metadata capture allocation | Increase |
| step/allocatePlan | At most one allocation, or pure plan completion | Increase/constant |
| step/attachPlan | Fused allocateMoved of one metadata node | Increase |
| finishSequence; indexWalk span retain | Retain an existing root | Constant |
| Cleanup/cancel starts | One release | Constant |
| Normal/cancel drains | One releaseStep | Nonincreasing |
| cancelDrainExisting | One drain, then pure prepareCancel | Nonincreasing |
| Planning, lookup/index, dispatch, terminal transfer | No byte ownership operation | Constant |

Current allocateMoved preflights and adjusts child/root counts before its final entry insertion. Those count-only intermediate arenas keep O and the pending charge unchanged. No allocation is followed by reclamation or a fallible cleanup in these Sequence paths. A failed event therefore has peak equal to its start, even if an earlier accepted event left a fresh ticket awaiting attachment.

## Actual ByteBuilder paths

| Event/path | Ownership operation | Byte shape |
| --- | --- | --- |
| begin | allocateControl(904): 48+904 modeled bytes | Increase |
| reserveSingleton; occupied-bin carryStep | One image reservation | Increase |
| commitSingleton; commitMerge | Preflight/read; commitImage | Valley, equal endpoints |
| makeLeaf | One fused allocateMoved | Increase |
| startConcat | One Sequence.beginConcat | Increase |
| concatStep | One Sequence.step, then fixed cursor rewrites | Inherits that event's shape |
| releaseLeft/right; finishControl; cancelStep/cancelDrain | One release or releaseStep per selected branch | Constant/nonincreasing |
| cancel at untouched control-root release | withdrawRootRelease, then pure startCancel | Constant |
| beginAppendOwned/FinishOwned; bin scans/transfers; terminals | Fixed owner/control rewrites | Constant |

Image commitment removes its unique reservation of weight W=48+length, then allocates a fresh image of that same weight: B follows S -> S-W -> S. Failure before allocation or from that lower arena cannot exceed S. Builder preflight occurs before constructing/merging bytes, and the reservation is already charged while inputs remain live. This is a logical reservation guarantee, not a physical buffer-allocation measurement.

Builder concatStep performs no Heap operation after nested Sequence.step. Pending/sequence outcomes forward its returned arena; unexpected byte/span/cancelled outcomes arise only from pure terminal Sequence transfer, so those error branches cannot hide an allocated byte prefix. No Builder event allocates and then reclaims entries in the same call.

## Evidence and change boundary

This is a source-path walkthrough plus general arithmetic/Node facts, not a universally quantified Sequence.step/Builder.step refinement theorem. RuntimePeakProofs proves conditional Nat token-charge and commit-valley bounds, independent reference arity/weight facts, and maximum-fold facts supported by the official proof surface. The source-path/valid-lineage hypotheses are essential. Internal Store helpers called outside their exposed event protocol are outside this contract, including forged overflowing-work cursors.

Inspected source revision: d5cdcb8. Source hashes:

- Heap: ee41c3fb11c1f60fc767448760e8020d934c253fc31eec2146413ec1a98b0e1b.
- RuntimeSequence: c352ecaf1eb161df6df7c6f0fed7fc7868d00e4fc9fe3590271d40604785a29e.
- RuntimeByteBuilder: c7f67542e9f7a4f05f6161fe5fc23b26cf85769c3538d89f0e69f35990eac00d.

Future fusion that allocates before reclaiming can invalidate endpoint sufficiency. Such an operation must accumulate max(totalOwnedBytes) at each internal ownership boundary and expose that peak on success and failure, or publish each prefix as a separate event. An event-specific peak envelope is sufficient; a broad Delta/layout change is not automatically necessary. Requalify this inventory when the named implementations change.


The two added modules passed canonical formatting, official source linking/build, and pinned Lean kernel compilation. Their 24 general theorems include the Nat charge/maximum facts, independent Node reference bounds, and actual UInt64 maxWord/failedEvent bounds. All 30 declaration axiom sets were traced exactly: three empty, four propext only, and 23 Quot.sound plus propext; no custom axiom, Classical.choice or sorry was introduced. The helper is source-owned; proof-only natMax is not a product fallback.

The three new production roots also passed official SDK eligibility and lowering against these exact sources. beginEvent reaches two functions, completeEvent four and failedEvent three; the runtime closures contain only RuntimePeak and Heap definitions, with RuntimeValue types. HeapAudit and RuntimePeakProofs are excluded from all three closures. [The scoped receipt](runtime-peak-production-closure.json) records the SDK and source identities. This is lowering evidence, with no certificate export or full-product peak refinement claim.

Focused new-API execution checked all three maximum operands, UInt64 maximum values, error propagation, and an actual release: an uncharged root cursor has total 128, then deleting its 48-byte two-child parent leaves ownedBytes 80 plus 32 pending-edge bytes, total 112. The reported event peak stays 128. A deliberately malformed overflowing pendingCount was used only to verify typed metric-error propagation.

A planted isolated completeEvent defect omitted beforeBytes. It still linked and compiled, then the actual release control rejected the underreported 112 with start peak missing (exit 1). Exact control restoration passed (exit 0). New source hashes: RuntimePeak: 79bec845c3a9dac4618ac51efccadddeb0040c35b37676058ec29904e8562448; RuntimePeakProofs: 1e5b22cf28aebaa50714deddae0cc27b0fc1d6c7a0c24aa11d0531711dc38d9c. No existing API/layout or Builder 904-byte capacity changed; the completed nine-suite/577-check qualification was not repeated. beginEvent/completeEvent admit the SDK's conservative allocation effect for the recursive Arena carrier; neither calls a Heap allocator.

RuntimePeakTests now registers six source-owned scenarios in scenarioNames and executes every one through its suite. They cover the actual two-child release, final charge growth, historical/failure folds, unsigned maximum and both typed overflow paths. The release scenario checks independent root inventories before release, after root transfer and after parent deletion. Canonical SDK roundtrip, source check/build, generated kernel compilation and all 13 exact declaration axiom policies pass. The same isolated omission makes this source suite return false/exit 1; exact restoration returns true/exit 0. [The scenario receipt](runtime-peak-tests-evidence.json) is separate from the original native composition and does not qualify physical memory or full-operation refinement.
