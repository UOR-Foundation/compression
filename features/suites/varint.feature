# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: varint

  @UC-VAR-01 @build
  Scenario: Shortest U64 ULEB128 encoding and strict decoding have source-owned domain and rejection rules, complete boundary vectors, and kernel-checked round-trip and size theorems.
    Given the source-owned unsigned 64-bit varint domain and byte grammar
    When the codec and its boundary and malformed inputs are verified by the locked SDK
    Then encoding is shortest, accepted bytes are canonical, both round trips and exact size hold, and malformed inputs reach their modeled rejection
