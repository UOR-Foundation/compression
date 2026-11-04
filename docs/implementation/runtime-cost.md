# Pre-call resource quotes

`RuntimeCost.lex.tex` is additive to the e87 API. Its 46 executable quote
functions can be called before a primitive; six proof-only helpers and ten
general weight/frame theorems remain outside runtime use. The complete module
passes source checks, kernel compilation and all 62 exact axiom policies.

The prior e87 source exposes `Heap.nodeWeight(node)` for pre-call owned-entry
byte admission. It does not expose dedicated logical event-work cost functions.
The new APIs distinguish the following accounts:

| Account | Source API | Scope |
| --- | --- | --- |
| Sequence event quote | `sequenceStepCost(arena,cursor)` and begin/cancel variants | Unit logical dispatch quote, available before the call. |
| Store event quote | `storeStepCost(arena,cursor)` and begin/cancel variants | Unit dispatch, distinct from event-local Heap Delta and the cursor's checked cumulative logical work. |
| Builder event quote | `builderStepCost(arena,cursor)` and begin/cancel variants | Unit dispatch; payload copying is quoted separately. |
| Heap successful operation work | `heapAllocateMovedSuccessWorkCost`, `heapCommitImageSuccessWorkCost`, other named success quotes | Moved allocation is one plus enabled move flags; image commit is two; ordinary successful ownership operations are one. |
| Owned-entry bytes | `heapAllocationOwnedBytesQuote(node)` | Delegates to the same `Heap.nodeWeight` used by allocation. |
| Reservations | `reservationOwnedBytes(length)`, `builderImageReservationQuote(cursor)` | Typed pre-admission capacity quotes, including overflow; not a physical memory estimate. |
| Builder copying | `builderStepCopyCost(cursor)` | Quotes singleton or merge payload work for the valid cursor phase. Failed validation can copy zero. |
| Table dispatch | `tableStepDispatchCost`, begin dispatch budgets | A step is one dispatch; composite begin wrappers have a three-dispatch budget. `PutTransition.delta` is cumulative and is a different account. |

Callers retain the arena/cursor and achieved failure prefix. A reservation or
begin budget is not actual failed-call usage. Table callers must compare
successive cumulative Deltas rather than charge each cumulative value again.
Image backing remains charged through reserve/copy/commit and until all old
image tickets are explicitly released.

The ten proved facts concern the real node-weight/reservation/frame/branch
definitions. **No universal theorem binding actual successful `Sequence.step`
result work to its pre-call quote has been proved.** Several attempted
structured-proof candidates were rejected and removed. The small source-only
`RuntimeCostResultProjectionRepro.lex.tex` demonstrates the fixed structured
proof lowering's late Result/match projection limitation; it is not evidence
that the proposition is false or impossible in LexLean Core proof authoring.
The quote module alone therefore does not qualify full operation-cost
refinement. Its source receipt is retained with the other implementation
evidence, and the producer's full cost contract remains a distinct proof gate.

None of these functions estimates wall time, allocator operations, Rc/index
overhead or RSS. The e87 native execution and complete locked-SDK release
acceptance remain separate evidence boundaries.
