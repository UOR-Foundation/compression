# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: archive-integrity

  @UC-INT-01 @open
  Scenario: Archive and basis integrity use an explicitly identified bounded streaming construction with standard SHA-256 components; digest agreement never replaces exact byte comparison.
    Given source-owned integrity identities, locked independent vectors, split updates, finalization and a planted arithmetic defect
    When the actual source checks the vectors and the archive path hashes the received preimage
    Then standard-domain boundaries and any separately named composition are explicit; invalid protocols fail, mutations are caught, and no cryptographic assumption is reported as a kernel theorem

  @UC-INT-02 @build
  Scenario: Format3/machine1 explicitly binds STREAM-MAX-v1 to charged image spans, preserves format2 CHAIN-v1 semantics, and seals input, basis and checkpoint identity domains without replacing exact reconstruction comparison.
    Given the explicit integrity profile map, charged ownership transitions and independent byte fixtures
    When source-generated runtime controls, actual reconstruction and planted integrity defects execute
    Then profile and ownership mismatches reject, chunking preserves digests, exact bytes reconstruct, and restored controls pass

  @UC-INT-03 @build
  Scenario: Source-owned checker soundness and constructive finite backward reduction are kernel checked for actual canonical framed messages and shape-valid source-SHA-authenticated traces, without global hash injectivity or an assumed frame codec. The decoder-to-authenticated-trace bridge and full product acceptance remain distinct.
    Given the byte equality bridge, actual candidate checker, canonical framing and finite source-SHA-authenticated reverse traces
    When the saved official component compiler verifies and replays the scoped proof source, then digest-guard, returned-witness and counter defects are planted and restored
    Then the scoped proofs and exact axiom policies pass; actual false checksum/witness/counter behavior rejects at new proof owners; restored attestations agree without promoting this to original pinned product acceptance

  @UC-INT-04 @build
  Scenario: Source-owned actual-call relations establish ordered hash admissions, drained selected-view EOF and checked cumulative basis/main work; composed generated-model controls reject failed-prefix resets and late corruption. Consumer snapshot preservation, canonical digest refinement and complete archive EOF remain separate obligations.
    Given retained immutable input snapshots, actual producer spans and charged hash operations
    When ordered source transcripts, pending-span EOF, cancellation, cumulative basis/main work and late-corruption controls execute through generated source models
    Then only actual ordered source byte admissions and drained boundaries qualify; failed prefixes remain charged once, exhausted counters fail without reset, and corrupt late bytes reject with owned cleanup
