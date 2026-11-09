# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: conditional-basis

  @UC-BAS-01 @open
  Scenario: Conditional archives bind the actual supplied ordered basis using a source-owned length-delimited domain-separated identity, and execute against that admitted basis. Open: the shipped client writes and reads self-contained archives only; conditional-basis archives are not shipped yet.
    Given actual immutable basis roots, altered bytes, reordered roots and inserted empty objects
    When basis admission derives its identity and conditional archive validation checks the stored identifier
    Then identity is never accepted merely because a caller supplies a matching tag; reconstruction is relative to the actual supplied basis and cryptographic binding assumptions remain explicit
