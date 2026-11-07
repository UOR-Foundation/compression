<!-- uorc:non-authoritative -->
# Basis attempt event agreement registration

Registered source root: Uorc.Core.RuntimeMachineActionRefinement.
Registered theorem: basis_attempt_event_agrees_with_core_action.
For every actual opcode, native basisAttemptEvent agrees with the imported
CanonicalSchedule.actionFor classification: the basis action has one attempted
reference-read event and every other action has none. This binds the event
before actual lookup, including missing-basis attempts, and adds no opcode
ordinal, signature, fee table, lookup, or full action-effect refinement.
Full source dependency proof qualification remains required independently of
any explicitly limited diagnostic computational projection.

Status: **sourceConsumerProofDependencyOpen**. The actual full unpruned source
builds at `893c9b4631eff2590f735650ee3bf0946bd2bee9fe7d8e999cb968c663186759`.
Its unchanged ReferenceSemantics.leaf_index dependency still fails kernel
compilation; no complete source consumer proof qualification is claimed.

The exact new theorem passes kernel checking against the existing diagnostic
computational dependency projection at
`a9585300781b548daf0b33b1469fe398fa5661280e13f57bc63404d6df935a96`,
with exact axioms `[propext]`. Both this law and the completion-origin law were
checked against actual Semantic source SHA256
`011f83fc1b1d026d73aac562fd9274d93cc2ed625e64cdc3474597b6c192d722`.
This diagnostic result does not replace the open full dependency proofs.

Mutation: change the actual basis comparison to storeNew. Diagnostic mutation
build `42a8e6066477b33aa0413e368b27ca8586fc2e1775e414ab015d7c8bbfcfefa8`
fails the new theorem in both basisGet and storeNew cases. The restored actual
source passes. Structured evidence with exact full and projected source closure
hashes is `tests/runtime-machine/evidence/action-refinement-kernel.json`.
