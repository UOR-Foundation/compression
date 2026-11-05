# Reference leaf proof diagnostic

This is an unqualified failure reproducer, not a replacement acceptance path.
Production source, the original theorem, its domain, proof, axiom policy, runtime
behavior, guards and all project acceptance roots are unchanged.

`fixture-src/Uorc/Core/ReferenceSemantics.lex.tex` retains seven exact original
declarations in source order, including the complete original theorem and all
of its parameters. `manifest.json` binds their canonical JSON and source bytes.
The original module's import lines are retained. `materialize.py` copies five
byte-exact dependency modules after checking their frozen hashes, and never
invokes a compiler. The complete original module remains authoritative.

Materialize into a new, absent scratch directory:

    python3 tests/reference-leaf-proof/materialize.py --out /tmp/reference-leaf-fixture

For the existing compiler owner only, using the already-built official SDK71
binary and the existing pinned Lean 4.32.1 environment:

    cd /tmp/reference-leaf-fixture
    "$LEXLEAN_BIN" --project lexlean.toml --diagnostic-format json migrate --to 1.2
    "$LEXLEAN_BIN" --project lexlean.toml --diagnostic-format json fmt --check src/Uorc/Core/ReferenceSemantics.lex.tex
    "$LEXLEAN_BIN" --project lexlean.toml --diagnostic-format json check src/Uorc/Core/ReferenceSemantics.lex.tex
    "$LEXLEAN_BIN" --project lexlean.toml --diagnostic-format json build src/Uorc/Core/ReferenceSemantics.lex.tex

Migration must preserve every retained declaration exactly. Save exact source,
compiler binary, generated module, command, environment and diagnostic hashes.
Compile the dependency closure and target using the already established direct
Lean diagnostic procedure. The expected baseline is failure of the unchanged
PF-21 proof, not a pass. No bootstrap, resource increase or production lock/pin
change is part of this reproducer.

## Generated-Lean-only experiment

`patch-generated-diagnostic.py` defaults to the exact published generated module
SHA-256 `909ffa4f488e18258b4ee25a5459bc5df1bd5d3d8df308b09313e95f723af3ed`.
Run it on that full module, not on the smaller fixture's newly generated module:

    python3 tests/reference-leaf-proof/patch-generated-diagnostic.py "$GENERATED_REFERENCE" --out /tmp/reference-leaf-generated-diagnostics

It makes three isolated copies and reports each hash. The positive copy retains
the entire original theorem header, statement and parameters byte-for-byte. Its
only proof change is a constructor/equality simplification after the existing
match split and before omega. Two controls keep the same inputs and left-hand
side, but replace the conclusion with constant `none` or constant `some 0`.

Compile each copy separately with the existing dependency oleans and the same
pinned Lean command. Save its precise command and complete diagnostic. The
positive must pass and **both controls must fail** before the experiment supports
the proposed tactic change. The false-none counterexample is backing `[1]`,
index `0`; false-zero is backing `[]`, index `0`; continuations may be `[]`.

This experiment edits generated diagnostic copies only. A pass cannot establish
that the compiler accepts a source proof, and must never replace source
regeneration, kernel/axiom checks, negative controls or complete `just vv`.

## Qualification after a supported source route exists

1. Regenerate from the unchanged complete production statement using an
   explicitly reviewed source-proof route, or a separately qualified official
   compiler enhancement. Do not publish generated Lean as product authority.
2. Verify the complete ReferenceSemantics closure, exact axiom policies and all
   original claims. Never use the seven-declaration reproducer as module acceptance.
3. In disposable source copies, keep the theorem statement fixed and plant:
   a leaf byte replaced with zero (counterexample `[1]`, index `0`); a failed
   observation replaced with `some 0` (empty backing); or a reference-reader
   index shifted by one (counterexample `[1,2]`, index `0`). Each must be rejected
   by the intended theorem, not an unrelated parser or dependency failure.
4. Restore each source mutant byte-exactly and rerun the positive qualification.
5. Only then qualify the 65 canonical-composition statements and execute their
   64 controls through the existing owner. Production export, pinned SDK and
   complete product acceptance remain separate unfinished boundaries.

No compiler, kernel, source mutant or generated-Lean diagnostic was executed
when this packet was assembled. The materializer, Python syntax checks and
synthetic patch-parser self-test were run; they are only tooling checks.
