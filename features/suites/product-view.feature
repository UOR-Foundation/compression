# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: product-view

  @UC-VIW-01 @open
  Scenario: Source-owned presentation values govern labels, enabled actions, focus, accessibility and evidence claims.
    Given the same typed product states and events used by the public API, including arbitrary byte spans and all failures
    When the generated presentation runtime renders ordinary operations and optional advanced evidence
    Then checked results remain checked, scoped evidence carries its scope, transport size remains separate and no browser qualification is inferred from source controls
