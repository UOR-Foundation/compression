# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: archive-integrity

  @UC-INT-01 @build
  Scenario: Every archive the client decodes is checked against its UORC-SHA256-CHAIN-v1 integrity trailer, so changing any single bit of an archive makes it fail to decode.
    Given a program archive and a raw archive, each with every byte altered in its low and in its high bit
    When uorc decompress reads each altered archive
    Then every altered archive returns a decode failure and none yields output

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
    When ordered source transcripts, undrained-span EOF, cancellation, cumulative basis/main work and late-corruption controls execute through generated source models
    Then only actual ordered source byte admissions and drained boundaries qualify; failed prefixes remain charged once, exhausted counters fail without reset, and corrupt late bytes reject with owned cleanup

  @UC-INT-05 @build
  Scenario: Source-owned completed-view and continuous publication proofs derive the actual chosen-profile digest and emitted-image reads, discharging canonical driver termination and OwnedHash control/cleanup lifetime for both explicit format identities and arbitrary finite source input. Initial source-layout correspondence and full product acceptance remain separate consumer obligations.
    Given actual completed ordered views, continuous actual finish/publication transitions and source-derived profile progress and lifetime laws
    When the composed source theorem, all-declaration audits, exact source mutations and restored generated-model controls are replayed
    Then canonical digest equality and emitted-image reads follow without caller-supplied hash success or preservation flags; source-layout and full byte reconstruction claims require their own consumer proofs
