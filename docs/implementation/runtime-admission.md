# Owned program admission interfaces

The source-owned components in `src/Uorc/Core/Runtime*.lex.tex` separate syntax, typed admission, immutable metadata construction and the eventual admitted-session boundary. The module sources, rather than this integration guide, define their behavior.

## Trust and ownership

`RuntimeProgram.Ready` is a fixed carrier of an arena, program/basis tickets, caller root partitions and explicitly byte-denominated usage. A record is not an unforgeable proof token. The untrusted-input product path must obtain it by completing actual admission, not accept a caller-supplied record, Boolean validation flag or saved cache as evidence. A trusted machine API needs the concrete admitted-session contract. Reader failures remain typed for malformed metadata and references.

Program and basis roots are only two partitions. Input, archive, wire, search, checkpoint, certificate and verification owners stay separately inventoried. Repeated handle IDs are repeated tickets, never a set deduplication rule. A borrowed literal span stays backed by its charged wire/image owner until its event is acknowledged and any new sequence leaf has retained that image. A completed production admission must not retain a parallel mathematical graph, Rope or Store outside this inventory.

## Bounded source components

- `RuntimeAdmissionOrder.start/advance` checks event order and counts only. It is not typed validation or a Ready producer. Scan's observer capture lane must finish before its updater reference scalar. Literal spans consume the declared length exactly; impossible span totals are rejected without wrapping arithmetic. Completing this preflight is insufficient to acknowledge an event before its typed checks and owned mutations finish.
- `RuntimeAdmissionChecks` resolves typed local distances and prior-block distances; checks immediate domains; checks Apply/Tabulate/Fold/Scan parameter prefixes, results and capture parameter types; and checks return and entry signatures. Any of the five value types can be a loop state. Zero iteration counts do not skip static checks.
- `RuntimeInstructionCheck.start/scalar/finish/captureContract` stores four fixed scalar positions, result/state types and at most two borrowed capture contracts. Each scalar returns an optional absolute local slot for a separately performed last-use write. Scan's sequence result does not overwrite its arbitrary state type. Choice branches must have the same type.
- `RuntimeLifetime.start/step` reads one indexed last-use cell per planner event. Unused parameters enter the initial release bucket. A produced value with no later use enters its creation instruction's post-instruction bucket. Recorded uses are checked against creation and body bounds. The return slot remains held until the frame root is dropped. Timestamps are zero-based instruction indices, not `n + 1` sentinels.
- `RuntimeRecordBuilder.start/step/rootInventory` replaces one owned fixed-depth metadata record. It keeps the old ticket until the replacement path is complete, then performs explicit resumable arena release. Failed writes retain the actual partial arena and temporary root inventory. No detached release cursor can become stale.

These components are being composed into actual streamed admission. They do not by themselves establish that the complete production admission, execution and archive pipeline has been qualified.

## Metadata ABI

`RuntimeMetadata.kindCode` defines internal kinds for block index, block record, instruction index, instruction record, lifetime index and release bucket. These identifiers are not archive wire opcodes. `RuntimeOpcode` owns 39 normalized symbolic operations; the two mathematical literal constructors normalize to the same charged literal operation.

A block record has fields 0 parameter count, 1 result type, 2 instruction count, 3 return distance, 4 parameter type table, 5 instruction index, 6 post-instruction lifetime index and 7 initial release bucket.

An instruction record has field 0 normalized opcode; fields 1–4 its required scalar operands; fields 5–6 declared capture counts; fields 8–9 capture tables; and field 10 a literal sequence handle when applicable. Unused fields are absent. Empty required sequence/table links are explicitly null. Readers never turn an erased in-range runtime slot into a default natural value.

A release bucket has field 0 count and field 1 its indexed absolute-slot table. A missing post-instruction bucket means no releases. `RuntimeProgram` readers validate record kind, tag, field type and range instead of trusting cache claims.

## Evidence distinction

The modules contain executable scenario owners and general theorem candidates. Scratch interpretation of actual source terms is diagnostic, not official compiler, kernel, generated-runtime or locked-SDK evidence. Planted defects have been checked against the same source owners for operand typing, Choice arm equality, missing versus timestamp-zero uses, premature Scan captures and omitted old-record reclamation. `RuntimeRecordTests` independently checks the actual heap inventory at each intermediate cursor state. Official qualification and complete release acceptance remain separate requirements.

`RuntimeFixtures` is test-only source construction over the resumable table API. Its fixed diagnostic driver does not create a product-level input-size or event-budget limit, and must not enter the production decode closure.
