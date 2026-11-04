# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: real-execution

  @UC-REAL-01 @open
  Scenario: A real nonliteral archive is independently parsed, executed and compared against every original byte before encoder publication.
    Given generated production executables, structured and unstructured binary corpora, exact source/SDK identities and baseline commands
    When compression produces the complete archive and an independent decode reconstructs all ordered objects
    Then byte equality and archive sizes are recorded from actual execution; finite fixtures, source simulation, performance measurements and mathematical proofs remain separate evidence
