# Second generated-only experiment: eliminate the five actual contradictions

Status: proposed diagnostic, not executed by its author. No production source,
compiler source, theorem statement/domain, runtime semantics, guard, axiom
policy, resource setting or SDK pin is changed. The original packet and
`patch-generated-diagnostic.py` remain byte-exact historical evidence.

## Observed failure of the first experiment

The existing Core owner [executed the first experiment](https://github.com/UOR-Foundation/compression/blob/9d995c291b9e84c31871f16baee96578271f0a52/tests/canonical-composition/reference-leaf-experiment-0e3/result.json).
The full original module failed with eight omega errors. The unchanged-statement
positive still failed with five errors. Both false conclusions failed. The
required positive/negative gate was therefore not met. No successful positive
olean existed, and the failed declaration's recovery `sorryAx` is not proof.

The [five actual residual contexts](https://github.com/UOR-Foundation/compression/blob/9d995c291b9e84c31871f16baee96578271f0a52/tests/canonical-composition/reference-leaf-experiment-0e3/positive-residual-contexts.txt)
contain, respectively, these explicit hypotheses:

1. `WorkEvent.failed invalidCursor = WorkEvent.advanced nextWork`.
2. `WorkEvent.byteDone selected = WorkEvent.advanced nextWork`.
3. `WorkEvent.byteDone selected = WorkEvent.sequenceDone rope`.
4. `WorkEvent.failed invalidCursor = WorkEvent.byteDone selectedByte`.
5. `WorkEvent.byteDone selected = WorkEvent.failed error`.

Thus the first simplification did rewrite the lookup equations and resolve
reflexivity/injectivity branches, but left constructor disjointness. These are
contradictory branch hypotheses, not semantic counterexamples to the theorem.

## One exact revision

Retain the first experiment's post-split narrow simplification and insert exactly
one additional line before the existing omega step:

    all_goals (try contradiction)

This is justified by the observed five equations, rather than a blind expansion
of the simp set. In pinned [Lean v4.32.1 Contradiction.lean](https://github.com/leanprover/lean4/blob/v4.32.1/src/Lean/Meta/Tactic/Contradiction.lean),
`MVarId.contradictionCore` has an explicit branch for equality of two constructor
applications with different constructor names. It assigns a proof produced by
`mkNoConfusion`. That branch directly covers every residual equation above.
The tactic uses its stock configuration; no search, heartbeat or resource
setting is increased.

Run the new script on the original full generated module, SHA-256
`909ffa4f488e18258b4ee25a5459bc5df1bd5d3d8df308b09313e95f723af3ed`:

    python3 tests/reference-leaf-proof/patch-generated-diagnostic-v2.py "$GENERATED_REFERENCE" --out /tmp/reference-leaf-generated-diagnostics-v2

It produces independent positive, false-none and false-zero copies, retaining
the original full theorem header and inputs in the positive. It does not consume
or overwrite the first experiment's modified files. Both false controls retain
their original counterexamples: backing `[1]`, index `0` for false-none; empty
backing, index `0` for false-zero. Continuations may be empty in both cases.

The existing Core owner should compile all three with the same pinned Lean
command, dependency oleans and stock budgets, saving exact input/log hashes and
exit codes. Require positive success and both controls to fail for the intended
false conclusion, not an unrelated syntax/dependency error. Only a successfully
compiled positive module may then be used for axiom inspection; reject any
`sorryAx` or other policy mismatch. Archive all inputs and restore originals
byte-exactly afterward.

Even a successful result would establish only that this generated diagnostic
proof works. It would not establish source-proof acceptance or authorize an
upstream compiler enhancement. A PF-21 change combining post-split equation
simplification and constructor discrimination would still change the normative
language-1.2 fixed proof script and require independent upstream qualification.
It is distinct from extending SM-35's supported term_cases carriers to builtin
Option. No such compiler or source-language edit is proposed as accepted here.
