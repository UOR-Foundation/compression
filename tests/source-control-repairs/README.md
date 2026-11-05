<!-- uorc:non-authoritative -->
# Source control repair regressions

This progress checkpoint repairs source admission and complete record equality.
The authoritative declarations remain in `src/Uorc/Search`; these files test
them. No product claim is derived from this note or from finite diagnostics.

The exact prerequisite commit is `28ae74f17a901235a68129e70e10e53e7d0e0716`.
It composes the three owned Core repairs from `95683d8`, the four source hygiene
repairs at `e1b870d`, and only the 36 binder type additions from PR #94
(`388c1e5`) onto `ad70ad9`. It does not merge unrelated Search branch changes.

## Source repairs

- TemplateAdmission declares its existing local callees before callers. All
  43 declaration values, bodies, signatures, proofs, annotations and source
  wrappers are preserved by an invertible declaration permutation.
- WaitingEquality compares complete values of 32 actual runtime types. It
  handles every constructor and every record field, including recursive Heap
  indices, dormant state, errors and outcomes. There is no validity premise,
  sampled projection, fuel bound or unchecked equality flag.
- The original State/Arena and outcome assertions in TemplateStreamControls,
  TemplateFundingControls and TemplateOwnerControls use those comparators.
  All original assertions, theorem statements, proof programs and policies
  remain present. FundingControls.startTrial uses its local Trial type name
  without an unavailable self-qualified module reference.

## Reproduction

Use an already supported, prepared language-1.2 project and the official
compiler, semantic-source helper, Lean and matching generated runtime. The
source readiness conversion must be audited through the official reader and
writer; this test does not select or install an SDK, rewrite product locks, or
provide a compiler/runtime fallback. The repository's full `just vv` in the
exact locked SDK remains the acceptance boundary.

The declaration-order regression inspects the actual source through the SDK,
checks every declaration value, and runs all four mandatory compiler gates:
original failure, repaired check, repaired build and swapped-order failure.
It restores the isolated source and confirms input sources and setup bytes
are unchanged. For an existing prepared project with these repairs:

```sh
python3 tests/source-control-repairs/test_template_admission_decl_order.py \
  --project "$PREPARED_PROJECT" --compiler "$LEXLEAN" \
  --semantic-helper "$SEMANTIC_SOURCE" --output "$ORDER_RECEIPTS"
```

`DefaultEqualityClassAudit32.lean` proves, for every pair of values of each
complete type, that its actual generated comparator returns true exactly when
the values are propositionally equal. These proofs impose no validity or
reachability condition. They use stock Lean settings, without local budget
options, `native_decide`, admissions or custom axioms. The proof prints the
actual axioms of every comparator and each equivalence theorem. All 32
comparator definitions were observed axiom-free. Some equivalence proofs use
standard Lean axioms: the Arena path uses `propext`, `Quot.sound` and
`Classical.choice` through ordinary ByteArray/Array equality. Those validation
lemmas do not change the source declarations' exact axiom policies.

The scenario fixture and runner (see `EQUALITY-RUNNER.md`) require all 17 identity,
constructor, payload and nested-field discrepancy cases; they are independent
source tests of the comparators. They do not replace the original controls.

## Observed qualification boundary

The six affected roots passed ordinary source check and build with their
actual 39-module closure. A missing binder annotation and a restored
self-qualified Trial reference were each rejected by their owning source
check; byte-exact restored source passed. The declaration-order defect was
also rejected and restored. All 17 separate VM comparator scenarios passed.
All 32 complete equivalence proofs passed, then a fresh consumer confirmed
stock defaults of recursion depth 512 and 200,000 heartbeats.

The complete original TemplateStreamControls proof still exceeds an external
2 GiB diagnostic guard at `twoImages_all_spans_released_after_ack`. The full
original control owners have therefore not acquired proof or VM acceptance.
A plain `decide` proof experiment and an independently proven equivalent
record-match factoring also reached that guard; neither change is adopted.
No original control is removed, disabled or marked ineligible in this repair.

These are diagnostics with the existing official compiler at source
`71defb4c663aaec5b9a3aab04cbc6a03e7106227`, not installed pinned-SDK,
native-export, whole-product or release acceptance. The immutable SDK,
template, policy files and verifier budgets are unchanged. Exact source
hashes and scope are recorded in `qualification-scope.json`.
