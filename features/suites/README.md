<!-- uorc:non-authoritative -->
# Suites

One Gherkin file per suite, one scenario per conformance ID (R3).

For UORC, conformance rows and their Given/When/Then intent are authored in
`src/Uorc/Registry.lex.tex`. `model/ids.toml`, `CONFORMANCE.md`, and every
`*.feature` suite are generated projections; they are never edited as
independent claim or test-intent sources.

A scenario is tagged with its ID and honesty level:

```gherkin
Feature: <suite name>

  <what the suite is about, in a sentence>

  @UC-CHR-01 @build
  Scenario: <the statement projected from the LexLean registry>
    Given <the LexLean-authored precondition>
    When <the LexLean-authored action>
    Then <the LexLean-authored expected result>
```

`just bdd` fails if a registered ID has no scenario, a scenario names no
registered ID, its honesty level differs, or no test name ends in the ID
lowercased with hyphens replaced by underscores.

The feature-first workflow remains: add the LexLean registry row including its
scenario intent, run `just model-write`, add the initially failing named test,
then implement the behavior.
