# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: codec-engine

  @UC-ENG-01 @build
  Scenario: The uorc engine moves the codec through its space without an LLM in the loop, under the laws of Uorc.Engine, which are kernel-checked: rule 30 on a 64-cell ring stays in the ring, search and portfolio weights stay within 1..16 under the bounded Hebbian rule (+2, -1), the winner of a proved dominance is scheduled before the incumbent it beat, dominance in bytes and steps is irreflexive, asymmetric and transitive and never costs bytes, a claim whose kappa is decided is never minted again, and the generated workflow's schedule and budgets are valid. Each tick mints candidates next to each family's incumbent by the rule 30 register and the search weights, decides them over the kappa-named corpus, and ships a moved portfolio only when it dominates the shipped one; the shipped table and the workflow are generated from the engine state and the model.
    Given the Uorc.Engine module, model/engine.toml and the engine crate
    When the locked SDK verifies the module and the engine's tests read the module's constants, closed vectors and theorem inventory and run ticks over a corpus
    Then every law is kernel-checked, the native transitions equal the model's, no decided kappa is decided twice, every consolidated portfolio dominates the one it replaced, and weights.rs and engine.yml equal what the engine state and the model generate
