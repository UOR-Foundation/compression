<!-- uorc:non-authoritative -->
# Cache reader and unchanged-write proof registration

Registered source root: Uorc.Core.RuntimeMachineCacheRefinement.
Theorem roots: unitsAt_reads_natural_leaf, unitsAt_missing_is_typed,
unitsAt_byte_leaf_is_invalid, storeUnitsAfter_unchanged_byte.
Scenarios use actual native table lookup and leaf decoding equalities, native
units-table tag and null-handle tests; successful natural decoding returns the
same units, null decoding produces missing, and byte decoding produces invalid.
The store scenario uses arbitrary units/bytes, the exact existing validity and
observed-nondefault-small-store guard, and writes the observed byte unchanged.
No whole-table mutation, cardinality, heap meaning, or canonical binding invariant
is assumed or established. Those remain separate obligations.

Proof authoring is currently **unqualified**. The exact four implication
statements build under official LexLean 0.4.0 in diagnostic build
`1fb7621b1571766b9336a98feb135ce7104a21d043c3282cdaddb2eed7c4570d`,
but Lean rejects their simplify proofs: that proof form emits only
`simp only [unitsAt]` (or `[storeUnitsAfter]`), without introducing or rewriting
from the hypotheses. No successful kernel or axiom claim is made.

The available `linear_arithmetic` route was also tested in diagnostic build
`2bfb71f35bf41019f3d5f0f9ce3add2f3ebea2b62701dc87760ea58869f29ac1`.
Its backend introduces hypotheses, substitutes local equalities, simplifies at
all locations, repeatedly splits conditionals, and invokes omega. Kernel checking
still fails: actual lookup/decode equations and Result equalities are not
arithmetic constraints. It does not rewrite from arbitrary lookup hypotheses.

`term_cases` rejects primitive Bool tests because it requires a document-defined
inductive type. SemanticType has no dependent proposition-valued parameter type,
so moving `hLookup : lookup = ok leaf` into theorem parameters is unavailable.
A premise-aware simplification or hypothesis rewriting proof primitive is needed
for these exact statements; no substitute parser/compiler or weakened invariant
was added. No mutation rejection or restored pass is claimed for this work.

The exact failing source was moved outside production `src` to
`tests/runtime-machine/probes/cache-reader-implication.lex.tex`, SHA256
`290f5e703c2ed789d4788f731a2b5b14c60b67985eb3954a438127192fe8e894`.
Its four names are unsupported probe roots, never validated source theorem roots.
There is no RuntimeMachineCacheRefinement module in production source roots.
`tests/runtime-machine/evidence/cache-reader-proof-blocker.json` records both
failed build/kernel attempts; adjacent probe logs preserve diagnostics with the executor workspace prefix removed. Original and published log hashes are recorded.
The second attempt differs only by selecting `linear_arithmetic` in place of
`simplify` for all four proof forms. No acceptance or mutation pass is claimed.
