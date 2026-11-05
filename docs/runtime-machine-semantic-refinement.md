# Semantic refinement proof increment

Registered source root: `Uorc.Core.RuntimeMachineSemanticRefinement`.
Registered theorem: `commit_preserves_canonical_failed_prefix`.
Scenario: for arbitrary budget, ledger and event, evaluate the actual adapter
commit; every error outcome retains exactly the original canonical Usage.
The acceptance obligation is the kernel proof by cases on actual chargeDecision.
A mutation replacing the rejected ledger with a different ledger must invalidate
that proof. This is a diagnostic proof obligation, not locked SDK acceptance.

The theorem is above actual `RuntimeMachineSemantic` and `RuntimeMachineEngine`.
It projects the actual fixed-width ledger into canonical Usage; it adds no
execution, event schedule, heap denotation, or canonical-refusal claim.
The unused CanonicalRuntimeRelation import was removed: cache-read transfer to
that relation remains open and is not part of this theorem.

Official compiler 0.4.0 built the 24-module diagnostic closure at
`c1d6ed225e6954b5764e611e7bb2f1575abb329521f0684d0ce5b07abb2d6ff4`.
The new generated Lean source is byte-identical to the kernel-checked pre-format
build `ca6ae5815e8c4865f3f8b5712819a36bbe604a93c288f45db201388b867e50b4`:
SHA256 `bd598a34e3d285fb08bf2c8a342e9e2bf7bc4df7e92f504aa4f06fe7e1bc69b4`.
Lean 4.32.1 `#print axioms` reports exactly `[propext, Quot.sound]`.
The private projection uses existing computational dependency projections and
the previously generated official portable runtime, not a full SDK release lane.

Planted defect: reset the ledger to `zeroLedger` on the actual commit rejection
branch. Official build `f8ca915fead4eadcf3d022fc833c4c72360b1495045085786f62e4377607af95`
generated successfully, but kernel compilation failed in the existing Semantic
prefix theorem with the unsolved rejected-case goal `zeroLedger = ledger`.
This demonstrates the dependency kernel gate detects prefix corruption; it does
not separately establish mutation sensitivity of the new Usage projection proof.
The exact original Semantic source was restored before the final build.

Migrating the currently raw data-form CanonicalRuntimeRelation together with
already authored language-1.2 Refinement dependencies remains unsupported by the
available whole-project migration route: language 1.1 rejects notation modules
(`LLC0103`), and language 1.2 rejects semanticdata (`LLP2003`). No substitute
relation, parser, or dependency stub was introduced.
