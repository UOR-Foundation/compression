# EnumerationStream Phase predicate repair

This is a separate source repair above frozen public checkpoint
`134db36039927f559b43caa28f744c8338699857`. This checkpoint contains the separately
published helper surface/axiom-policy repair above `aa4772de`; its Stream source
is identical to `aa4772de`. This change replaces nine scalar `equal`
predicates on `EnumerationStream.Phase` with inline exhaustive matches over the
same twelve nullary constructors. It does not introduce a new capability or
change the model's claims.

## Exact scope

The repaired predicates are:

- `offerMatches`: `offered`.
- `viewOutcome`: `offered`, `complete`, and `cancelled`.
- `quiescent`: `complete` and `cancelled`.
- `cancel`: `cancelled`, `drainSpan`, and `drainControl`.

All nine retain the original `cursor.phase` scrutinee. Exactly one branch returns
true in each match. Every constructor appears once, in declaration order, with
no payload or binder. The gate compares the full original parsed source IR from
Git with the patched IR, permits only these nine replacements, and checks the
canonical source bytes including the unchanged non-JSON prefix and suffix.
All declarations, signatures, numeric/handle comparisons, funding decisions,
charges, root inventories, and custody transitions otherwise remain identical.

Source SHA256 values:

- Original: `cfa09481e6d2a18e22752b91217d286913f4f5a22bb10626b20be22012c7225c`.
- Patched: `6c0c3275c0670e5ecb61d51457cee9da46a3f3e455df61f3d1d634cfb5cffd02`.

The nine sites were found by static source inspection. This document does not
turn those findings into nine compiler results. No official source check or
compiler run was performed for this patch.

## Dedicated gate

Run the new, unfiltered gate from the repository root:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -B tests/enumeration/stream_phase_comparison_equivalence.py
```

The dedicated result is
[`enumeration-stream-phase-comparison-evidence.json`](enumeration-stream-phase-comparison-evidence.json).
The earlier producer Phase receipt and the earlier Stream runtime receipts are
preserved unchanged; they are not relabeled as results for these new bytes.

Before the source edit, this command exited 1 with:

```text
ValueError: offerMatches/offered: expected Phase match
```

### Earlier interrupted attempts

Two pre-final executions were intentionally interrupted and are not product
failures or passing gate evidence:

- The first, uninstrumented gate version had SHA256
  `894198e13e9385f1d72f4d0999665662bc24bcae48ffc7401a9efd1deb62a379`.
  Its exact session returned exit 130 after about 21 minutes; stdout and stderr
  were empty. It was stopped to add stage telemetry and eliminate repeated
  scans of unrelated evaluator dispatch entries.
- The instrumented version had SHA256
  `9062e0d68e24dae98ac6e7dc6c874e2d9a09cafd0086e849e643d3a1076b4f3a`.
  It also returned exit 130 with empty stdout/stderr when a separately
  prepared helper-repair checkpoint was requested as the new baseline. Its scratch log showed
  synthetic checks completed, then the original full-path empty fixture
  completed and the `00` fixture began. No complete receipt was produced.

The final gate's implementation hash and result are recorded separately in its
JSON receipt. Scratch stage telemetry is written to
`enumeration-stream-phase-comparison-progress.jsonl` in the system temporary
directory; those stage markers are not success evidence.

### Final executed result

The final command exited 0 with empty stderr. Its stdout is preserved verbatim
in the linked JSON receipt. The final gate SHA256 is
`33d912cd638190ffd11a3a635153258587085fff0cb8c95c3d2f5b74f3bfab2f`.

The original and patched complete input/output traces match at SHA256
`d4cb7441edf2ae9664a010fa01eec837e6bb59079da28d986982f2a897aae44f`.
Each complete eight-fixture run performs 2,975 full `fund` calls and 4,027 joint
custody audits, covering 111 operation prefixes, 11 offers, 114 partial-payment
cancellation forks, 114 split-payment checks, 114 resource refusals, 125
zero-credit checks, 92 invalid-offer checks, two invalid-source checks, 233
terminal replays, and 491 funded repeated-cancel prepayment checks. All twelve
Phase constructors occur in these actual adapter paths.

All 53 intended source-IR mutants were rejected and restored. The final complete
restoration matched, all nine implementation omissions were rejected, and both
optimized/filtered child processes exited 1 at their intended guards. The final
source reread, 30-definition dispatch coherence check, and 319 frozen-input
bindings also passed. These remain finite source-diagnostic results only.

### Checked cases

The gate checks:

1. All twelve constructors at each of the nine predicates: 108 cases.
2. Synthetic cursor cases with deliberately distinct handles, offsets, lengths,
   positions, flags, and prepayment: 432 exact/altered offer correlations and
   direct acknowledgments, plus 192 cancellation, 192 outcome, and 192 quiescence
   cases. These arbitrary cursors exercise the predicate truth tables and field
   preservation; they are not asserted to be reachable producer/heap states.
3. Fresh evaluation of full `fund` paths in the original and patched modules,
   including nested calls. Installing a module updates both evaluator module
   declarations and the function dispatch table. Complete input/output traces,
   including Arena/Cursor/Usage/Delta/outcome/credit/charge fields, must match.
4. Real ByteBuilder-backed fixtures for the adapter: empty, `00`, `ff`, `0000`,
   `00ffff`, `ff00ff`, bytes 0 through 16, and seventeen repetitions of `00ff`.
   These fixtures test adapter byte delivery and custody, not end-to-end
   producer enumeration or general reachability.
5. Joint producer/Stream root-multiset audits at actual operation and cancellation
   prefixes, zero and refused payment, split advance and acknowledgment payment,
   altered offer fields, prior-span replay, partial-prepayment cancellation, repeated full `fund(Request.cancel)` with zero
   new credits from partially paid cancelling states, terminal replay, and final independent producer cleanup. Stream completion
   and cancellation must leave the producer ticket outstanding.

The gate reuses only the existing source evaluator and source-bound native
OwnedRead parser cache. It verifies their frozen inputs; it does not rerun the
official parser or overlay a new Core checkout. No new current-Core union run is
claimed. The receipt records the exact cache, parsed-reader and source hashes.

## Gate defects and restoration

The new gate plants real source-IR defects and requires their intended failures:

- At every repaired predicate: restore scalar equality, remove a branch,
  duplicate a branch, invert the true branch, and invert one false branch.
- Add a Phase constructor or payload, change the scrutinee, change the unrelated
  acknowledgment binder, or change cancellation prepayment.
- Execute without payment, omit the actual image retain, or erase the partial
  charge receipt. These must fail the actual unpaid-effect, joint-custody, and
  charge-accounting assertions, respectively.

The first fifty mutations exercise shape, truth-table, or exact-delta guards.
The three payment/custody mutations exercise complete source-evaluated paths.
Each mutation is discarded, the original patched source semantics are rerun,
and every payment/custody mutation is followed by a full restored path run.
A final restored full run must match the original patched result.

Nine implementation mutations cover constructor filtering, omitted source
mutants, removed argument/optimization guards, stale source loading, omitted
full-path execution, missing nested dispatch switching, omitted frozen input
binding, and omitted full-path restoration. The executable AST fingerprint
rejects each. Real child-process invocations using optimized Python and a filter
argument must also exit nonzero without a result. This is accidental-omission
resistance, not a security boundary against coordinated malicious edits.

## Preserved authority and remaining qualification

The receipt binds frozen source/model/evaluator/cache inputs to `134db360` and
records their SHA256 manifest. In particular, `UnsignedByteConversion`,
`OwnedEnumeration`, `OwnedEnumerationModel`, `OwnedEnumerationProofs`, and
`EnumerationFunding` remain byte-identical to this public baseline. The separately
repaired helper SHA256 is
`881306b4a7b71345e5a673f808e4165a8aa06b887748fab3742253c27013713f`;
its new gate, evidence, and documentation are also frozen inputs. All Core and Wire source files,
model records, locks, and existing implementation receipts are unchanged.

This is finite source-evaluator evidence only. It is not official elaboration,
generated-native execution, kernel admission, a general producer/Arena proof,
conversion-proof expansion, archive trial/controller integration, resource-bound
proof, release qualification, or complete exact-SDK `just vv` acceptance.
No compiler/bootstrap, Core/Wire repair, merge, tag, or release is part of this
change.
