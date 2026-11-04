# Production closure diagnostic

The isolated language-1.2 diagnostic fixture passed the official SDK source check for 14 modules. The configuration and lock hashes below refer to that fixture, not the product SDK locks. Scoped lowering succeeded for 21/21 production roots: 19 runtime roots and 2 test roots.

All requested runtime-closure checks passed: True. The check inspected the official eligibility runtime closure, emitted reachable function origins, and lowered ADT provenance for mathRope/Rope/mathTrie/Trie/CheckedGraph/HeapAudit and atomic RuntimeTable.put/writePath.

| Root | Category | Reachable functions | Diagnostic functions | Atomic table calls |
| --- | --- | ---: | ---: | ---: |
| Uorc.Uorc.Core.Heap.allocate | runtime | 29 | 0 | 0 |
| Uorc.Uorc.Core.Heap.releaseStep | runtime | 17 | 0 | 0 |
| Uorc.Uorc.Core.HeapTests.suite | test | 86 | 22 | 0 |
| Uorc.Uorc.Core.RuntimeByteBuilder.begin | runtime | 35 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeByteBuilder.beginAppendOwned | runtime | 8 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeByteBuilder.beginFinishOwned | runtime | 8 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeByteBuilder.cancel | runtime | 18 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeByteBuilder.step | runtime | 111 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeSequence.beginConcat | runtime | 31 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeSequence.beginSlice | runtime | 31 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeSequence.beginIndex | runtime | 31 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeSequence.beginSpan | runtime | 31 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeSequence.step | runtime | 85 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeSequence.cancel | runtime | 28 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeStore.step | runtime | 65 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeStore.cancel | runtime | 21 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeTable.step | runtime | 53 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeTable.cancel | runtime | 7 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeTable.beginPut | runtime | 59 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeTable.beginErase | runtime | 59 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeTableCursorTests.suite | test | 104 | 21 | 0 |

Test-root HeapAudit dependencies are intentional independent auditing. Counts concern syntactically reachable emitted function closures, including alternate branches and captured closures, rather than observed executions. No claim is made about all unregistered executable helpers.

Source ID: `6aefce65f980b6dea56416ff66c0bf4af6616232f8cc0fb74fe8dfb867121690`. Semantic ID: `9d3938447672513235aa6d50a0fd37f78a4cf52d29dd1f71094b838fbbf122cd`.

This is read-only diagnostic evidence. It does not qualify certificate B, export a package, render new Rust, or measure native behavior, allocation, RSS or time.

SDK source: [`71defb4c663aaec5b9a3aab04cbc6a03e7106227`](https://github.com/UOR-Foundation/LexLean/tree/71defb4c663aaec5b9a3aab04cbc6a03e7106227).

| Source or configuration | SHA-256 |
| --- | --- |
| `src/Uorc/Core/Heap.lex.tex` | `8bb1318242ce855b266a7b0f076a0c49f28c7ddd742bd902c937534cfb32fe81` |
| `src/Uorc/Core/HeapAudit.lex.tex` | `9e777b30e67fc05df81d23fbd0674d06c65f606742370622ae9c5abbbcaa794e` |
| `src/Uorc/Core/HeapTests.lex.tex` | `a58801b2f7b7cc36b197d5025fc23cfc646c7caf97d303e9fd770de114c5af3b` |
| `src/Uorc/Core/RuntimeByteBuilder.lex.tex` | `c7f67542e9f7a4f05f6161fe5fc23b26cf85769c3538d89f0e69f35990eac00d` |
| `src/Uorc/Core/RuntimeByteBuilderTests.lex.tex` | `f39406f2cfd5fbb988b9ace5a22e33a6ff42343047a8b763e01f632416ccdf80` |
| `src/Uorc/Core/RuntimeIndexTests.lex.tex` | `ed4f5f7a5cdfc69a76a41de39e914e690ac921e06d00e44a9c0a2fb076644e41` |
| `src/Uorc/Core/RuntimeSequence.lex.tex` | `c352ecaf1eb161df6df7c6f0fed7fc7868d00e4fc9fe3590271d40604785a29e` |
| `src/Uorc/Core/RuntimeSequenceTests.lex.tex` | `09a3bf982a3288a4b5969b782fd06ee165d3af469767350f8c26a9e0264d643c` |
| `src/Uorc/Core/RuntimeStore.lex.tex` | `833f2fb91be50ad892dbca9f60973300e9b2626dcd3e5650b6a7db8950004ee6` |
| `src/Uorc/Core/RuntimeStoreTests.lex.tex` | `680b88d23503c07a2b23dce78d67832356168b85f117128457b4a17273b42410` |
| `src/Uorc/Core/RuntimeTable.lex.tex` | `38162eb828d933561668ecab43b34a512b58a23259cd588a203cc2799e4abfbc` |
| `src/Uorc/Core/RuntimeTableCursorTests.lex.tex` | `8d9758d9a21fad6c7af6112b25e9e2657d875084144b4f826b526aab53bc0df5` |
| `src/Uorc/Core/RuntimeTableTests.lex.tex` | `8fab9615375973cd0d6fbd4a71e9f8aabfd850502ae11bb219c1c08f83d8727b` |
| `src/Uorc/Core/RuntimeValue.lex.tex` | `8b3ed5db4fbfce0f796d1164ff106733c0fa63f150b3bc162f99b505257b331f` |
| `lexlean.toml` | `dc4e001a3fd15bbd483a9c6a26164ed641341333dac77aa6e2ceadd661544590` |
| `lexlean.lock` | `49427fafc859cfe07e70d0fe28c6a337daea59889161afa0608f4e369b3d51d2` |
