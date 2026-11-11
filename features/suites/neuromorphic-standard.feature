# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: neuromorphic-standard

  @UC-NEU-01 @build
  Scenario: The ncm coder's plasticity is stated as the Prism neuromorphic standard in Uorc.Neuromorphic, with kernel-checked bounds: every mixer weight stays within plus or minus 2^22 under the bounded Hebbian rule, potentiation never lowers and depression never raises a weight, the plasticity lies between 24 and 128 per 65536 (128 at the first byte, 24 in the limit), one bit's weight change is at most 16376 and every product fits a 32-bit lane, a context slot never steps past its gap to the coded bit and consolidates at count 127, and every probability-map point stays within 16 bits; the native coder reads the model's constants.
    Given the Uorc.Neuromorphic module and the native ncm coder's plasticity constants
    When the locked SDK verifies the module and the native test reads the module's constants and theorem inventory
    Then every bound theorem is kernel-checked and every native plasticity constant equals the model's
