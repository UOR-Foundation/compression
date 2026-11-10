# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: real-execution

  @UC-REAL-01 @build
  Scenario: Real archives that uorc compress writes for text, structured binary, noise and edge-case inputs are independently decoded and compared against every original byte.
    Given the shipped client and empty, single-byte, text, structured binary, noise and invalid UTF-8 inputs
    When uorc compress writes each archive and uorc decompress reconstructs it
    Then every reconstruction equals its original byte for byte, and compressible text takes a program archive under a quarter of its size
