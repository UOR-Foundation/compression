<!-- uorc:non-authoritative -->
# Accounting component qualification inputs

These are the exact language-1.2 diagnostic configuration and lock used for
the source-owned accounting proofs and controls. Product SDK/language locks
are unchanged. This is not a preservation-export or release acceptance path.

Copy these four configuration/workspace files into a fresh fixture directory.
Copy the six authoritative repository source files named in
`source-inventory.json` to their same relative paths in that directory, and
verify every SHA-256 before checking. The original component commands are:

```sh
EXISTING_LEXLEAN --project FIXTURE/lexlean.toml fmt --check \
  src/Uorc/Core/RuntimeAccountingProofs.lex.tex \
  src/Uorc/Core/RuntimeAccountingTests.lex.tex --diagnostic-format json
EXISTING_LEXLEAN --project FIXTURE/lexlean.toml check \
  src/Uorc/Core/RuntimeAccountingTests.lex.tex --diagnostic-format json
EXISTING_LEXLEAN --project FIXTURE/lexlean.toml build \
  src/Uorc/Core/RuntimeAccountingTests.lex.tex --diagnostic-format json
```

The retained official compiler is SDK Git
`71defb4c663aaec5b9a3aab04cbc6a03e7106227`, binary SHA-256
`38f8502e5c108a362265663253283a0f60f14538951e592b2d5dd4db9e67be0a`,
with Lean 4.32.1. No SDK was built or downloaded for this component.
Kernel, scenario evaluation, exact axiom observations, two completed mutation
gates and byte-exact restoration are separately recorded in
`docs/implementation/runtime-accounting-evidence.json`, including normalized
commands and exact source/build identities. Generated modules, inspection
harnesses and binaries remain private diagnostic artifacts.

The full recursive invariant-to-sum theorem is unproved and absent from the
source. See `docs/implementation/runtime-accounting-refinement.md` for the
qualified general statements, premises and remaining global obligations.
