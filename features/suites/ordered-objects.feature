# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: ordered-objects

  @UC-OBJ-01 @build
  Scenario: Archives of ordered objects preserve the exact object boundary vector, including zero objects and distinct zero-length objects.
    Given object vectors with binary data, internal and repeated empty objects, zero objects, and archives truncated or extended by one byte
    When uorc compresses each vector as one archive and decompresses its objects
    Then the header records every object length, the decoded objects equal the originals in order with every empty boundary preserved, and truncated or extended archives fail
