# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: complete-synthesis

  @UC-SYN-01 @build
  Scenario: Program search under a step budget keeps a candidate only as a fully checked archive: more effort can change which archive wins, never whether the result decodes to the input.
    Given an input and search budgets of 0, 10000 and 200000 steps
    When uorc compress runs the search at each budget and each archive is decoded
    Then every archive decodes to the exact input and a larger budget never yields a larger archive
