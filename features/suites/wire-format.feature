# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: wire-format

  @UC-WIRE-01 @build
  Scenario: The client decodes only complete UORC archives in their canonical layout, self-contained or over the standard basis: truncated archives, trailing bytes and unsupported format identities are typed decode failures.
    Given a program archive and a coded archive over the standard basis, each truncated at several points, extended with trailing bytes, and with an altered format identity
    When uorc decompress parses them
    Then the intact archives decode and every altered one returns a decode failure
