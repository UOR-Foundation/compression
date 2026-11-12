# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: scoped-optimality

  @UC-OPT-01 @build
  Scenario: For each frame, compress returns the least of the archives it checked, by bytes and then by decode steps: the raw archive, the trials its frame's portfolio schedules (coder profiles, named model states, and the x86 branch filter where the filter changes the input) and any search candidate within the step budget; no minimum beyond that candidate set is claimed.
    Given empty, single-byte, text, structured binary, noise and invalid UTF-8 inputs
    When uorc compress chooses its archive
    Then the archive is never larger than the raw archive of the same length, and compressible text takes a strictly smaller archive
