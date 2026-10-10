# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: sdk-capabilities

  @UC-SDK-01 @build
  Scenario: SDK capability probes execute LexLean-authored cases through the locked public SDK and retain typed failures without substituting host implementations.
    Given the locked SDK and LexLean-authored capability cases
    When the complete acceptance gate checks, builds and verifies each selected public binding and exercises its rejecting case
    Then language, exporter, runtime and unbound requirements remain distinct and unsupported mandatory functionality prevents M0 acceptance

  @UC-SDK-02 @build
  Scenario: The compatibility report binds observed commands, source identities and exact public symbols and schemas without promoting finite cases to large-input feasibility.
    Given capability command transcripts and their bound source inputs
    When the report is generated from the observed SDK run
    Then missing, failed, stale or omitted observations cannot appear as supported capability
