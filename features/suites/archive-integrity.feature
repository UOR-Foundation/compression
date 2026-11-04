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
  Scenario: Finite reverse-trace candidates remain untrusted until actual successful SHA computations independently validate a collision witness; generic acceptance-guard reflection does not claim global SHA injectivity or reduction completeness.
    Given finite framed reverse traces, an untrusted candidate extractor and an independent successful-SHA candidate checker
    When the generated source controls exercise malformed and non-collision candidates and a planted false digest-equality result
    Then malformed, failed-SHA and unequal-digest candidates reject; the planted equality defect fails its owning observation and restored controls pass; no global SHA injectivity or trace completeness claim follows
