# Pointwise owned-weight refinement

The authoritative source is `RuntimeAccountingProofs.lex.tex`, with registered
controls in `RuntimeAccountingTests.lex.tex`. These modules add no runtime
carrier, fallback, production limit or mutation to Heap behavior.

For arbitrary `rows`, `all` and `arena`, `weightsRows_reflect` proves that the
actual `HeapAudit.weightsRows rows all arena` returns true exactly when the
independent recursive `WeightsInvariant rows` holds. Each row requires its
stored weight, converted by the existing `HeapAudit.wordToNat`, to equal the
independent `HeapAudit.referenceWeight` of its node. No valid-arena, finite
sample, row uniqueness or complete-index premise narrows this statement.

The independent `sumReferenceWeights` reads node reference weights rather than
stored weights. `reference_sum_cons` gives its general recurrence.
`cons_sum_from_tail` proves one composition step for arbitrary head and tail
with explicit premises: the head weight is valid and the actual stored tail
sum already equals its independent reference sum.

The full implication `WeightsInvariant rows -> sumWeights rows =
sumReferenceWeights rows` remains unproved. Bounded source proof attempts did
not discharge the conditional induction hypothesis; no weakened theorem,
modified generated proof or additional compiler strategy is retained. The
qualified composition lemma is the available bridge, not a claim that the
recursive implication is complete.

The six source-registered controls cover empty rows, a fixed-weight store leaf,
a binary image, a mixed reference sum, rejection of a wrong stored weight,
and rejection of balanced wrong weights. The fixed-weight control also
allocates a real store leaf, looks up the achieved entry, checks its literal
40-byte weight and runs the actual row checker. The balanced defect has the
correct total sum while both entries are wrong, so a total-only check cannot
replace the pointwise invariant. Registration preceded implementation: the
official check first failed on the missing proof module.

Component source checks, affected kernel checks, all six source controls and
all 24 exact axiom policies pass under the recorded existing SDK71 diagnostic
language-1.2 configuration. The two completed defect gates are:

- Change actual `Heap.nodeWeight` for a store leaf from 40 to 41. Its runtime
  dependencies still kernel-check; the actual allocation/lookup/audit control
  fails. Exact restoration passes the kernels, controls and policies.
- Add one to the actual `HeapAudit.sumWeights` recurrence. Its runtime and
  prior audit-proof dependencies kernel-check; the owning
  `cons_sum_from_tail` theorem fails with an arithmetic countermodel. Exact
  restoration passes the kernels, controls and policies.

Earlier checker mutations were terminated during proof conversion and are
explicitly excluded from semantic defect evidence. Exact source hashes,
commands, policy observations, outcomes and restoration checks are retained
in `runtime-accounting-evidence.json`; generated modules and binaries remain
private diagnostic artifacts.

The next general prerequisites remain: complete fixed64 index enumeration
and route injectivity; the full invariant-to-reference-sum implication;
totals/cursor reflection; valid-node and acyclic representation preservation;
and preservation of the complete root/count/weight inventory by every actual
Heap operation. This component does not prove `wordToNat` is mathematical
unsigned conversion, adequate interpretation fuel, sequence/store/table or
machine refinement, or physical allocator retention. Pending release-edge
charges and external owners must still be composed explicitly. Preservation
export, native physical measurements and full locked-SDK `just vv` remain
separate acceptance stages.
