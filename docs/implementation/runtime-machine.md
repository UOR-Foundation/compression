# Owned decoder machine implementation evidence

This is non-authoritative implementation and verification evidence. The LexLean
source remains authoritative. Issue #14 and implementation acceptance remain open.

The lane starts at public integration commit
`c4f261008c614ea66cbe103f0d229c6b8819bd88`. It imports the Heap-owned source from
`d10bf90dcdf8c0f536632dc1be28d14fa37de127` without changing that contributor branch,
and the compatible Core helpers and source repairs published in
`cc987e418cbc3d39c019fc41869b34c8b1f92066`.

`RuntimeMachine.lex.tex` supplies typed instruction preparation for all 39
normalized symbolic opcodes, fixed-width checked arithmetic and bit operations,
lexical block access, typed local access, bounded capture transfer, iteration
control, and resource preflight. `RuntimeMachineEngine.lex.tex` is the composing
execution driver under qualification. A prepared internal work action is not a
successful sequence/store/block execution result.

Runtime metadata uses zero-based local and backward-block offsets, as specified
by the committed `RuntimeProgram.localByDistance` and
`RuntimeAdmissionChecks.priorBlockIndex`. The independent mathematical Graph
uses one-based distances. `RuntimeMachineTests.fixtureOperand` converts the
mathematical test inputs once using the explicit `RuntimeOpcode.spec` roles.
Production dispatch does not infer wire tags from constructor order.

Captures read immutable `BorrowedInstruction` capture tables by index. A capture
item remains borrowed from the caller's slot-root ticket until the successful
destination mutation installs its edge; `captureCommitted` then advances exactly
one bounded item. There is no copied capture/value list. Iteration control
separates observation and update. Scan's observation always transitions to an
update at the same coordinate, including its last coordinate. Empty loops have
no invocation. Two general kernel lemmas state those control properties; they
are not general machine/reference refinement proofs.

Preflight charges work before testing the output/frame event. A later refusal
retains the successfully charged work prefix. The driver composes the actual
Heap-owned cursor APIs, keeps achieved arena states and cumulative ownership
changes, and preserves Ready's caller-domain inventory. The output transfer API
is distinct from cleanup and Ready transfer. Modeled failures do not imply
rollback after fatal or unknown host outcomes.

## Scoped observations

The official LexLean 0.4.0 release binary was checked against its published
checksum (`ae0112f07e2de5bd1cca1b98f7785919de93d9e2bb41bf294df14ddeb1a472d2`).
The observations use the official generated Lean modules and Lean 4.32.1. These
are diagnostics, not the immutable SDK selected by `prismpm.lock`.

The typed runtime definitions in `RuntimeMachineTests.lex.tex` execute:

- 847 comparisons of checked arithmetic and shifts against the independent
  `Core.ReferenceSemantics`, across eleven boundary words in each operand;
- all 39 symbolic instruction dispatch cases, with actual arena-backed slots
  built by `RuntimeTable.beginPut/step`;
- scalar dispatch result comparison against independent instruction evaluation;
- byte conversion boundaries, the in-range released-slot distinction,
  failed-work prefix retention, and the mandatory final Scan update control.

All seven scoped scenario groups pass. The helper source's generated definitions
and its two general control lemmas compile in the Lean kernel. Exact axiom
traces contain only the empty set or `{Quot.sound, propext}`.

A source mutation replacing checked addition with checked subtraction still
builds and compiles, but the independent `arithmeticScenario` returns false and
the assertion process exits 1. The control source is never mutated in place.
Further mutation and end-to-end engine evidence will be recorded with their
actual outcomes.

## Remaining qualification

The complete engine scenario inventory, per-phase cancellation and cleanup,
retained-output audits, partial-allocation failures, general refinement proofs,
certified native export, physical copy/retention measurements, model projection
integration, and locked-SDK `just vv` remain separate obligations. At this
checkpoint the Heap lane's sequence cancellation API and builder cancellation
correction are awaiting publication/qualification. The machine does not claim
those missing dependencies are complete. No lock or universal policy is changed.
