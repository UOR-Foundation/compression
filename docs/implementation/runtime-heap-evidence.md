# Runtime ownership verification evidence

This is scoped implementation evidence. It does not accept a compression release or close the resource/refinement issues.

The source was checked with the saved official LexLean 0.4.0 compiler from public commit `71defb4c663aaec5b9a3aab04cbc6a03e7106227` and Lean 4.32.1. Product SDK/template locks remain unchanged. The standalone fixture uses language 1.2; the product's older SDK/language lock has not been migrated or qualified here.

## Observed results

- Source linking and canonical formatting pass for the carrier, arena, auditor, tables and their tests.
- Eleven closed kernel negative scenarios elaborate, including each public error category. All 102 definition/theorem dependency sets are traced; exact UInt64/UInt8-related dependencies are recorded as `Quot.sound` and `propext`; carrier-only definitions retain exact empty sets. `runtime-heap-axioms.json` records the observed sets. No custom axiom or `sorry` was introduced.
- Both ownership suites execute successfully in the official Lean runtime. They cover aliased child edges, binary slices, distinct equal-image/equal-value allocations, exact-budget release, unique image reservations and fresh-image commitment, persistent slot paths through zero/maximum keys, metadata owners and final reclamation. This includes the explicit caller-root inventory checks: missing roots are rejected, while cursor-root transfer, retained aliases, persistent tables and metadata captures match their supplied inventories.
- A planted source defect changes `retainEntry`'s incoming increment from one to zero. The owning `conformance_negative_countOverflow` kernel theorem rejects it with `LLV7002`. The diagnostic copy is restored to the exact control hash. Its sibling root-overflow check cannot mask the planted defect: this scenario has incoming `UInt64.max` and roots `1`.
- Actual production export is blocked by `LLV7015`, preservation certificate B, in the generated RustStd correspondence proof for `Heap.allocate`. Kernel elaboration, Leanchecker and exact axiom audit precede this failing stage. No generated Rust bundle is accepted, and no generated Rust native execution is claimed.

The normalized product diagnostics and exact mutation hashes are in `runtime-heap-evidence.json`. They contain no environment credentials or private machine configuration.

## Reduced export reproducer

`tests/heap/export-repro/src/Main.lex.tex` reproduces the same certificate-B failure using only one record, one three-constructor recursive index, and the `left` accessor. It does not import the compressor or its arena. The rejection is in `HeapRepro.Main.left`, generated `fun0`, under `LexLeanPreservation.Rust.Corr.mCons`. The original full arena failure is generated `fun21`; with the reservation API present it is generated `fun22`. Both are kernel application-type mismatches in the actual proof; neither is an unsigned-axiom-policy mismatch or an unsupported-source diagnostic.

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
lexlean production targets --diagnostic-format json > /absolute/fresh/heap-check/targets.json
node tests/heap/request.mjs /absolute/fresh/heap-check/targets.json > /absolute/fresh/heap-check/request.json
lexlean --project /absolute/fresh/heap-check/lexlean.toml production build --request /absolute/fresh/heap-check/request.json --diagnostic-format json
```

The prepared source inventory records every copied source's hash. The retained original ownership request fails at certificate B. The expanded runtime request now selects the actual resumable entry points; it has not been claimed to pass export, and the known compiler failure is not bypassed. Once a bundle is accepted, replay it with `lexlean production verify-artifact BUNDLE`, then use `node tests/heap/native.mjs /absolute/BUNDLE /absolute/fresh/native-check` to run assertions against byte-identical generated implementation copies. That Rust-native stage remains unvisited while export is blocked.

To plant the actual count defect, prepare a second fresh fixture and run `node tests/heap/mutate.mjs /absolute/fresh/mutant-check`, then `lexlean --project /absolute/fresh/mutant-check/lexlean.toml verify src/Uorc/Core/HeapTests.lex.tex --diagnostic-format json`. Require the owning `LLV7002` rejection; a stale source, wrong diagnostic, skipped test or generic compiler failure is not the intended negative result.

## Remaining acceptance boundaries

The whole machine must replace the old value/rope/store/graph/cache backing with these handles and owner roots, account its pending reservation root tickets, and compose the independent reference interpretation. It must also preserve the separate work partitions. General reference refinement, actual generated ownership/RSS behavior, immutable locked SDK qualification, model/registry projection integration and complete `just vv` remain separate stages. The local language/runtime checks are not their substitute.

## Composed runtime source checkpoint

The fourteen-module snapshot at source commit `1020389325ca824c70c23b3b33705f49e339ed22` passes canonical formatting, official source linking/build, kernel compilation, exact SDK notation roundtrips and all seven source suite predicates in a native Lean C diagnostic. This includes sequence cancellation, sparse-store operations, resumable table writes/erasure and the builder's 35 cases. All 406 definition/theorem dependency policies match their actual axiom traces exactly, with only empty sets or `{Quot.sound, propext}`. Three table helpers' overdeclared dependencies were corrected to their observed empty sets without changing their bodies or weakening the audit.

`runtime-composed-evidence.json` binds every source, generated Lean/C input and executable hash. Source ID is `6aefce65f980b6dea56416ff66c0bf4af6616232f8cc0fb74fe8dfb867121690`; semantic ID is `9d3938447672513235aa6d50a0fd37f78a4cf52d29dd1f71094b838fbbf122cd`; build ID is `fbdd0a0b87be180ef905c9d4153a8aebae4ccd2de546caa1831c873531b491a4`. The diagnostic main only asserts source-owned Booleans. Generated implementations were not edited. The new cancellation mutation discards a produced owned result: the intended gate rejects it, and exact restoration passes.

The official SDK independently lowers all 21 actual production roots. Its nineteen runtime closures exclude mathematical Rope/Trie/CheckedGraph backing, the independent HeapAudit and atomic RuntimeTable writes. The two test roots intentionally retain auditing. `runtime-production-closure.json` and `runtime-production-closure.md` record that scoped evidence; these checks do not accept preservation certificate B.

The e87 authentic parser cache is retained in Git history at `tests/heap/semantic-cache/runtime-owners.json`, SHA-256 `7bf8c8898aac8776919c0c10646a19b65cb81a8b23f461c181762705b78386e9`. It contains all fourteen actual parsed modules, source hashes/imports and executable/axiom metadata. It was generated through the official `prismpm-semantic-source read` helper, with repeat-read and byte-exact writer checks; the saved official LexLean SDK parses all fourteen identically. The current cache can include later additive modules and the fused Heap; its README and `tests/heap/export-cache.mjs` give the supported reproduction route. Source remains authoritative.

These are source, kernel and native Lean diagnostics. Accepted Rust export, independent Leanchecker replay on this final snapshot, physical allocator/RSS measurements, general runtime/reference refinement and full locked-SDK release acceptance are separate unclaimed stages. The older generated Rust investigation establishes shallow Rc sharing for its exact saved program, not a measurement of the final carrier's physical cost.
