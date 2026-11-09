# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: conditional-basis

  @UC-BAS-01 @build
  Scenario: Conditional archives name the client's standard basis, the ulz and ncm decoder libraries, by its UORC-BASIS-V1 identity and decode only against that actual basis.
    Given archives written at the default and best levels, and the same archives with their basis identity altered and resealed
    When uorc inspects and decompresses them
    Then the intact archives name the standard basis identity and decode to the input; an archive naming any other basis is a decode failure even though its header and seal are well formed
