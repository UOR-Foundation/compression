# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: scoped-optimality

  @UC-OPT-01 @open
  Scenario: The product minimizes complete archive byte length and unsigned lexicographic order within an explicitly sealed wire, basis and decoder-resource scope.
    Given a checked incumbent and the complete finite set of earlier canonical archives in the sealed scope
    When the independent verifier replays every required predecessor or a proved equivalent canonical framing factorization
    Then a minimum certificate is accepted only after complete coverage; interrupted or exhausted verification remains incomplete and no unrestricted global minimum is claimed
