# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: machine-semantics

  @UC-MACH-01 @build
  Scenario: The client's reference-semantics evaluator decodes program archives, including search-found programs and the ulz kernel graph when the certified fast path declines, to their exact original bytes.
    Given a 64-byte program archive found by the exported search and a ulz kernel archive whose padded payload the fast path declines
    When uorc decompress evaluates each program graph by the reference semantics
    Then the output equals the original bytes exactly
