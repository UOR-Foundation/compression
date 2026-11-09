# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: product-interfaces

  @UC-API-01 @build
  Scenario: The uorc library, CLI and WebAssembly exports run the same exported compress and decompress over arbitrary bytes, with typed budget, overflow and decode failures; inspect and verify operations are not part of this interface yet.
    Given binary input including NUL and invalid UTF-8, passed in memory and on stdin
    When the library, the uorc binary and the Wasm exports compress and decompress it
    Then all three produce the same archive and the exact original bytes, and a non-archive is a decode failure in each
