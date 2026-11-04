# Builtin Option term_cases source fixture

This isolated source project states:

`For every chosen : Option UInt8, observe (toEvent chosen) = chosen.`

`Event` is a document inductive with `missing` and `present UInt8` constructors.
`toEvent` maps builtin `Option` into it; `observe` maps back. The theorem uses
only the existing source `term_cases` form on `chosen`, with the exact
`[propext]` annotation. The source and statement are unchanged from the fixture
used for the admission comparison.

The SHA-256 of `src/Main.lex.tex` is
`fba1b5ef08d40dc8e961471815132ce53912ed9dace95d2ce472e1d516bd10f2`
(998 bytes).

## Reproduce with the existing candidate

Use the already-built LexLean candidate from source commit
`3fe13b50e8d3b20e7e24d7fc8cd6b7a232c1471e`, without rebuilding it. Set
`LEXLEAN_BIN` to that existing executable. Run the following from this directory
in a disposable checkout. The `lock` command creates only this fixture's local
diagnostic lock; it does not change the compression product lock.

```sh
"$LEXLEAN_BIN" --version
"$LEXLEAN_BIN" --project lexlean.toml --diagnostic-format json lock
"$LEXLEAN_BIN" --project lexlean.toml --diagnostic-format json check src/Main.lex.tex
```

If `check` succeeds, continue:

```sh
"$LEXLEAN_BIN" --project lexlean.toml --diagnostic-format json build src/Main.lex.tex
"$LEXLEAN_BIN" --project lexlean.toml --diagnostic-format json verify src/Main.lex.tex
```

Keep ordinary compiler/project limits unchanged. These commands qualify only
this isolated proof fixture.

## Observed status

- **Older compiler observation:** the compiler built from public source
  `71defb4c663aaec5b9a3aab04cbc6a03e7106227` rejects this fixture during source
  checking with `LLT4001`: `term_cases splits a term that is not of a document
  inductive type`.
- **Exact candidate source inspection:** at
  [`3fe13b50`](https://github.com/UOR-Foundation/LexLean/blob/3fe13b50e8d3b20e7e24d7fc8cd6b7a232c1471e/crates/lexlean/src/ir/semantic.rs#L8804),
  `check_term_cases` still requires a named document inductive. This is a
  source inspection result, not an execution result.
- **Candidate execution:** this fixture has not yet been run with that
  already-built candidate. No candidate acceptance or execution-based defect
  claim is made here.

## Existing related work

[LexLean #280 / PR #299](https://github.com/UOR-Foundation/LexLean/pull/299)
concerns fixed constructor disjointness/injectivity simplification after an
admitted document-term split, including `Option` equations in its goals.
This fixture separates builtin `Option` scrutinee admission from those
simplification rules. It does not reopen or duplicate that issue or PR.
