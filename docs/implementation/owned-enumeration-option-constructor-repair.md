# Exact requiredWork Option annotations

This source-only repair starts at public
`0582704ce41d1a385c23e0ed9a986047a43e18ed`. The retained qualifier's terminal
admission report identified `OwnedEnumeration.requiredWork`: `Option.some`
lacked its explicit type argument. Pinned compiler `71defb4` confirms that
`constructor_signature` requires exactly one type argument for both Option
constructors (`crates/lexlean/src/ir/semantic.rs`, lines 7649–7667).

An exhaustive recursive source inventory found exactly 16 Option construction
terms in the producer, all direct `Option.some` bodies of `requiredWork`, whose
unchanged result is `Option UInt64`. The exact branches are 0–8, 10–15 and 17
of declaration 26. Each gains only `type_arguments: [{kind: uint64}]`. The two
`builderStepFee` branches, 9 and 16, remain unchanged. Stream contains no Option
construction terms. Match patterns are not construction expressions and are
not edited. This is one observed failure class plus exact same-class source
findings, not 16 independently observed compiler errors.

The dedicated gate is:

```
PYTHONDONTWRITEBYTECODE=1 python tests/enumeration/option_constructor_equivalence.py
```

It compares every producer AST node and source byte to exactly those 16 added
arrays. Erasing only the named arrays must recover the complete original bytes.
It also verifies all other tracked source bytes, including both full original
proof roots and their 63 complete theorem declarations, the helper, the new
byte-step proof input, and the typed Stream. No body, branch, payload, declared
result, Phase case, charge, custody transition, proof or statement changes.

The saved result is `owned-enumeration-option-constructor-evidence.json`.
The gate rejects 51 actual source-IR mutations (missing/wrong/extra annotation
at every site, changed payload, changed return type and omitted Phase branch),
rechecks the unmodified source after each in-memory mutant, and rejects real
optimized/filtered processes
and temporary gates with omitted source checking or filtered branches. Its
normalized AST seal covers the entire gate implementation, including validation
logic; this is drift/omission evidence, not a security authentication boundary.
These are structural checks, not execution of the mutated charges or custody.

## Preserved qualification and blockers

The exact earlier helper at `134db360` passed retained check/build/ordinary
verify. The newer closed UInt8 byte-step law at `0582704` passed check/build,
then `decide` failed to synthesize Decidable for its universal proposition.
No theorem axiom audit or false-control proof rejection was reached. That law
is not proved, and no alternative proof is included here.

The ordinary full Model verification reached compiler LLS8002 at its unchanged
300,000 ms child limit (300,005 ms measured); no theorem-specific error was
reported. Its earlier 60-second external diagnostic interruption is a different
event. No Model proof change follows from either observation.

Further official execution is currently unavailable. This repair does not
substitute another executor, compiler or toolchain. No new official source admission, kernel/axiom audit,
native run, broad finite rerun or SDK acceptance has been attempted. The new
producer remains explicitly unqualified. All historical receipts retain their
original source identities.

Further proof variants and Search adapter integration are frozen. Public
Product target descriptor `da31cd88f79608fcf1e3611670b0d5e5cf3f307e` is available,
but the real ordered comparator and fixed-slot cleanup/return APIs remain
private and incomplete. They must not be duplicated or replaced with flat-byte
equality, borrowed post-clean handles, an uncharged root list or a fallback.

When authorized existing execution is available again, first check this exact
annotated producer/Stream closure, retain actual next diagnostics, and qualify
the byte-step proof route before Nat induction. Product/Wire must publish the
real owned comparison/cleanup contract before the correlated direct-sequence
Search trial adapter is implemented. Full producer refinement, fair discovery,
canonical membership, native execution and locked-SDK `just vv` remain open.
