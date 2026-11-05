# Table final-drain cancellation control

The source-owned `Uorc.Wire.TableCancelRepro` uses actual shared Core source at
`f947c988c6769fac2d94a850afee19015d18653c`; it does not implement a table cursor.
It starts `RuntimeTable.beginPutReserved` on an empty `slots(wire)` table at key
0 with natural value 129, then calls the actual `step` until the cursor is in
`PutPhase.draining`, `control` is null and `subtree` is non-null. The fixture
reaches that state after 133 Core mutation steps.

Cancellation at this boundary reports `PutOutcome.cancelled` and returns no
cursor, but HeapAudit against an empty external inventory fails. In this finite
fixture handle 66 still has one root ticket; the remaining logical owned bytes
are 3648. Those fixture identifiers are diagnostic observations, not API limits.
The cancelled outcome contains no table/result ticket that the caller could
collect. This distinguishes the defect from failing to collect an ordinary
`PutOutcome.complete(table)` result.

The source reason is the draining branch's terminal condition: once release is
complete and control is null, it returns cancelled when the flag was changed,
without first returning to cancellation cleanup for the still-live subtree.
The table owner should review that shared contract. No Core file is changed by
this wire checkpoint.

`LengthStore.cleanup` now advances a release already in progress through the
actual Core step. If that step completes the put normally, it records the
returned table owner and releases it during cleanup. Otherwise it preserves the
actual cursor and initiates cancellation after that release step. It neither
rewrites a Core result nor manufactures an empty inventory. The source-owned
LengthStoreScenarios gate, which failed at wire prefix 134 before this fix, now
passes all 138 prefixes through the first stored length.

Executed commands used the existing LexLean candidate
`3fe13b50e8d3b20e7e24d7fc8cd6b7a232c1471e`: fresh diagnostic `lock`, `build`,
compiler-emitted Lean compilation, then `lean --run` importing
`Uorc.Wire.TableCancelRepro.run` (expected ownership failure, exit 1) or
`Uorc.Wire.LengthStoreScenarios.run` (restored controller, exit 0). The diagnostic
receipt binds the build/source hashes and outputs. Complete acceptance remains
the locked SDK's `just vv`; it has not been run.
