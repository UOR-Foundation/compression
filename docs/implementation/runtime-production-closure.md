<!-- uorc:non-authoritative -->

# Production closure diagnostic

The isolated language-1.2 fixture passed the official SDK source check for23 modules. Configuration/lock hashes identify that diagnostic fixture, not the product SDK locks. Scoped lowering succeeded for 21/21 production roots: 19 runtime roots and 2 test roots.

All requested runtime-closure checks passed: True. The check inspected the official eligibility runtime closure, emitted reachable function origins, and lowered ADT provenance for mathRope/Rope/mathTrie/Trie/CheckedGraph/HeapAudit, all Proofs modules, HeapMovedReference/oldAllocateMoved, and atomic RuntimeTable.put/writePath.

| Root | Category | Reachable functions | Diagnostic functions | Atomic table calls |
| --- | --- | ---: | ---: | ---: |
| Uorc.Uorc.Core.Heap.allocate | runtime | 29 | 0 | 0 |
| Uorc.Uorc.Core.Heap.releaseStep | runtime | 17 | 0 | 0 |
| Uorc.Uorc.Core.HeapTests.suite | test | 87 | 22 | 0 |
| Uorc.Uorc.Core.RuntimeByteBuilder.begin | runtime | 35 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeByteBuilder.beginAppendOwned | runtime | 8 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeByteBuilder.beginFinishOwned | runtime | 8 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeByteBuilder.cancel | runtime | 18 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeByteBuilder.step | runtime | 112 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeSequence.beginConcat | runtime | 31 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeSequence.beginSlice | runtime | 31 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeSequence.beginIndex | runtime | 31 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeSequence.beginSpan | runtime | 31 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeSequence.step | runtime | 86 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeSequence.cancel | runtime | 28 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeStore.step | runtime | 66 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeStore.cancel | runtime | 21 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeTable.step | runtime | 54 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeTable.cancel | runtime | 7 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeTable.beginPut | runtime | 60 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeTable.beginErase | runtime | 60 | 0 | 0 |
| Uorc.Uorc.Core.RuntimeTableCursorTests.suite | test | 105 | 21 | 0 |

Test-root HeapAudit dependencies are intentional independent auditing. Counts concern syntactically reachable emitted function closures, including alternate branches and captured closures, rather than observed executions. No claim is made about all unregistered executable helpers.

Source ID: `fae2a2fdfeacf7c7a0950ff7143b9fd3e408363965f0a40e879f65f9d256e81b`. Semantic ID: `6395322665699eb184cc81fae7264483706e3181f24289cd47a03e6203797bea`.

This is read-only diagnostic evidence. It does not qualify certificate B, export a package, render new Rust, or measure native behavior, allocation, RSS or time.

SDK source: [`71defb4c663aaec5b9a3aab04cbc6a03e7106227`](https://github.com/UOR-Foundation/LexLean/tree/71defb4c663aaec5b9a3aab04cbc6a03e7106227).

| Source or configuration | SHA-256 |
| --- | --- |
| `src/Uorc/Core/Heap.lex.tex` | `ee41c3fb11c1f60fc767448760e8020d934c253fc31eec2146413ec1a98b0e1b` |
| `src/Uorc/Core/HeapAudit.lex.tex` | `9e777b30e67fc05df81d23fbd0674d06c65f606742370622ae9c5abbbcaa794e` |
| `src/Uorc/Core/HeapMovedReference.lex.tex` | `1f5620417509899049cb2e16119aebcf9c62a23de2886f6fac5db2360d71a76e` |
| `src/Uorc/Core/HeapMovedTests.lex.tex` | `871ba6e67c8e0430dd35242f3d9f7fbaa3d9145a085010a07e132648056f4e93` |
| `src/Uorc/Core/HeapTests.lex.tex` | `a58801b2f7b7cc36b197d5025fc23cfc646c7caf97d303e9fd770de114c5af3b` |
| `src/Uorc/Core/RuntimeAuditProofs.lex.tex` | `4a4c4585afbe0af68e9551700d9ce2527e7588e30602eaebb547cef4a77fa322` |
| `src/Uorc/Core/RuntimeByteBuilder.lex.tex` | `c7f67542e9f7a4f05f6161fe5fc23b26cf85769c3538d89f0e69f35990eac00d` |
| `src/Uorc/Core/RuntimeByteBuilderTests.lex.tex` | `f39406f2cfd5fbb988b9ace5a22e33a6ff42343047a8b763e01f632416ccdf80` |
| `src/Uorc/Core/RuntimeCost.lex.tex` | `3912333b87cd99b9a6ca8b072911ec3678e6b4ba4e5698169f56e9c8e0de0002` |
| `src/Uorc/Core/RuntimeIndexTests.lex.tex` | `ed4f5f7a5cdfc69a76a41de39e914e690ac921e06d00e44a9c0a2fb076644e41` |
| `src/Uorc/Core/RuntimeObservationProofs.lex.tex` | `c3bc061208a9205dd814da1a450d9aeb0b1bd1e3fd31ad7edf0cb16d12c4eb4b` |
| `src/Uorc/Core/RuntimeOwnershipProofs.lex.tex` | `cdd3cfb90bc7435018cd1d49df173f29a6d6c2f4aa8db698eaded6d7066af088` |
| `src/Uorc/Core/RuntimeSequence.lex.tex` | `c352ecaf1eb161df6df7c6f0fed7fc7868d00e4fc9fe3590271d40604785a29e` |
| `src/Uorc/Core/RuntimeSequenceProofs.lex.tex` | `f22bf696ad3a762ac3f60b1290ad5fd6b4c6db1dfc278165156eb75db5d4dc54` |
| `src/Uorc/Core/RuntimeSequenceTests.lex.tex` | `09a3bf982a3288a4b5969b782fd06ee165d3af469767350f8c26a9e0264d643c` |
| `src/Uorc/Core/RuntimeSequenceTicketTests.lex.tex` | `d693068b2c3bdd3bd3053bdf94e5981d1921c117e26f14819a85af67789b7b05` |
| `src/Uorc/Core/RuntimeSequenceTickets.lex.tex` | `09b439881a7d509022acabb8a4a224ff8fe642d7d5de485876d2026774697b75` |
| `src/Uorc/Core/RuntimeStore.lex.tex` | `833f2fb91be50ad892dbca9f60973300e9b2626dcd3e5650b6a7db8950004ee6` |
| `src/Uorc/Core/RuntimeStoreTests.lex.tex` | `680b88d23503c07a2b23dce78d67832356168b85f117128457b4a17273b42410` |
| `src/Uorc/Core/RuntimeTable.lex.tex` | `38162eb828d933561668ecab43b34a512b58a23259cd588a203cc2799e4abfbc` |
| `src/Uorc/Core/RuntimeTableCursorTests.lex.tex` | `8d9758d9a21fad6c7af6112b25e9e2657d875084144b4f826b526aab53bc0df5` |
| `src/Uorc/Core/RuntimeTableTests.lex.tex` | `8fab9615375973cd0d6fbd4a71e9f8aabfd850502ae11bb219c1c08f83d8727b` |
| `src/Uorc/Core/RuntimeValue.lex.tex` | `8b3ed5db4fbfce0f796d1164ff106733c0fa63f150b3bc162f99b505257b331f` |
| `lexlean.toml` | `9a245abe5cfb040c7b4db71bfac6a036b65ecb7b53372ab47561b65bdbe2b5de` |
| `lexlean.lock` | `1ff537ad549116310c6fff037517795d27926f7d035d9c550ecd61b46c5f8717` |

This final-policy rebind changes only HeapMovedReference axiom-policy metadata. All 21 official root closure records are identical to the preceding fusion review. The final build is `341efda77646bef3e62f974d08bf7810c976237698dc5318267adbd23e89e7aa`.

All four Proofs modules, RuntimeCost, RuntimeSequenceTickets, and HeapMovedReference have zero production roots and are absent from every emitted runtime function origin. Six runtime roots reach the new fused allocation helpers.

Official generated Heap declaration spans remain 54 unchanged named declarations, one changed function (`allocateMoved`), and two added helpers. All 44 Heap declarations in the frozen ownership proof module's transitive reference closure retain byte-identical generated declarations. Compiler-generated private auxiliaries are outside this count.

All 23 generated Lean files are independently confirmed byte-identical across the metadata correction.
