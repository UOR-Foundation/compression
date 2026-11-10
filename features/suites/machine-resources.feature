# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: machine-resources

  @UC-MACH-02 @build
  Scenario: Decoding is charged against an explicit decoder work budget, and exhausting the budget is a typed refusal with no partial output.
    Given a program archive and decoder work budgets below and above its cost
    When uorc decompress runs under each budget
    Then the small budget returns a decode failure and no bytes, and the sufficient budget returns the exact original
