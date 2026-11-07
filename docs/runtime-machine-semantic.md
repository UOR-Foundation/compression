<!-- uorc:non-authoritative -->
# Logical-event ledger and charged cache components

`RuntimeMachineSemantic` is bound to the distinct
`uorc.logical-reference-events/1` authority imported from core commit
`06d5e1c9229bab88e3238670a49c99955d92f452`. `CanonicalCost.referenceValueUnits`
is the single mathematical payload measure. `CanonicalBindings` and
`CanonicalSchedule` supply binding and event contracts. This authority remains
ordinary-proof unqualified. These components do not establish full engine event
coverage, cache/reference refinement, semantic admissibility, or a minimum.

## Profile compatibility

Both `Ledger` and `Budget` carry the actual `CanonicalCost.ProfileId` enum.
The fixed-width `StoragePolicy` distinguishes unconstrained, payload units, and
obsolete payload bytes. `checkedAdvance` rejects an obsolete ledger profile;
`charge` additionally rejects an obsolete budget profile and byte-named scope.
The typed errors are `unsupportedProfile` and `unsupportedStorage`. These checks
happen before any commit; failed `commit` preserves the exact achieved prefix.
There is no public unchecked charging helper that bypasses compatibility.
Old `canonicalReferenceWork1` evidence is rejected, not relabeled. A fresh replay
is required for the new identity even where numerical fees remain equal.

## Fixed-width accounting

`Event` has the sixteen logical event forms; allocation and release carry
`UInt64`. `Ledger` carries eight `UInt64` counters plus its fixed profile tag.
Checked arithmetic precedes commit. `wordOverflow` denotes operational inability;
semantic work/storage limits and invalid evidence remain distinct typed errors.
At the runtime boundary, `isOperational` classifies overflow, unsupported
profile/scope, invalid usage, and storage/frame underflow as incomplete or
protocol-invariant failures. Only work/storage limits are potential canonical
refusals, and only after a full bridge is established. Cache errors are likewise
operational or invariant failures, never semantic predecessor exclusions.
`RuntimeWork` operational fees and `Heap.Delta` ownership are separate measures.
There is no retained mathematical usage or event list. `meaning`, `eventMeaning`,
and `cacheWitnessMatches` are reference-only interfaces.

The instruction-fee realization agrees with `CanonicalCost` for every opcode by
a compiled source theorem. Operand events must follow all `RuntimeOpcode.spec`
roles, including block references, through `CanonicalSchedule.instructionPrefix`;
no copied operand-count schedule is introduced. The engine bridge is still open.

## Charged per-binding caches

`emptyUnitsTable` is a native `RuntimeTable` with slot role `typeCache`.
`beginBind`, `beginErase`, `stepBinding`, and `cancelBinding` delegate actual
resumable table transitions, carrying their real arena, ownership delta, peak,
and charged cursor. `unitsAt` reports typed missing/invalid/heap errors; missing
units are never replaced with zero. No second arena or whole-store traversal is
introduced.

`fixedValueUnits` returns eight for naturals, one for bytes and predicates, and
no fixed answer for sequences or stores. Three general scalar theorems agree
with the actual core reference measure for arbitrary scalar values. Dynamic
sequence units require logical cached length; store units require a maintained
witness. `initialStoreUnits` is one.

`storeUnitsAfter(previousUnits, defaultByte, observedOld, newByte)` adds nine on
insertion, preserves units on replacement, and subtracts nine on removal. It
checks the numerical shape one plus a multiple of nine. A claimed one-unit store
with an observed nondefault old value is rejected as `CacheError.invalid`, for
both replacement and removal. The observed old byte must come from the actual
store cursor. Numerical shape does not prove cache equality to reference values.

Engine integration must charge and inventory units-table roots and preserve
witnesses through aliases, choose, parameters, accumulator transfers, and return.
The cache-to-reference relation remains an explicit core-owned obligation.

## Fresh finite evidence

All ten registered controls passed through the **generated Lean evaluator**
against the new profile: the previous eight ledger/cache controls were replayed,
plus old-profile/scope rejection and fixed scalar-size controls. They include
actual native table operations in the generated Lean model, not accepted Rust
execution or native physical-resource certification. The fee control independently
uses `CanonicalCost`; a block-reference control uses `CanonicalSchedule`.

Every remaining definition and theorem in both modules has an exact declared
axiom policy checked against generated Lean `#print axioms` (71 declarations).
Only the two general ledger/fee proofs and three general scalar-measure proofs
are retained; unused field-projection lemmas were removed earlier.

The isolated 37-module diagnostic dependency projection was prepared from actual
source data and exact-hash official parser-cache entries, then migrated with the
official compiler. Exact Heap and both new source modules were overlaid.
Fresh build: `4735a146494488d7b68d0c430124e3d83e37757e4a1e99ff4c54d6ed94561369`.
Its portable evidence bundle records source/build-input hashes, the full source
root closure, policy observations, and evaluator outcomes. A planted profile
compatibility bypass is rejected by `semanticProfileCompatibility`; restored
exact-source controls pass. This is diagnostic finite evidence, not full locked
SDK `just vv` acceptance.

The boundary classification correction was registered before implementation.
With the old helper, the strengthened existing
`semanticLedgerCanonicalBudgetDistinct` control failed; after correction it
passes and explicitly checks all eight error variants.

Execution in this evidence is the generated Lean evaluator. No accepted Rust
package, Wasm result or physical memory measurement is asserted.
