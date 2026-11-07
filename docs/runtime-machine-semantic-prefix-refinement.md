<!-- uorc:non-authoritative -->
# Native prefix agreement proof registration

Registered source root: Uorc.Core.RuntimeMachineSemanticPrefixRefinement.
Registered theorem roots: reference_role_events_agree, instruction_prefix_agrees,
instruction_prefix_slots_succeed. Scenarios: every actual OperandRole agrees
with CanonicalSchedule.operandEvents; for every actual Opcode the five actual
prefixEvent positions 0 through 4 produce exactly instructionPrefix, and none
returns an error. These are general theorems by constructor cases, not sampled
execution. Completion at position 5 and invalid positions are separate controls.
The proof module adds only non-executable event-list meaning, with no new fee,
operand specification, execution, or event table. Error-to-empty conversion is
used only at the five positions independently proved successful.

All seven source declarations compiled through official LexLean 0.4.0 and
Lean 4.32.1 in the 19-module diagnostic closure
`ed3275608d19fd9930ab7684e33d0268d2164ae7bc4ce103b05dbbffb36fe9ae`.
Each exact axiom policy is `[propext, Quot.sound]`. Portable JSON evidence at
`tests/runtime-machine/evidence/semantic-prefix-kernel.json` records all source
closure hashes, final proof source and generated Lean hashes, and policies.

The three relevant actual main CanonicalSchedule declarations (`operandEvents`,
`operandPrefix`, `instructionPrefix`) compare exactly as parsed JSON declarations
to commit `06d5e1c9229bab88e3238670a49c99955d92f452`. Its existing official
migrated computational projection supplies these same bodies. No claim about
all current Schedule declarations or its complete proof profile is made.
The actual Semantic source is byte-identical to main after official formatting.

Planted defect: actual `referenceRoleEvent` omitted the block-reference event.
The compiler generated mutation build
`2ea8787f485788bc445ad283eb72d69aa9ff1d3a549dcae7a5f0dd3b88d890a2`,
but Lean rejected both new agreement theorems: the role case lost its event,
and apply/tabulate/fold/scan lost their block-reference events. Restoring the
actual Semantic source and rerunning the full closure produced exit 0 and the
exact policies above. This establishes mutation sensitivity of the new proofs.

These theorems establish the actual native prefix function's source schedule
agreement. Engine consumption, runtime admissibility, cache/reference equality,
store witnesses, and the full locked SDK acceptance boundary remain separate.
