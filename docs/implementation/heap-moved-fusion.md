<!-- uorc:non-authoritative -->

# Fused moved-child index updates

Capability `heap-moved-fusion/v1` preserves `Heap.allocateMoved`'s observable
Arena, result handle, Delta/work and typed errors while removing one persistent
index update per nonnull moved child. The authoritative behavior remains source
in `Heap.lex.tex`. `HeapMovedReference.lex.tex` retains the previous composition
of unchanged `allocate`, move preflight and `dropRetainedRoot` for exact
comparison; `HeapMovedTests.lex.tex` supplies diagnostic acceptance scenarios.

The source gate first records the old structural counts 3/5/5/4 for one moved
child, two distinct moved children, two aliased moved tickets, and a moved plus
unmoved alias. The acceptance target is 2/3/3/3. These count full-depth index
updates, not logical Heap work, allocator calls, CPU time or RSS. Existing Delta
coefficients and work must remain unchanged.

Tests compare full returned arenas, handles, every Delta field and exact error
constructors, including transient incoming overflow and preflight precedence.
Parent integration owns the full locked-SDK gate and existing cross-executor
instrumentation; no timing or memory claim follows from this structural change.

## Guard-preserving algorithm

`allocateMoved` keeps the old release check followed by
`preflightMovedRoots`. `allocateMovedChecked` uses the exact `allocate` guard and
parent-insertion tree, substituting two calls to `attachMovedChild` and the
algebraically identical moved-root/work Delta fields. The source gate restores
only those deliberate substitutions and compares the entire remaining semantic
body against `allocate`; it separately checks the complete release/preflight
outer tree against the old `allocateMoved` composition.

A nonnull moved child checks the old transient incoming increment, then writes
its original incoming count and roots minus one in one index update. A second
child that aliases an earlier moved child virtually restores the earlier retain
before checking its own retain increment. It therefore rejects both a single
moved child at MAX incoming and two moved aliased tickets at MAX-1 incoming. A
second unmoved alias performs the same virtual check and writes its net incoming
increment once. Ordinary unmoved children continue through unchanged
`retainEntry`. Null children produce no child index update.

The helper introduces no eager drop error. The unchanged preflight establishes
`incoming >= roots >= required tickets` for every moved handle. After successful
old retains this implies the old drop checks: nonzero roots and incoming at least
two. For aliases, requiring both tickets before allocation also establishes the
second drop's guards. A forged first moved entry with incoming zero and roots one
is rejected by the old preflight with `countUnderflow`, even if the second child
would overflow during retain. The new path preserves that precedence. The tests
also cover first-retain success followed by a second-child overflow, missing move
tickets versus an exhausted budget, reservations, next-ID collision/exhaustion,
owned/live count overflow, malformed node shape and foreign release.

`allocate`, `dropRetainedRoot` and `preflightMovedRoots` remain byte-identical.
Moved roots are still represented in `Delta.droppedRoots`; `Delta.work` remains
one allocation operation plus one operation for each requested root drop,
including a requested drop of a null handle. Entry weights and owned-byte budgets
are unchanged. This removes index updates, not modeled ownership operations.

## Qualification

The isolated fixture `/workspace/heap-moved-fixture` uses official language-1.2
source build and pinned Lean 4.32.1, without changing product locks or rebuilding
the compiler. Its official build is
`4cf1df6ebb5b35e23d3543f0e667b34c35211946569ba68369e6b94f10ba3b32`.
The original source gate fails with 3/5/5/4 versus the target 2/3/3/3; the restored
source passes. The gate parses the actual source through the official SDK, checks
its byte-exact notation round trip, binds its own source hash, and checks the
unchanged guard trees and called update sites. It is a structural source probe,
not dynamic allocator telemetry.

`HeapMovedReference.oldAllocateMoved` is the exact previous source composition,
with only qualification, diagnostic name and executable-policy changes.
`sameResult` compares the entire fixed64 Index recursively, every Node and runtime
value field (including image Bytes), all arena counters and release-cursor fields,
the returned handle, every Delta field, and the exact error constructor.
`HeapMovedTests` contains five diagnostic groups, including 192 explicit
alias/distinct/move-mask/counter combinations and near-MAX counter controls.
Valid ownership cases additionally run the independent `HeapAudit.auditRoots`.
Forged-counter cases compare errors/outputs without calling them valid arenas.
Both new modules are diagnostic (`executable: false`) and outside the product
entry closure. Actual Heap cursor/allocation helpers stay executable.

All five groups pass pinned Lean kernel compilation and VM assertions. Actual
axiom traces for both new helpers, `allocateMoved`, the old reference and the
suite are exactly `[propext, Quot.sound]`.

The same byte-preserved official generated Lean files were compiled through
Lean's C backend and pinned `leanc -O2`, linked with the retained official SDK
Portable runtime and an assertion-only main, and executed as
`/workspace/heap-moved-fixture/native/heap-moved-tests`. All five groups pass in
that native executable. This is native Lean/C diagnostic acceptance; it is not
Rust/export or locked-SDK release acceptance. The build/run commands and hashes
are retained in `native-build.log`, `native-execute.log` and the adjacent JSON
evidence file.

Two isolated source mutants were built by the official CLI, kernel-compiled,
compiled to fresh native C objects together with the reference/tests, and linked
before their assertions ran:

- Omitting the moved root decrement is rejected by `basicScenario`, with
  `planted missing-drop defect rejected`, native exit 1.
- Bypassing move preflight is rejected by `preflightScenario`, with
  `planted missing-preflight defect rejected`, native exit 1.

Receipts are under `/workspace/heap-moved-mutant-missing-drop` and
`/workspace/heap-moved-mutant-missing-preflight`. These mutants never modified
production source. The retained unmodified native executable was rerun after
both defects and passed; the source gate also passed again. The JSON evidence
binds source, generated/runtime inputs, runner and receipt hashes.

The parent's original e87 dynamic index-write counter belongs to another
executor; its harness/trace files were not transferred here. Its rerun against
the new Heap hash remains a cross-executor integration check. The numbers above
are source-qualified structural counts, and no new admission/affine trace total,
timing, allocator or RSS result is claimed. Full locked-SDK `just vv`, fresh
parent IR publication and native Rust/export acceptance remain integration gates.

The unchanged existing `HeapTests.suite` also passes official source linking,
pinned kernel compilation and VM execution against the new Heap. Its separate
receipts are `build-regression.json`, `HeapTests-kernel.log` and
`regression-execute.log` in the scoped fixture. No other worker source was edited.
