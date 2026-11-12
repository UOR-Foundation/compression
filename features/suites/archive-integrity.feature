# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: archive-integrity

  @UC-INT-01 @build
  Scenario: Every archive the client decodes is checked against its UORC-SHA256-CHAIN-v1 integrity trailer, so changing any single bit of an archive makes it fail to decode.
    Given a program archive and a raw archive, each with every byte altered in its low and in its high bit
    When uorc decompress reads each altered archive
    Then every altered archive returns a decode failure and none yields output
