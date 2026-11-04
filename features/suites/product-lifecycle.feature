# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: product-lifecycle

  @UC-PROD-01 @open
  Scenario: The source-owned product lifecycle handles demand, EOF, cancellation, edits, retries, stale completion and publication without exposing unfinished results as completed files.
    Given live sessions, staged outputs, delayed responses, failures after output and host publication outcomes
    When the controller applies the actual operation result and records the adapter outcome
    Then only the current verified result may publish; cancellation cleans owned resources, late responses do not replace new input, and partial or unknown publication is reported honestly
