<!-- uorc:non-authoritative -->

# Exact e87 production-export consumer

This diagnostic fixture pins the fourteen source modules and four project files
at public compressor commit `e87cd22372037374f7ce28b6ab0ea3e619129610`.
The source bytes match the authentic fourteen-module parser cache; it includes
no new proof or cost modules and changes no production API. The fixture is an
isolated language-1.2 consumer, not the product's digest-pinned release project.

Create the deterministic source-only archive from this checkout:

```sh
python3 tests/heap/export-e87-fixture.py /absolute/new/compression-e87cd223-production-export-fixture.tar.gz
```

The delivered archive has 62,560 bytes and SHA-256
`3e811591cf2c27d687115dd6a165332a4f0d70fcb3514b8b3a11ecfb069fb4ce`.
Its manifest binds every supplied source, configuration and diagnostic helper.
The request producer corrects the SDK-required ascending root ordering; it does
not alter any e87 LexLean source.

Extract into a fresh directory in the executor with the already-built official
compiler at `3fe13b50e8d3b20e7e24d7fc8cd6b7a232c1471e`, then run:

```sh
node tools/run-export.mjs /absolute/existing/lexlean "$PWD"
```

The runner verifies the manifest and exact source inventory, records the compiler
binary hash/version, runs source lock/format/check, obtains that compiler's
actual `rust-std` target digest, requests all roots and replays the resulting
bundle. It builds no compiler and pulls no SDK. It preserves every command's
exit status, stdout and stderr under `consumer-results/`; an existing result
directory is refused. A failed command cannot produce an accepted receipt.

The complete sorted root inventory is:

```text
Uorc.Uorc.Core.Heap.allocate
Uorc.Uorc.Core.Heap.releaseStep
Uorc.Uorc.Core.HeapTests.suite
Uorc.Uorc.Core.RuntimeByteBuilder.begin
Uorc.Uorc.Core.RuntimeByteBuilder.beginAppendOwned
Uorc.Uorc.Core.RuntimeByteBuilder.beginFinishOwned
Uorc.Uorc.Core.RuntimeByteBuilder.cancel
Uorc.Uorc.Core.RuntimeByteBuilder.step
Uorc.Uorc.Core.RuntimeSequence.beginConcat
Uorc.Uorc.Core.RuntimeSequence.beginIndex
Uorc.Uorc.Core.RuntimeSequence.beginSlice
Uorc.Uorc.Core.RuntimeSequence.beginSpan
Uorc.Uorc.Core.RuntimeSequence.cancel
Uorc.Uorc.Core.RuntimeSequence.step
Uorc.Uorc.Core.RuntimeStore.cancel
Uorc.Uorc.Core.RuntimeStore.step
Uorc.Uorc.Core.RuntimeTable.beginErase
Uorc.Uorc.Core.RuntimeTable.beginPut
Uorc.Uorc.Core.RuntimeTable.cancel
Uorc.Uorc.Core.RuntimeTable.step
Uorc.Uorc.Core.RuntimeTableCursorTests.suite
```

Nineteen are runtime roots. HeapTests and RuntimeTableCursorTests deliberately
include independent auditing. Accepted export/artifact replay is distinct from
the completed seven native Lean suites, 406 exact axiom policies, general
refinement, physical retention/copy measurements and full locked-SDK `just vv`.
This executor prepared and hash-verified the fixture; it has not run the candidate
compiler's export. No new accepted-export result is claimed here.
