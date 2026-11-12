# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: integrity

  @UC-SHA-01 @build
  Scenario: Incremental SHA-256 and the separately named bounded chain have source-owned streaming semantics, typed failures and independently locked byte-oriented CAVP oracle evidence.
    Given the complete locked NIST short and long SHA-256 response files and the source-owned integrity model
    When the gate verifies the source, executes all 129 cases and 2919 partitions, exercises protocol errors, and plants arithmetic and length defects
    Then every expected byte digest agrees, malformed or incomplete corpora fail, the owning observations reject the source defects, and restored controls pass

  @AUTH-NIST-SHA256-SHORT @some-true
  Scenario: The locked NIST CAVP SHA-256 short byte-oriented response data supplies 65 independent expected digests, reproduced as finite oracle checks.
    Given the original response bytes attributed to NIST and acquired through the pinned public mirror
    When the complete importer and source-generated integrity runtime execute the locked short corpus
    Then all 65 declared digest cases are reproduced without promoting informal test evidence into CAVP certification or a universal proof

  @AUTH-NIST-SHA256-LONG @some-true
  Scenario: The locked NIST CAVP SHA-256 long byte-oriented response data supplies 64 independent expected digests, reproduced as finite oracle checks.
    Given the original response bytes attributed to NIST and acquired through the pinned public mirror
    When the complete importer and source-generated integrity runtime execute the locked long corpus
    Then all 64 declared digest cases are reproduced without promoting informal test evidence into CAVP certification or a universal proof
