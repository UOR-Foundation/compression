<!-- uorc:non-authoritative -->

# Resumable runtime table updates

The source-owned capability `runtime-table-cursor/v1` is implemented in
`src/Uorc/Core/RuntimeTable.lex.tex`, with executable scenarios in
`RuntimeTableCursorTests.lex.tex`. It supplements the existing table API. The
atomic `put` and recursive `writePath` remain diagnostic conveniences with
`executable: false`; they remain available to pinned Lean VM diagnostic tests.
`put` no longer carries a production-root annotation. Production callers use the
cursor API because a later allocation failure must preserve the achieved arena,
ownership tickets, cumulative cost and peak.

## API and transition contract

- `PutLeaf.slot(role, RuntimeValue)` and `PutLeaf.metadata(kind, key, target)` are
  the only admitted write payloads. A saved cursor never contains `Heap.Node`,
  `Bytes`, a mathematical rope/trie, or a reference graph.
- `beginPut(arena, table, key, leaf) : Result PutTransition HeapError` validates
  the kind/tag/payload, reserves control storage, and captures the original table
  root and the leaf's handle target. There is at most one arena allocation in
  this call. Pre-mutation failures return `Result.error`; a capture failure after
  reservation returns `Result.ok` with `PutOutcome.failed` and the reservation.
- `beginErase(arena, table, key)` has the same result type. Its saved leaf is
  `Option.none`, so erasure never creates a replacement value or fake slot.
- `step(state) : PutTransition` performs one bounded cursor event. Descent reads
  one borrowed branch, leaf insertion allocates one node, and ascent allocates
  at most one parent. A release event starts release or executes exactly one
  `Heap.releaseStep`. There is no drain loop in the production closure.
- `cancel(state) : PutTransition` selects cancellation cleanup. Subsequent
  `step` calls finish it. Cancellation can begin before descent, after an
  allocation failure, during ascent, or during this cursor's release drain.
- `ownedRoots(state)` returns the cursor's external tickets, preserving
  multiplicity. Append these to the caller's independently maintained inventory
  for `HeapAudit.auditRoots`. The Heap release cursor inventories its transferred
  root itself; `ownedRoots` excludes that ticket after transfer.

`PutTransition` contains the current `arena`, optional `cursor`, `outcome`,
cumulative `delta`, and cumulative `peakOwnedBytes`. `PutOutcome.failed(error)`
is an ordinary transition retaining all of them. Every returned state replaces
its predecessor; old states are not additional ownership tickets. A caller may
cancel or retry a failed state. `delta` is cumulative since begin, so charge its
increase since the preceding observation, and charge `peakOwnedBytes` to the
machine's Usage peak. Descent performs no ownership operation; Heap delta work
continues to mean logical ownership work, not instructions or elapsed time.

`PutOutcome.complete(table)` transfers exactly one nonzero result root ticket
(or the empty root, which has no ticket), and clears the cursor. Add that root
to the caller inventory. `PutOutcome.cancelled` clears the cursor without
transferring a result. Calling `step` or `cancel` on a terminal state reports
`protocolFailure`; terminal results must already have been consumed by the
caller.

Raw `lookup` returns the null handle for an erased or missing key. Frame/store
integration must preserve logical slot count separately: an index below that
count with null lookup produces its typed released-slot error. Erasure does not
renumber indices or decrement that logical count.

## Fixed continuation storage and ownership

`PutPath64` contains exactly eight `PutPath8` records, each with eight handle
fields. It is a real 64-field record, not an unbounded List backing disguised by
a bound. Depth and key traversal use fixed-width scalars. Siblings remain
borrowed under the captured old root during all 64 MSB-first descent events.
The single new subtree ticket is moved into each ascending parent using
`allocateMoved`. The old caller ticket remains independent of the cursor's
capture, so aliases and captures survive cancellation.

`Heap.allocateControl(arena, 1024)` allocates `imageReservation(control, 1024)`
for fixed cursor control storage. It never receives a Bytes payload and cannot be
committed as an image. Its explicit control provenance permits checkpoint aliases
and owner edges while preserving the same accounting. The declared logical reservation is `48 + 1024 = 1072` bytes. The
1024-byte control allowance budgets 512 for the 64 sibling IDs, 16 for table kind,
8 for key, 48 for the optional typed leaf, 16 for depth/remaining key, 48 for the
six handle fields, 16 for phase/cancellation, 72 for cumulative Delta, 8 for peak,
and 280 for discriminants, transition outcome and fixed control padding. This
is an explicit logical packable representation, not an assertion about generated
Rust layout, allocator storage or process RSS. The saved Arena is the machine's
arena state, not a second payload owner. Helper root lists are inventories, not
saved path storage. No table-cursor metadata schema is allocated in another
worker's reserved metadata kind range.

Cleanup releases the optional new subtree on cancellation, the leaf capture,
the table capture, and finally the control reservation. A transferred release
root stays charged in the Heap until processed. The final control release step
simultaneously clears the fixed cursor and returns its terminal outcome: there
is no returned state containing an uncharged path record. The result subtree is
kept owned during successful cleanup, before transfer to the caller.

Release-start and release-drain are distinct phases. If another owner starts
release while this cursor is descending, ascending, or starting cleanup, `step`
returns `releaseInProgress` and preserves the foreign cursor. It never invokes
`releaseStep` from those phases. The machine serializes arena transitions and
must not splice an unrelated arena into a saved drain state.

## Erase shape

Erase validates the borrowed leaf, sets the ascending subtree to null, and
collapses a branch only when both children are null. A single surviving child
retains its branch at the same depth. Every surviving parent is allocated with
`allocateMoved`; a later failure reports the already allocated subtree and its
cost exactly as for insertion. Captured old roots are released only after the
new root has been installed in the cursor's owned subtree field.

## Evidence and limits

The isolated language-1.2 fixture is `/workspace/table-cursor-fixture`; it uses
the official `/workspace/LexLean/target/debug/lexlean` CLI and pinned Lean 4.32.1.
It leaves the product's old SDK/language-1.1 locks unchanged. Scoped evidence
includes official source build, generated Lean compilation and executable source
assertions. `RuntimeTableCursorTests.run` audits explicit outside plus cursor
roots at every event, checks the achieved peak, and rejects more than one new
arena entry per step.

Scenarios cover slots at keys 0/1, metadata at UInt64.max, wrong leaf kind,
malformed branch shape, foreign release, empty erasure at UInt64.max, successful
erasure with an old readable table, and both write/erase failure prefixes. The
write budget allows control plus leaf plus three parent allocations: the failed
transition reports five entries and 1296 allocated bytes. Cancellation returns
to the prior live byte total while two external table aliases and the metadata
target survive. The erase budget allows control plus three parent allocations:
its failed transition reports four entries and 1240 allocated bytes, then
cancels to the prior total.

Full locked-SDK `just vv`, native Rust/export acceptance, the shared source
registry/ledger integration, and the composed machine Usage integration remain
parent integration gates. The known export-B mismatch is tracked by
LexLean #310 / PR #393; this lane adds no compiler workaround or handwritten
runtime. Scoped source/kernel execution is not full release acceptance.

The control-backed fixture build before diagnostic-only metadata changes was
`b9496e30add56fef5b9bd56ee8b974e348002fb1595cc2968c2b6f8ebdf13c8b`.
`cursor-execute.log` records all seven scenario passes and the exact axiom traces:
`beginPut`, `beginErase`, `step`, `cancel`, and the scenario suite use exactly
`[propext, Quot.sound]`; `ownedRoots` uses `[]`. `regression-execute.log` records
the independent heap, table and index suite passes. The Heap source SHA-256 is
`8f3880d42cc0294356bfbee1aa18e7e95fa4f9bfdcda63199523a366d304ae05`, matching
the parent's stable foundation.

The two isolated planted defects are recorded under
`/workspace/table-cursor-mutant-prefix` and `/workspace/table-cursor-mutant-bit`.
Both were built by the official CLI and compiled with pinned Lean before their
assertions ran:

- Replacing cumulative `putAddDelta` with the latest event's delta makes
  `failurePrefixScenario` reject the state: `planted prefix defect rejected`,
  exit 1.
- Replacing cursor descent `highBit` with `lowBit` makes `slotsBitsScenario`
  reject the wrong lookup result: `planted bit defect rejected`, exit 1.

The unmodified control-backed source was retained in the main fixture, and its
complete seven-scenario rerun passed. These mutations never touched production
source. Reproduce a scoped assertion with:

```sh
LEAN_PATH=/workspace/table-cursor-fixture/diagnostic-oleans \
  /workspace/.setup/elan/toolchains/leanprover--lean4---v4.32.1/bin/lean \
  /workspace/table-cursor-fixture/ExecuteCursor.lean
```

The fixture's `build-cursor.json`, `generated-root.txt`, per-module kernel logs,
and assertion logs separate compiler linking, kernel compilation and execution.

After marking `put`/`writePath` non-executable, the final explicit cursor closure
build passed with no diagnostics:
`414ecf63764e9c24c7752ccccf391ab3940dc75ad3ea7d262101702dc0242883`.
Its `Heap`, `HeapAudit`, `RuntimeValue`, `RuntimeTable`, and cursor-test generated
Lean files are byte-identical to the compiled and executed control-backed build
above; executable metadata changed without changing any generated implementation
bytes. The final receipt is `build-final-cursor.json` in the same fixture.

Worker snapshot SHA-256 values (before canonical formatting and the composed exact-axiom metadata correction; the current compiler-parsed cache binds the newer source bytes):

- `RuntimeTable.lex.tex`:
  `daef4c6cd618483365ef6bb356d69e6600449568ebc8f8b511f89178a40a0f0b`
- `RuntimeTableCursorTests.lex.tex`:
  `481926ec61c492935c59054880a7f1a3577098b34080d56ba28b47d88bd856a5`
