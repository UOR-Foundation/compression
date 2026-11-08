# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: checkpoint-replay

  @UC-CHK-01 @open
  Scenario: Imported checkpoints are untrusted replay requests, not authority for a queue, incumbent, coverage, source identity or resource receipt.
    Given a saved checkpoint plus current full-byte source, basis, model and policy snapshots
    When source-owned checkpoint validation compares actual data and reconstructs the admissible replay
    Then mismatches fail or require fresh work; stale digests and serialized coverage cannot manufacture verification or a minimum certificate
