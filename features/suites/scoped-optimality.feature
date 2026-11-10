# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: scoped-optimality

  @UC-OPT-01 @build
  Scenario: Compress returns the smallest of the archives it checked, which are the raw archive, the ulz archive, the ncm archive at the best level and any search candidate within the step budget; no minimum beyond that candidate set is claimed.
    Given empty, single-byte, text, structured binary, noise and invalid UTF-8 inputs
    When uorc compress chooses its archive
    Then the archive is never larger than the raw archive of the same length, compressible text takes a strictly smaller archive, and the best level is never larger than the default
