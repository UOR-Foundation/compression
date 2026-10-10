# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: reproducibility

  @UC-REP-01 @build
  Scenario: Source-derived charter projections and complete LexLean build and verification artifacts are byte-identical across clean absolute roots under the locked SDK.
    Given the locked SDK, the source-derived projection inventory, and two clean roots with separate proof and application outputs
    When the acceptance gate regenerates projections and executes full LexLean build and verification in both roots
    Then the complete canonical artifact inventories and bytes agree without normalization, and no acquisition cache is accepted as current-build proof

  @UC-REP-02 @build
  Scenario: Dependency and execution-boundary failures are explicit and planted SDK-lock, source-lock, adjacent-checkout, and host-PATH fallbacks are rejected.
    Given missing and stale locks, a valid adjacent checkout, planted host tools, and missing or changed dependency setup
    When the owning SDK and LexLean gates execute the planted failures and the offline runner checks its source-bound acquisition receipt
    Then PP5401 or PP7601 rejects SDK lock defects, LLC0102 rejects source lock defects, and URD001 through URD004 reject missing setup, stale setup, changed caches, or an online verification boundary
