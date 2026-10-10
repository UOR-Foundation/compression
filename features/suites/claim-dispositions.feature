# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: claim-dispositions

  @UC-HON-01 @build
  Scenario: Evidence classes and research dispositions remain distinct in the LexLean claim policy.
    Given the complete finite evidence-kind inventory and its LexLean classification functions
    When the SDK kernel-checks every evidence classification and benchmark outcome case
    Then fixtures, certificates, version metadata and measurements cannot become stronger proof or record claims

  @UC-HON-02 @build
  Scenario: The honesty ledger and claim summaries are exact source projections with open research claims protected against promotion.
    Given the LexLean claim policy, registered claims and generated ledger
    When the model gate checks projections and planted claim-promotion defects
    Then missing, duplicate, mismatched and promoted ledger evidence is rejected while unmet benchmark outcomes remain measured data
