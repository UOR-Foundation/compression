# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: product-storage

  @UC-STG-01 @open
  Scenario: Storage admission and observed job-owned storage remain separate from logical arena memory and physical observations.
    Given typed storage reservations, achieved partial writes, bounded immutable input/output leases, prepaid image admission/extraction, frozen accounting work prefixes, scoped I/O totals and actual release/publication receipts under explicit requested quotas
    When the generated storage, payload-image and output reporting owners execute quota/refusal, retained-prefix resume, cancellation, attachment identity, partial failed writes, stale settlement, unknown publication, cleanup and one-time transfer controls and reject planted defects
    Then reserved and achieved storage remain separate from logical arena memory; input/output materialization is prepaid and bounded, failure retains achieved owners and actual publication, and settlement alone cannot claim verified archive success
