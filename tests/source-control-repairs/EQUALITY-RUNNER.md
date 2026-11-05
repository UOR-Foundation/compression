# Equality regression replay

`EqualityClassScenarios.lex.tex` contains all 17 source-owned cases: deep
State/Arena discrepancies, payload-sensitive Funding/Owner outcomes, and identity
controls. The runner asserts every case individually.

Use Python 3.11+, an already prepared language-1.2 compression project containing
the repairs, the official semantic-source helper, and its matching compiler and
Lean toolchain. `RUNTIME_ROOT` must contain the official
`LexLeanRuntime/Portable.olean` and its matching compiled sidecars. No tools or
runtime implementation are installed or vendored by this runner.

```sh
python3 tests/source-control-repairs/run_equality_regression.py \
  --compiler "$LEXLEAN" \
  --project "$PREPARED_PROJECT/lexlean.toml" \
  --helper "$SEMANTIC_SOURCE_HELPER" \
  --lean "$LEAN" \
  --runtime-root "$RUNTIME_ROOT" \
  --proof-file tests/source-control-repairs/DefaultEqualityClassAudit32.lean \
  --output "$NEW_OUTPUT_DIRECTORY"
```

The output directory must not exist. The runner copies the prepared source,
configuration, workspace files and lock into it, then temporarily places the test
fixture under that copy's configured source root. The input project is unchanged;
the fixture remains a test file in this repository.

It builds through the official compiler, derives the generated module/import
closure, compiles every complete emitted module, runs all 17 VM assertions, replays
the sibling stock-option kernel audit, and checks a fresh import. It adds no
`set_option` or Lean budget flags. Each command has a 60-second external timeout;
any failure stops the replay. `summary.json` and per-command output retain source,
tool, runtime and generated-module hashes.

These are selected regression and equality-proof results. They do not establish
full Controls, whole-graph, pinned-SDK or release acceptance.
