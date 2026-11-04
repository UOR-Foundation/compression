# Runtime ownership verification evidence

This is scoped implementation evidence. It does not accept a compression release or close the resource/refinement issues.

The source was checked with the saved official LexLean 0.4.0 compiler from public commit `71defb4c663aaec5b9a3aab04cbc6a03e7106227` and Lean 4.32.1. Product SDK/template locks remain unchanged. The standalone fixture uses language 1.2; the product's older SDK/language lock has not been migrated or qualified here.

## Observed results

- Source linking and canonical formatting pass for the carrier, arena, auditor, tables and their tests.
- Eleven closed kernel negative scenarios elaborate, including each public error category. Exact UInt64/UInt8-related dependencies are traced as `Quot.sound` and `propext`; carrier-only definitions retain exact empty sets. No custom axiom or `sorry` was introduced.
- Both ownership suites execute successfully in the official Lean runtime. They cover aliased child edges, binary slices, distinct equal-image/equal-value allocations, exact-budget release, unique image reservations and fresh-image commitment, persistent slot paths through zero/maximum keys, metadata owners and final reclamation. The explicit caller-root inventory extension was added afterward; its execution remains separately qualified until the next receipt.
- A planted source defect changes `retainEntry`'s incoming increment from one to zero. The owning `conformance_negative_countOverflow` kernel theorem rejects it with `LLV7002`. The diagnostic copy is restored to the exact control hash. Its sibling root-overflow check cannot mask the planted defect: this scenario has incoming `UInt64.max` and roots `1`.
- Actual production export is blocked by `LLV7015`, preservation certificate B, in the generated RustStd correspondence proof for `Heap.allocate`. Kernel elaboration, Leanchecker and exact axiom audit precede this failing stage. No generated Rust bundle is accepted, and no generated Rust native execution is claimed.

The normalized product diagnostics and exact mutation hashes are in `runtime-heap-evidence.json`. They contain no environment credentials or private machine configuration.

## Reduced export reproducer

`tests/heap/export-repro/src/Main.lex.tex` reproduces the same certificate-B failure using only one record, one three-constructor recursive index, and the `left` accessor. It does not import the compressor or its arena. The rejection is in `HeapRepro.Main.left`, generated `fun0`, under `LexLeanPreservation.Rust.Corr.mCons`. The full arena failure is generated `fun21`. Both are kernel application-type mismatches in the actual proof; neither is an unsigned-axiom-policy mismatch or an unsupported-source diagnostic.

Copy the fixture to a fresh directory and run the official compiler:

```sh
lexlean --project REPRO/lexlean.toml lock
lexlean --project REPRO/lexlean.toml check --all
lexlean --project REPRO/lexlean.toml verify --all --diagnostic-format json
```

`REPRO` is the absolute path of that copy. Do not weaken or bypass preservation checks. This is a reduced confirmed reproducer, not a proposed upstream correction.

## Reproduce the ownership diagnostic fixture

```sh
node tests/heap/prepare.mjs /absolute/fresh/heap-check
lexlean --project /absolute/fresh/heap-check/lexlean.toml lock
lexlean --project /absolute/fresh/heap-check/lexlean.toml fmt --check --all
lexlean --project /absolute/fresh/heap-check/lexlean.toml check --all
lexlean production targets --diagnostic-format json > targets.json
node tests/heap/request.mjs targets.json > request.json
lexlean --project /absolute/fresh/heap-check/lexlean.toml production build --request request.json --diagnostic-format json
```

The prepared source inventory records every copied source's hash. The last command currently fails at certificate B. Once a bundle is accepted, replay it with `lexlean production verify-artifact BUNDLE`, then use `node tests/heap/native.mjs /absolute/BUNDLE /absolute/fresh/native-check` to run assertions against byte-identical generated implementation copies. That Rust-native stage remains unvisited while export is blocked.

To plant the actual count defect, prepare a second fresh fixture and run `node tests/heap/mutate.mjs /absolute/fresh/mutant-check`, then `lexlean --project /absolute/fresh/mutant-check/lexlean.toml verify src/Uorc/Core/HeapTests.lex.tex --diagnostic-format json`. Require the owning `LLV7002` rejection; a stale source, wrong diagnostic, skipped test or generic compiler failure is not the intended negative result.

## Remaining acceptance boundaries

The whole machine must replace the old value/rope/store/graph/cache backing with these handles and owner roots, account its pending reservation root tickets, and compose the independent reference interpretation. It must also preserve the separate work partitions. General reference refinement, actual generated ownership/RSS behavior, immutable locked SDK qualification, model/registry projection integration and complete `just vv` remain separate stages. The local language/runtime checks are not their substitute.
