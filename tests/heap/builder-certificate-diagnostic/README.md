# Builder certificate diagnostic

This diagnostic investigates a failed source refinement. It is not a smaller
production export or release acceptance path. The full matched 21-source,
21-declared-root verification remains the authority: the original `fc419…`
Builder requires 6,703,926 recorded certificate-B bytes; the discarded
sixteen-helper `415d8…` split requires 6,866,519. Both exceed the unchanged
4,194,304-byte guard. Production keeps `fc419…`.

`prepare-matched.py` reproduces both complete input directories from:

- The [public Product diagnostic archive](https://github.com/UOR-Foundation/compression/blob/e0a00895a74f0504e331eee6e82341997b174016/validation/product/read-image-core-compatibility/d5a19f2b6d554a5f18506b1111c7ca92b0ce1b61bfee8a9fc243e6c7203f44d3/diagnostic.tar.gz),
  SHA-256 `b87eaa03a33bf55279852efd1e0f3960670f90d5525461d7b5a0786fbacabc81`.
- The [immutable discarded Builder source](https://github.com/UOR-Foundation/compression/blob/69f7b8986388fc70c2bec090b1b44fc92611bc68/src/Uorc/Core/RuntimeByteBuilder.lex.tex),
  SHA-256 `415d8c8d8183b3787611cf96f5620009177332c71f2a198861c1a77df32bb075`.

Supply those existing files; the preparer performs no download, build or
compiler execution:

```sh
python3 prepare-matched.py diagnostic.tar.gz RuntimeByteBuilder.415d8.lex.tex NEW_OUTPUT
```

All 21 source files and four configuration/workspace files are hash-checked
against `matched-inputs.json`. Both contexts preserve their annotations,
imports and limits. Only the one declared Builder source differs. Archive
members are read individually from that fixed inventory, never extracted by
their paths. An existing output directory, wrong archive or wrong candidate
is refused before output creation.

The original verification command, from each prepared case directory, is:

```sh
EXISTING_OFFICIAL_LEXLEAN --diagnostic-format json verify src/Uorc/Product/ReadImageScenarios.lex.tex
```

The recorded runs use official compiler Git
`71defb4c663aaec5b9a3aab04cbc6a03e7106227`, binary SHA-256
`38f8502e5c108a362265663253283a0f60f14538951e592b2d5dd4db9e67be0a`,
the archive's diagnostic language-1.2 configuration and Lean 4.32.1. They do
not change the product's locked language-1.1 SDK. Exact earlier verification
receipts remain in
`docs/implementation/runtime-byte-builder-matched-qualification/`.

The following scopes remain separate: certificate generation under its file
guard, kernel preservation checking, accepted export and artifact replay,
native behavior, physical memory/copy measurements, and full locked-SDK
`just vv`. A certificate-size refusal establishes none of the later stages.
Product.Application's separate no-Core certificate refusal is not a Builder
result.

## What was measured

`sdk-findings.json` records two new, bounded diagnostic calls through the
existing official SDK library. They check all 21 modules and enumerate all 21
roots, then inspect the actual `Builder.step` root at index 6. They preserve
the original generation budget. The public `certificate_b_data` API returns
typed data before the ordinary renderer's final exact source-size check;
public `sdk::assemble` records that actual data with the same official printer
and header. The resulting lengths reproduce both earlier size refusals
exactly. Historical failed B bytes were never retained, so this establishes
matching identities and lengths, not bytewise comparison with a historical B
file. Neither diagnostic record has kernel or export acceptance.

| Measurement | Original | Discarded split |
| --- | ---: | ---: |
| Target functions | 126 | 142 |
| Certificate declarations | 131 | 147 |
| Certificate DAG nodes | 329,490 | 333,046 |
| Recorded certificate bytes | 6,703,926 | 6,866,519 |
| Rendered Rust bytes | 906,513 | 868,878 |
| SDK record-slot measure | 53,011 | 53,011 |
| Match-arm pairs | 3,198 | 3,198 |
| Literal hexadecimal digits | 512 | 512 |

The split preserves the total branch/literal structure while adding actual
calls and a separately derived function theorem for every helper. The Rust
lowerer retains function calls; it does not inline the helper bodies. Source
function names do not occur in either recorded B, so shorter names are not a
remedy. Full-arity record patterns and their proof witnesses amplify the
baseline certificate, but their unchanged counts cannot explain this split's
increase. Global DAG sharing and notation inlining couple the byte costs;
source length, Rust length and simple node-count bounds do not predict them.

`inspect-b.rs` is a read-only host diagnostic using the existing SDK, not a
runtime implementation. Build only this adapter against the already available
SDK71 library and its existing dependencies:

```sh
EXISTING_RUSTC --edition=2021 inspect-b.rs \
  -L dependency=EXISTING_SDK_DEPS \
  --extern lexlean=EXISTING_SDK_RLIB \
  --extern serde_json=EXISTING_SERDE_JSON_RLIB \
  --extern camino=EXISTING_CAMINO_RLIB -o inspect-b
./inspect-b PREPARED_CASE/lexlean.toml src/Uorc/Core/RuntimeByteBuilder.lex.tex \
  Uorc.Core.RuntimeByteBuilder step NEW_DIAGNOSTIC_OUTPUT
```

Use a 120-second outer timeout for each of the two serial diagnostic calls.
Both recorded calls completed within it. The adapter refuses changed file or
child-time guards and a different full input/root count. It writes private
diagnostic data and a receipt, never an accepted export bundle. `measure.rs`
separately reads the already emitted program through the SDK's actual parser
and measurement APIs; it does not regenerate a certificate. File digests,
SDK identity, exact logical arguments and code references are in the findings;
the original host-specific invocation receipts remain privately hash-bound.

## Singleton-only isolation

`singleton-only/baseline/` and `singleton-only/candidate/` contain one diagnostic
module each. The original one declaration or split's 17 declarations are
copied byte-for-byte from their authentic source; official SDK rereads confirm
their complete declaration objects are unchanged. Both add the same
`probe(UInt8) -> Bytes` root and import no Core module. All 256 possible byte
values remain in the source definition. This isolates the changed function
boundaries without narrowing the byte domain.

| Measurement | Original singleton | Split singleton |
| --- | ---: | ---: |
| Target functions | 2 | 18 |
| Certificate declarations | 7 | 23 |
| Certificate DAG nodes | 47,323 | 50,859 |
| Recorded certificate bytes | 1,094,760 | 1,257,410 |
| SDK record-slot measure | 0 | 0 |

The isolated increase is 162,650 bytes, only 57 bytes different from the full
context's increase. Both isolated records fit the unchanged file-size guard.
This reproduces the growth mechanism; it does **not** reproduce the full
21-root size refusal or qualify a smaller product. No preservation kernel,
standard verification, accepted export or replay was run for these records.
`singleton-only-findings.json` binds source/body checks, complete configurations,
identities, actual results and private artifact hashes.

Build the separate `inspect-singleton.rs` adapter against the same existing
SDK dependencies as `inspect-b.rs`. From each case directory its arguments are:

```sh
INSPECT_SINGLETON lexlean.toml src/Uorc/Core/BuilderSingletonOnly.lex.tex \
  Uorc.Core.BuilderSingletonOnly probe NEW_DIAGNOSTIC_OUTPUT
```

This adapter requires exactly one module and that one probe root. It does not
change the strict 21-module/21-root adapter. Both serial runs completed within
the same 120-second outer bound and unchanged generation guards. No further
source factoring is proposed from this experiment.
