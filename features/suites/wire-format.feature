# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: wire-format

  @UC-WIRE-01 @build
  Scenario: The client decodes only complete self-contained UORC archives in their canonical layout: truncated archives, trailing bytes and unsupported format identities are typed decode failures.
    Given program and ulz kernel archives, each truncated at several points, extended with trailing bytes, and with an altered format identity
    When uorc decompress parses them
    Then the intact archives decode and every altered one returns a decode failure
