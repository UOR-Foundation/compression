# Reference leaf proof: exact diagnostic and bounded repair analysis

Status: source-only diagnosis and experiment packet. No working source-proof
repair or compiler enhancement is claimed. No production declarations changed.

## Evidence and source identity

The frozen module at `ad70ad9a32bff5cca1a039a60794ac5eb0e7c079` has SHA-256
`c42518e8849de7e931183ef0777ad2e4651d052a4f0aaf916a830c035b6b252b`.
It is byte-identical in the frozen composed graph `8198d5d4fbf84fa0bd4944fad0313c099af25e5d`.
The complete theorem's canonical JSON SHA-256 is
`0959e651ebe42087d0eedc7c21c17ef010a507aea2c9b14d3d29b157c6c6241f`;
its unchanged statement alone is
`bd0b0423cf86bf0f0e81741352794772a00cb441197c9d6454e6cd11f0ef33b8`.

The existing compiler owner's [failure receipt](https://github.com/UOR-Foundation/compression/blob/0fb8f4952e73ccd46c8eaeec71222c8490f456bf/tests/canonical-composition/reference-leaf-failure.json)
binds the official migration, exact command, compiler binary, original generated
module and kernel failure. Its [eight actual contexts](https://github.com/UOR-Foundation/compression/blob/0fb8f4952e73ccd46c8eaeec71222c8490f456bf/tests/canonical-composition/reference-leaf-eight-contexts.txt)
were obtained with `-Dtrace.omega=true`, leaving the generated theorem and proof
unchanged. No compiler-extension requirement follows merely from the failure.

## All eight goals

Let `q` be the unchanged `LexLeanRuntime.index backing index`, and let `event q`
be `failed invalidCursor` for `none`, or `byteDone selectedByte` for `some`.
The fixed tactic splits the observer's WorkEvent match first and the independent
reader's Option match second. It does not subsequently rewrite the resulting
lookup equation into the event equation.

1. `event q = advanced nextWork`, `q = none`: rewriting gives constructor disjointness.
2. `event q = advanced nextWork`, `q = some b`: same disjointness.
3. `event q = sequenceDone rope`, `q = none`: disjointness, and the negated goal is already `not (none = none)`.
4. `event q = sequenceDone rope`, `q = some b`: disjointness.
5. `event q = byteDone b`, `q = none`: disjointness.
6. `event q = byteDone b`, `q = some selected`: byteDone injectivity identifies the bytes; both observations become the same `some (wordToNat (byteToNatural selected))`.
7. `event q = failed error`, `q = none`: the negated goal is already `not (none = none)`.
8. `event q = failed error`, `q = some b`: disjointness.

Thus the obstacle is constructor/equality simplification after splitting, not
arithmetic, UInt8 conversion disagreement, a missing premise or resource limits.

## Exact SDK71 references

All references below use LexLean commit `71defb4c663aaec5b9a3aab04cbc6a03e7106227`.

- [Backend semantic.rs, lines 1432–1472](https://github.com/UOR-Foundation/LexLean/blob/71defb4c663aaec5b9a3aab04cbc6a03e7106227/crates/lexlean/src/backend/semantic.rs#L1432-L1472): the fixed PF-21 script introduces/substitutes, simplifies definitions and arithmetic bridges, simplifies fixed-width projections, splits remaining if/match goals, then invokes omega. There is no post-split simplifier.
- [Semantic checker, lines 8287–8307](https://github.com/UOR-Foundation/LexLean/blob/71defb4c663aaec5b9a3aab04cbc6a03e7106227/crates/lexlean/src/ir/semantic.rs#L8287-L8307): PF-21 definitions must be prior document definitions. Arbitrary tactics and imported Lean lemmas cannot be inserted as definitions.
- [Proof schema, lines 689–756](https://github.com/UOR-Foundation/LexLean/blob/71defb4c663aaec5b9a3aab04cbc6a03e7106227/crates/lexlean/src/ir/semantic.rs#L689-L756): the closed semantic proof IR has no general tactic-sequencing or post-split continuation form.
- [term_cases checker, lines 8503–8554](https://github.com/UOR-Foundation/LexLean/blob/71defb4c663aaec5b9a3aab04cbc6a03e7106227/crates/lexlean/src/ir/semantic.rs#L8503-L8554): the scrutinee must have a document inductive type. Builtin Option is rejected.
- [cases checker, lines 8715–8805](https://github.com/UOR-Foundation/LexLean/blob/71defb4c663aaec5b9a3aab04cbc6a03e7106227/crates/lexlean/src/ir/semantic.rs#L8715-L8805): supported carriers are Nat, List, Bool and document inductives, not builtin Option.
- [SPEC.md line 2774](https://github.com/UOR-Foundation/LexLean/blob/71defb4c663aaec5b9a3aab04cbc6a03e7106227/SPEC.md#L2774): the exact current PF-21 script, including its ordering, is normative.
- [PF-21 and SM-35 criteria](https://github.com/UOR-Foundation/LexLean/blob/71defb4c663aaec5b9a3aab04cbc6a03e7106227/SPEC.md#L10674-L10715): PF-21 covers integer kernels with split match/if goals. SM-35 explicitly covers document-inductive term cases.

## Local route assessment and narrow recommendation

Existing source mechanisms were inspected: reflexivity, simplification by prior
document lemmas, congruence, supported cases/induction, term_cases, exact theorem
application, and the registered fold laws. No supported narrow source-only
composition was found that rewrites the eight new constructor equations after
PF-21's final split while retaining this universal statement. This is a bounded
inspection result, not a theorem that no source encoding can exist.

General helper lemmas conditional on `q = none` or `q = some b` do not themselves
supply exhaustive elimination of an arbitrary builtin Option. Replacing Option,
adding representation premises, restricting inputs, discarding cases, migrating
the entire production module into a different representation, or weakening the
statement is not an acceptable repair.

The first narrowly scoped experiment is the generated-only patch in
`tests/reference-leaf-proof/patch-generated-diagnostic.py`: insert after the
existing split a `simp_all only` over the local equations, Option.some
injectivity and WorkEvent.byteDone injectivity, leaving omega afterward. This
has not been executed. The positive and two false-conclusion controls must be
compiled by the existing owner before concluding that this tactic suffices.

If it suffices, a separately reviewed PF-21 post-split simplification enhancement
would keep the production theorem and source proof unchanged, but **would change
the normative language-1.2 proof strategy and compiler semantics**. It is not
already promised general constructor-fusion coverage. It would need its own
upstream source, specification, conformance tests, negative controls, bootstrap
and official SDK qualification. No such upstream action or SDK pin is made here.

A broader SM-35 change admitting builtin Option as a term_cases carrier is a
different public proof-language extension, not a prerequisite inferred by this
packet and not the preferred first experiment. Neither upstream change is
assigned or authorized by this diagnostic.
