# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: archive-integrity

  @UC-INT-01 @open
  Scenario: Archive and basis integrity use an explicitly identified bounded streaming construction with standard SHA-256 components; digest agreement never replaces exact byte comparison.
    Given source-owned integrity identities, locked independent vectors, split updates, finalization and a planted arithmetic defect
    When the actual source checks the vectors and the archive path hashes the received preimage
    Then standard-domain boundaries and any separately named composition are explicit; invalid protocols fail, mutations are caught, and no cryptographic assumption is reported as a kernel theorem
