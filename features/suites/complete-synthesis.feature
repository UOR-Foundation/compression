# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: complete-synthesis

  @UC-SYN-01 @open
  Scenario: Fair synthesis combines input-derived algebra, recurrence, residuals, dictionary, typed sharing and complete syntax enumeration without making heuristic proposals the admissible universe.
    Given arbitrary input bytes, canonical input normalization, fair phase schedules and exact typed graph lowering
    When the source scheduler advances proposals under explicit effort and each proposal passes independent archive validation
    Then only fully charged executable candidates become incumbents; effort changes search progress rather than admissibility and transport partitions do not choose a different canonical search prefix
