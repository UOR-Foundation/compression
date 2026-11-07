<!-- uorc:non-authoritative -->
# Builder certificate growth: structural diagnosis

Splitting the singleton tree into 16 helpers reduced rendered Rust from 906,513 to 868,878 bytes, but increased genuine certificate B notation from 6,703,926 to 6,866,519 bytes in the same SDK 71 context containing 21 modules and 21 roots. Normalizing numeric function IDs by their official origins leaves only singletonBytes changed among 126 common functions. Sixteen functions are added and the algebraic data types are identical. Both B records use numeric function identities: singletonBytes, RuntimeByteBuilder and clearCleanupTarget do not occur in either record.

The actual B DAG has 16 more constant nodes, 3,076 more applications, 432 more lambdas and 32 more forall nodes. Maximal application DAG spines headed by Corr.callOps increase 463→479. Corr.cond remains 665; fieldRead remains 2,627; fieldHeld remains 110. Recorded SDK metrics keep record slots 53,011, shapes 5,439, match pairs 3,198 and literal hex digits 512 unchanged. The extra 368 lambdas belong to the 16 new function rows, with 64 more in crate_ok.

The exact rendered-byte partition uses declaration reachability: each printed node block, including its inlined children, belongs to the earliest declaration that reaches its root. All headers, blocks and surrounding source reconcile with no unattributed bytes:

| Assigned material | Change in bytes |
|---|---:|
| Original singleton function proof | -794,986 |
| Sixteen new helper function proofs | +944,570 |
| Target program | +2,924 |
| Rust crate data | +3,299 |
| crate_ok | +6,389 |
| Remaining function rows | +397 |
| Total | +162,593 |

Fifteen helper rows each receive 59,317 bytes and the remaining helper row 54,815 bytes under this convention. Global sharing, syntax-node inlining, row order and numeric references affect the partition. It is not an isolated causal cost for each function.

A separate diagnostic with one module copies the authentic singleton declarations and uses the same pure probe wrapper in both cases. With no Heap, Sequence or record slots, its functions increase 2→18 and B notation increases 1,094,760→1,257,410 bytes: 162,650 bytes, only 57 more than the full-context increase. Both small outputs are below the unchanged limit. This reproduces the growth associated with function boundaries; it does not reproduce the full threshold failure or establish production acceptance.

The portable analyzer consumes only two inspect-b directories and the expected findings. It hash-checks six path-neutral files per case before parsing: identity, program, origins, rendered Rust, B data and B notation. Official SDK metrics are copied from the supplied findings and explicitly labelled as recorded, not recomputed. A one-byte corruption of a temporary program copy is rejected before measurement and leaves the original unchanged. No retained Lean LCNF or SSA artifact is asserted; target-program and Rust AST measurements are labelled as such.

The original full 21 standard verification remains the acceptance boundary. No source optimization, compiler build, guard change, kernel check, export or full verification was performed by this analysis. The complete private provenance report is hash-bound in summary.json; portable reports contain no executor paths.
