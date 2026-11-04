# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: machine-resources

  @UC-MACH-02 @open
  Scenario: Execution accounts for logical work and runtime-owned bytes under the explicit uorc.logical-owned-bytes/1 profile, including aliases, metadata, closures, transient copies and cancellation; old diagnostic live-cell counts remain distinct from bytes and neither is a claim of physical CPU time or resident memory.
    Given checked source runtime states, resource budgets, immutable backing identities and release cursors
    When the checked machine takes one transition or ownership release step
    Then attempts, decoder work, peak live ownership and failures preserve exact accounting; native addressability refusal is operational rather than a restriction of mathematical sequence meaning
