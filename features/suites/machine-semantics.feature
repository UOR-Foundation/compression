# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: machine-semantics

  @UC-MACH-01 @open
  Scenario: Every admitted graph has the source-defined meaning of all 39 wire operations, including eager typed failures, prior-block calls and observer-before-update Scan with its final updater.
    Given typed admitted graphs and an independent mathematical reference semantics
    When the source machine executes and its simulation obligations and distinguishing examples are checked
    Then values and failures agree with the reference; unreachable code is still validated and no unsupported opcode or hidden literal shortcut is accepted
