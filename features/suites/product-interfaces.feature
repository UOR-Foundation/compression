# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: product-interfaces

  @UC-API-01 @build
  Scenario: The uorc library, CLI and WebAssembly exports run the same compress, decompress, inspect and verify operations over arbitrary bytes, with typed budget, overflow, decode and mismatch failures.
    Given binary input including NUL and invalid UTF-8, passed in memory and on stdin, and an original altered in one bit
    When the library, the uorc binary and the Wasm exports compress, decompress, inspect and verify it
    Then all three produce the same archive, the exact original bytes and the same header description; a non-archive is a decode failure and an altered original is a mismatch
