<!-- uorc:non-authoritative -->

# Exact public machine parsed-source transfer

machine-72f3.json is read-only output from the existing official PrismPM reader
for these two modules at public compressor commit
`72f3eec3198f080fe763db67a014b1c6e4b9608a`:

| Module | Original source SHA256 |
| --- | --- |
| RuntimeMachineRefinement | dbc882c904b1ca60b88ad20c02bcb14ac678ad2d86de6a752d7fe3fb5bd0606c |
| RuntimeMachineEngine | dff9d16ab57303428665b272ee2f3dd4194fa18b4b0d08edd6ef5ba53d2dff3a |

Each original UTF8 source was fetched through the GitHub connector at that
immutable ref and checked against its Git blob SHA1 and byte length in the
complete commit tree. The cache retains each unmodified parsed_module plus
source/import/executable/axiom metadata. Parsing twice gives byte-identical
official output. The source remains authoritative; this cache is never loaded
by the product and does not declare new behavior or substitute a parser.

The exact RuntimeMachineRefinement source roundtrips byte-exactly. The official
writer normalizes RuntimeMachineEngine text; reading that normalized text
produces byte-identical IR to reading the original. The original source SHA256
and parsed IR are preserved, and the separate canonical-text hash is recorded.
No source file in the machine branch was edited or reformatted.

Only these two actually present modules are included. RuntimeMachineSemantic
is absent at that commit. The official read_module API does not require a
linked import closure, so no declarations were inferred or copied from a
different revision. This is grammar/reader evidence, not linking, kernel,
runtime, compiler export or release acceptance.

Reproduce after materializing the exact source bytes and their Git-bound
source-manifest.json using the existing official reader:

```sh
node tests/heap/export-machine-cache.mjs \
  /absolute/official-PrismPM/target/debug/prismpm-semantic-source \
  /absolute/official-PrismPM /absolute/exact-source-root \
  /absolute/output/machine-72f3.json
```

The bundle records the clean PrismPM commit, helper source/binary, Cargo.lock,
vendored LexLean crate and export-script SHA256 values. No compiler or parser
was rebuilt. Bundle size: 886,870 bytes; SHA256:
`ea2802eeec281a1d376d3c674c8d54720ce2e9fbf0ea8c44674fae217a655c3f`.
