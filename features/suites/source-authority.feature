Feature: source-authority

  The UORC project keeps product semantics and claim truth in one LexLean
  authority graph and treats Markdown/TOML as generated projections or evidence.

  @UC-SRC-01 @build
  Scenario: Every project-owned UORC semantic, proof, test, report-intent, and documentation-semantic root is authored in the LexLean authority graph, while generated artifacts are derived evidence rather than independent authority.
    Given the UORC LexLean authority modules
    When the source-authority audit inspects project-owned prose and generated projections
    Then a second handwritten UORC specification or semantic rule is rejected

  @UC-SRC-02 @build
  Scenario: The LexLean authority graph generates the project model and documentation projections, and the source-authority audit rejects a second handwritten UORC specification or semantic rule.
    Given the committed model and documentation projections
    When the model gate regenerates every projection from the LexLean authority graph
    Then every committed projection is byte-identical to the generated result

  @UC-PERF-01 @open
  Scenario: Public-corpus compression improvement remains an open measurement until the sealed benchmark plan produces the stated result.
    Given the registered public-corpus performance claim
    When the honesty gate reads its disposition
    Then the claim remains open rather than established by implementation work

  @UC-REC-01 @open
  Scenario: External compression-record status remains open unless the identified external authority accepts the measured result.
    Given the registered external-record claim
    When the honesty gate reads its disposition
    Then the claim remains open without external acceptance evidence
