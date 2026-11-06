# Source type repairs and minimal conversion-check request

This is unqualified source progress after the retained compiler's actual
8f39fcd source checks, published in `7ce8a3811f44bf0088c5fbc13181942d67119d13`,
`docs/verification/enumeration-8f39fcd-source-checks.json`.
Neither natural root reached kernel checking.

## Exact observed and same-class changes

The Model check rejected `modelUnsignedByte`: checked_convert requires an
integer input, but modelByte returns Nat. The owned closure rejected primitive
scalar equal on Phase in ticketMatches. Two matching cancel comparisons are
source-audit findings under that same rule, not additional observed errors.

The producer replaces exactly those three boolean predicate nodes with complete
matches over all 18 nullary Phase constructors. True is returned only at the
original waiting/cancelled/drain target respectively. No other source AST,
argument, field, ownership transition or prior binder repair changes.
`phase_comparison_equivalence.py` records exact byte/AST deltas, 54 constructor
predicate cases, 144 ticket-custody cases, 288 cancel cases, 21 actual source-IR
mutants and five gate-omission mutants, with restoration and optimization/filter
rejection. This is focused structural/semantic evidence, not official typing or
a general Arena proof.

## Supported general conversion, no input restriction

The exact retained compiler source 71defb4 excludes Nat from checked_convert
and to_int. It does not expose Int.ofNat/Int.negSucc constructors or arbitrary
Lean builtin calls. No new primitive or assumed cast is used.

The new pure module `Uorc.Search.UnsignedByteConversion` uses only the existing
UInt8 checked_add, Option matching and structural Nat recursion:

- nextByte(value) checks value + UInt8 1; success returns its actual byte and
  overflow returns UInt8 0.
- fromNat(0) is UInt8 0; fromNat(succ n) is nextByte(fromNat(n)).
- modelUnsignedByte(payload) calls fromNat(modelByte(payload)). Its public
  argument remains every Nat. Existing pointwise modulo normalization bounds
  the helper argument by intrinsic byte width, not a product/input/enumeration
  limit. Every existing word, zero, length, inverse and rank statement stays.

The minimal helper is non-production and imports no Core or other Uorc module.
Its intended full-Nat law, toNat(fromNat(n)) = n % 256, is a new outstanding
proof obligation, not an axiom or an accepted fact. All 63 existing statement,
parameter, axiom-policy and proof ASTs remain unchanged; the wrapper definition
and its new import are the only Model changes. Broader finite reruns have not
been substituted for source admission.

## First retained-executor request

First source-check the exact tiny helper, at unchanged stock settings:

```
lexlean check src/Uorc/Search/UnsignedByteConversion.lex.tex
```

Record source/configuration/tool hashes and the actual result. Do not bootstrap,
change the language/locks, or assume an operation just because Lean has it.
After admission, check both complete original roots (now Model closure 2 and
OwnedProofs closure 16), then retain their full build/verify obligations.

The subsequent genuine proof work is explicit:

1. An all-UInt8 one-step law for nextByte.
2. The pure Nat normalization identity for successor modulo 256.
3. Full-Nat structural induction for fromNat, using those checked laws.
4. The unchanged public scalar/list inverse and rank/coverage obligations,
   including false controls and observed kernel/axiom audits.

The retained linear_arithmetic bridge list does not unfold native checkedAdd
or checkedFromInt. Its definitions list admits prior functions, not arbitrary
runtime names or induction hypotheses. simplify admits prior theorem rules and
local Prop hypotheses. Therefore a checked-add successor proof is an honest
remaining strategy question; the existing arithmetic programs are not assumed
to solve the new recursive helper. No theorem/domain is weakened to fit it.

`owned-enumeration-type-repair-progress.json` binds every changed source, both
complete import closures, exact preserved theorem contracts, checker-rule/source
references and the focused phase evidence. Historical source-format/runtime/
current-Core/proof/Stream receipts remain tied to their original checkpoints.
There is no new conversion-runtime, official helper, whole-Core, native,
kernel, canonical-membership or SDK pass in this checkpoint. Full-project
verification, exact-SDK just vv and the direct pinned-sequence closed-trial
handoff remain mandatory and open.
