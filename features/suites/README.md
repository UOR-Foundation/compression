# Suites

One Gherkin file per suite, one scenario per conformance ID (R3).

For UORC, conformance rows are authored in `src/Uorc/Registry.lex.tex`.
`model/ids.toml` and `CONFORMANCE.md` are generated projections; they are
never edited as independent claim sources.

A scenario is tagged with its ID and honesty level:

```gherkin
Feature: <suite name>

  <what the suite is about, in a sentence>

  @UC-SRC-01 @build
  Scenario: <the statement projected from the LexLean registry>
    Given <the fixture>
    When the suite exercises the registered behavior
    Then <the assertion, in the registry's words>
```

`just bdd` fails if a registered ID has no scenario, a scenario names no
registered ID, its honesty level differs, or no test name ends in the ID
lowercased with hyphens replaced by underscores.

The feature-first workflow remains: add the LexLean registry row, add its
scenario, add the initially failing test, then implement the behavior.
