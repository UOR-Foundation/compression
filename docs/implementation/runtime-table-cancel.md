# RuntimeTable final-control cancellation fix

An independent Core counterexample confirmed a real cancellation defect. In the
old whole module `38162eb828d933561668ecab43b34a512b58a23259cd588a203cc2799e4abfbc`,
a completed path's final control release could already be active while the
cursor still owned the new table subtree. Cancelling at that prefix caused the
next step to return `cancelled` with no cursor while 65 table entries and their
3648-byte owned result remained live. This was established independently of the
Wire controller's separately requested failing trace.

The fixed source withdraws that **untouched final control-root release before
processing it**, restores the existing control ticket, then cleans up the table
subtree before releasing the control reservation last. It does not free the
1072-byte control allocation while a fixed64 cursor remains live.

## Source and API

| Source | SHA-256 |
| --- | --- |
| `RuntimeTable.lex.tex` | `7c7e77f1f4048a2e6f80969221950351279a37858dd9fa89a203a218f3d1d05b` |
| `RuntimeTableCancelTests.lex.tex` | `ad55b85219b2a72cc2951543fea3cf11061217e0fe2a741ad53cff58f3b4cdea` |

`cancel`, `ownedRoots`, `beginPut` and `beginErase` remain byte-identical. The
cursor layout and 1072-byte control charge are unchanged. During final control
release only, the dead descent field `PutCursor.borrowed` records the exact
**nonowning expected control ID**. Starting that release stores the ID. The
withdrawal path checks rootStep, count one, a singleton pending list, and handle
ID equality before calling `Heap.withdrawRootRelease`; it checks the returned
ID too. Identity or protocol failure preserves the original arena, cursor and
cumulative Delta/peak. A successful withdrawal adds one work unit and no entry
allocation/release. Every step performs at most one Heap operation.

The additive pure query APIs are:

- `outcomeRoot(outcome : PutOutcome) : Handle`
- `outcomeRoots(outcome : PutOutcome) : List Handle`, bounded to zero or one entry

They expose a completion root already transferred to the caller, without
retaining it or minting a ticket. Null roots and noncompletion outcomes produce
no ticket. `ownedRoots(state)` remains the cursor's temporary-ticket inventory;
it intentionally excludes the completed outcome's caller-owned root.

A call to `cancel` after completion still reports typed `protocolFailure`.
The caller already owns the prior completed table and must keep/take that result
before another operation overwrites its outcome. This fix does not silently
roll back or release an already-transferred result. The Wire controller still
needs correct completed-outcome custody.

## Audited trace

The simple empty-slot-table control produces this actual native Lean/C trace:

| Stage | Owned bytes | Live entries | Control ticket | Table subtree | Work | Peak |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| final-control-release prefix | 4720 | 66 | transferred to Heap release | 66 | 133 | 4720 |
| cancel request | 4720 | 66 | transferred | 66 | 133 | 4720 |
| withdrawal | 4720 | 66 | restored ID 1 | 66 | 134 | 4720 |
| cancellation finished | 0 | 0 | none | none | 202 | 4720 |

Root inventory and actual control-reservation checks pass at every stage.
Allocated and released bytes both end at 4720. `nextHandle` remains 67: cleanup
preserves achieved accounting/peak and does not roll IDs back. See
`cancellation-trace.txt` for all values.

## Qualification and negative controls

The new source-owned diagnostic suite audits roots at every transition and
independently checks the live control entry's kind, promised length 1024, and
weight 1072 for every cursor state. It covers late/early cancellation, normal
outcome ownership, terminal protocol custody, exact wrong-ID failure-state
preservation, and withdrawal/cumulative accounting.

- The **unchanged old whole Core module** compiled against the final new tests;
  no outcome-query helpers were added to its baseline. The tests use independent
  local outcome-pattern inventory. Its gate fails at runtime, not source linking
  or compilation.
- Fixed source and tests pass official source linking, pinned Lean kernel and VM
  checks, and actual native Lean/C execution. The existing
  `RuntimeTableCursorTests` suite also passes. Only the affected table suites
  were run; no old nine-suite rerun or SDK build was performed.
- A planted alternative that retains the cursor after freeing its control
  reservation has `root audit=true` but `live control reservation=false`.
  The new gate rejects it. This demonstrates why root auditing alone would miss
  an uncharged continuation. The unmodified fixed native gate passes again
  after that mutation. Mutants were isolated and never changed shared source.
- Actual axiom sets are `[propext, Quot.sound]` for `step`, `cancel`, and the new
  suite; both outcome query helpers have `[]`.

Exact language-1.2 diagnostic project/configuration and lock/workspace files are
included in `diagnostic-project`. Its recorded explicit input selection was the
new cancellation suite and existing cursor suite, rather than the configuration's
legacy default entrypoint. `qualified-dependency-project` records the original
context of reused, previously qualified Heap/C objects. The public product's
language-1.1/old-SDK lock was not changed. These are scoped diagnostic contexts,
not a product migration or release configuration.

The authoritative fix and tests are source-owned LexLean; native runners contain
only assertions/tracing around official generated implementation. Raw generated
compiler artifacts remain private. This checkpoint does not claim accepted
certificates/export, whole-product peak refinement, or resolution of the separate
Wire controller outcome-custody obligation. Exact source/build/native/config
hashes and receipts are in `receipts.json`.

A final complete policy trace corrected four test-only axiom declarations to their actual empty sets. All24 test policies now match exactly. All7 generated Lean modules are byte-identical to the already kernel/native-qualified d165 checkpoint. Its native execution is reused through that equality; no new native run after the metadata-only rebind is claimed. The old native test-source identity e8dde9ed70784a7770354483003759b169f30e398f0d3f0f09b34aac319047a7 remains recorded separately from the published metadata-corrected identity.
