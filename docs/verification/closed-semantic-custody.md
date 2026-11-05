# Closed archive semantic custody diagnostic

`Uorc.Wire.ClosedSemanticScenarios.run` owns
UC-WIRE-CLOSED-SEMANTIC-01 through -05. The final five-scenario source owner
passes, rejects an incoming-ledger-reset mutant with typed `Error.prefix`, and
passes after exact restoration. The scenario is identical across those three
runs. The accompanying JSON binds all 58 projected modules, generated Lean
hashes, raw source hashes, compiler hashes and command templates.

The runtime controls exercise four narrow paths:

- `beginWithSemantic` receives an actual `RuntimeMachineSemantic.commit`
  byte-read prefix whose `semanticWork` is 1, and exposes that value unchanged.
- With semantic work limited to 1, actual `observeSemantic(byteComparison)`
  refuses and preserves that accepted value. `fundSemanticWork(1)` followed by
  the same observation succeeds once, exposes work 2 and clears the refusal.
- Cancellation before parsing completes the actual custody audit and cleanup,
  then `takeClean` returns the semantic session with work 1 and an Arena whose
  roots audit empty. A second `takeClean` is retained. The moved State has the
  actual `Owner.handedOff` constructor, which carries usage without an Arena.
- An entry budget allowing 100,000 owned bytes disagrees with the actual Arena
  budget of 1,000,000. Entry must return `Failure.resource`, and the original
  archive ticket must still audit live in the caller's inventory. The negative
  matches the resource constructor; it does not separately match its subtype.

The first three controls inspect the nonzero semantic-work prefix. They do not
independently perturb every field of the semantic ledger. Scenario -04 records
source compatibility with actual Product `ReadyTransfer` and
`semanticSuspended`; this owner does not qualify general Machine runtime
controls or decoded-output transfer.

The entry guard uses existing `RuntimeResources.observeArena` before
`ArchiveSnapshot.begin`. It maps an error to `Failure.resource` before ticket
transfer and installs the actual observed Usage on success. It introduces no
second arena validator. Scenario -05 was registered before this production
change. The unchanged five-scenario owner failed before the guard in build
`99e16f490d2b69ee64bac0a7060937aafb38a6aa9344806969453039ea98527a`:
the typed run exits 1 with `ClosedSemanticScenarios.run: fail refusal`.
With the guard, the same owner passes. This closes the budget-mismatch entry
path that formerly reached permanent framing cleanup refusal.

The dependency authority is Product.MachineBinding
`8fff69085cd89e15285df26af5508e33dbc5e4c3`, RuntimeMachineEngine,
RuntimeMachineSemantic and RuntimePeak
`7cf135da579abb7f3012184ccc9c253496259f99`, and CanonicalCost
`29d25233a2565157dc0bd39acc497f620c567788`. Their exact raw hashes are in the
receipt. The final copied ClosedArchive source hash is
`97dc8db7c6bcb7d6c8e647639e41fb77c39281a8414a0e34197e30c1ef3297e7`;
the five-scenario source hash is
`40d4203bea19923c3186f34207d33b20340bdf0879f4894c65a0aa1925b01475`.

The final green build is
`5964bbd83e55243f8ebbe0dcf374f53300e056f690b757e9619733d4aa36381b`.
The disposable mutant changes only the successful `beginWithSemantic` record's
`SemanticSession.ledger` from its incoming ledger to actual
`RuntimeMachineSemantic.zeroLedger`. Its build is
`8404e2adb9a085fed0ac32d3b71ccf8ec1f58664bf306f3bf69b9818a12e674a`;
the unchanged typed owner exits 1 with `ClosedSemanticScenarios.run: fail prefix`.
Canonical AST comparison confirms one changed record field and all 52 controller
declarations retained. Restoring exact projected bytes gives the original final
green build ID and native exit 0 again. The mutant was confined to a disposable
project; this diagnostic lane did not edit production source.

The earlier four-scenario checkpoint remains historical evidence: green build
`e087eaf9b8e10568e755bf1d97b4095edc4d44a899fe5332d9553787fb904fd1`,
ledger-reset mutant
`f31fae57e8ed3ad59bfb4b00a2298482c04f30d4ed774662a3519be795a5c80b`
with typed `prefix`, then exact restored green. It does not qualify the new entry
guard. The receipt preserves those source/projection hashes and results.

Earlier failures are preserved as fixture evidence, not as the semantic mutant
gate. Build `45c8035f08554e7e52829469ad7465c8a59e65917d7d970277891e215f9622fa`
passed source checking but all three native cases returned `fixture`: the
zero-length `SS.setupBytes` fixture failed before controller entry. It now uses
one byte `0x00`, cancelled before parsing. Build
`3a5b97e13480067368d080abd4873407fbf8a2a35cbc5df537c83e3c46e6be5b`
then passed entry and refusal, but cancellation exhausted test fuel. The fixture
budget allowed 100,000 owned bytes while its actual Arena allowed 1,000,000;
`ArchiveParse.prepay` returned `arenaBudgetMismatch`. At tick 10,000 custody was
validated, release was inactive, and cleanup remained in `cleanupFrame`.
Matching the fixture budget allowed the intended semantic controls to run.
The new entry negative then demonstrated that mismatched budgets also required
an early production refusal. Test `run` propagates its original typed errors,
so the ledger mutant reports `prefix` rather than generic `fixture`. Scenario
statements and assertions remain present.

The diagnostic removes imported theorem declarations and slices unused imported
declarations. It preserves all declarations in ClosedSemanticScenarios,
ClosedArchive, ClosedArchiveContract, ArchiveSnapshot and ArchiveIntegrity, and
adapts source notation from language 1.1 to 1.2. All 58 projected modules pass the
existing candidate compiler's check/build. Native dependency cache reuse is bound
to equal projected dependency hashes; controller and scenario modules are
recompiled for this replay. Generated product code comes from source `.lex.tex`
modules. Concrete private artifact locations are recorded separately.

This is narrow candidate/native custody evidence. It is not `just vv`, locked
SDK acceptance, a production export, a general Machine control qualification,
or a physical-memory refinement. General permanent framing-failure propagation
through `ArchiveSnapshot.cleanupFrame` is outside this control. Retained-source
custody, layout correspondence and whole-stream EOF proofs retain their existing
owners and open obligations.
