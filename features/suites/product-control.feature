# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: product-control

  @UC-CTL-01 @open
  Scenario: Source-owned asynchronous product transitions preserve run, revision, ordered input and output ownership.
    Given typed positive and negative control scenarios covering repeated edits, start, cancel, navigation, checkpoint, retry, publication and shutdown
    When the generated control runtime executes every registered scenario and a planted stale-completion defect
    Then obsolete completions have no publication effect; checked output is downloadable only under current ownership and cleanup is explicit
