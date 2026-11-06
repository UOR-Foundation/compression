# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: product-storage

  @UC-STG-01 @open
  Scenario: Storage admission and achieved job-owned storage remain separate from logical arena memory and physical observations.
    Given typed storage reservations, partial failed writes, releases and custody transfers under explicit requested quotas
    When the source-owned storage transition owner executes positive and negative quota and failure traces
    Then reserved and achieved storage are accounted separately; failure retains achieved bytes and custody transfer creates no duplicate allocation claim
