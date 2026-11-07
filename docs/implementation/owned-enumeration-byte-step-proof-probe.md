<!-- uorc:non-authoritative -->
# Exact unsigned-byte step proof input

`Uorc.Search.UnsignedByteConversionLaws` is a one-theorem, two-module proof input
for the retained official qualifier. Its imported helper is unchanged from
`134db36039927f559b43caa28f744c8338699857`, SHA-256
`881306b4a7b71345e5a673f808e4165a8aa06b887748fab3742253c27013713f`.
That exact helper has an actual retained check/build/ordinary-verify pass. This
new theorem has no source, kernel or axiom-audit result yet.

The statement is universal over the complete UInt8 carrier, including 255. It
says that converting `nextByte(value)` back to Nat equals the remainder of
`toNat(value) + 1` on division by 256. There are no hypotheses, input bounds,
excluded values or computational tables. The universal binder is inside the
closed theorem statement, not a free theorem parameter. The proof program is
the admitted `decide` form. Actual success depends on the pinned Lean toolchain
synthesizing a computational Decidable instance for that universal proposition
and checking it at unchanged stock settings. Neither fact is assumed here.

The candidate declares the exact existing axiom set `[Quot.sound, propext]`.
This is not an allowed upper bound. The qualifier must audit actual equality of
the observed and declared sets for both the imported helper and new theorem.
No new axiom declaration, admitted theorem, compiler extension or tactic-budget
change is introduced.

The exact source/closure manifest and intended false controls are in
`owned-enumeration-byte-step-proof-input.json`. Minimal commands are:

```
lexlean check src/Uorc/Search/UnsignedByteConversionLaws.lex.tex
lexlean build src/Uorc/Search/UnsignedByteConversionLaws.lex.tex
lexlean verify src/Uorc/Search/UnsignedByteConversionLaws.lex.tex
```

No command above was run locally and no compiler/toolchain was constructed.
The same retained qualifier must run these commands at stock settings, preserve
generated Lean/kernel replay and actual axiom audits, then apply the exact false
controls in isolated copies and restore/reverify the original source. A source
check alone does not reject a false theorem; its proof must fail verification.

If Decidable synthesis or the stock reduction budget prevents this proof,
record that precise strategy blocker. Do not replace the universal domain with
finite evaluator examples, extend a guard, alter the helper or weaken a theorem.

Only after this byte-step theorem and its false controls qualify should the
modular-successor lemma and genuine Nat-induction proof of the full unbounded
`fromNat` conversion law be attempted. They are not included in this checkpoint
and remain open. All 63 original Model/OwnedProofs declarations, proof programs
and domains remain byte-identical and mandatory, as do owned Arena/builder
refinement, actual archive comparison, native execution and full pinned-SDK
acceptance. This input does not turn byte-step correctness into producer
completeness or a compression guarantee.
