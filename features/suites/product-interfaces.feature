# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: product-interfaces

  @UC-API-01 @open
  Scenario: The same product semantics serve native library, CLI and browser interfaces, with explicitly versioned streaming ownership rather than invented compatibility with bounded whole-buffer adapters.
    Given binary input including NUL and invalid UTF-8, large streams, resource handles and repeated browser actions
    When the generated native and Wasm interfaces execute compress, decompress, inspect, verify, resume, basis, frontier and comparison operations
    Then arbitrary byte input has no media or UI-sized format gate; modeled errors, generated computation failures, resource refusal and fatal host outcomes remain distinct
