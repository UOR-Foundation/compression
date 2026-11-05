# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: product-storage

  @UC-STG-01 @open
  Scenario: Storage admission and achieved job-owned storage remain separate from logical arena memory and physical observations.
    Given typed storage reservations, achieved partial writes, bounded immutable read leases, prepaid image reservations and actual release/transfer receipts under explicit requested quotas
    When the generated storage and payload-image owners execute quota/refusal, same-lease resume, cancellation, identity, cleanup and one-time transfer controls and reject planted defects
    Then reserved and achieved storage remain separate from logical arena memory; payload materialization is admitted before request, every failure retains achieved owners, and custody transfer creates no duplicate allocation claim
