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
