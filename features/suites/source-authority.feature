Feature: source-authority

  The UORC project keeps product semantics and claim truth in one LexLean
  authority graph and treats Markdown/TOML as generated projections or evidence.

  @UC-CHR-01 @build
  Scenario: UORC charter, claim-boundary, evidence-boundary, and research-position records are accepted by the locked LexLean project and generate the committed project documentation projections.
    Given the UORC LexLean authority modules
    When the source-authority audit inspects project-owned prose and generated projections
    Then a second handwritten UORC specification or semantic rule is rejected

  @UC-CHR-02 @build
  Scenario: Handwritten project prose cannot define a second UORC specification or BCP-14 semantic rule, while non-authoritative evidence and planning prose remain permitted.
    Given the committed model and documentation projections
    When the model gate regenerates every projection from the LexLean authority graph
    Then every committed projection is byte-identical to the generated result

  @UC-PERF-01 @open
  Scenario: measured public-corpus compression improvement, with exact target and scope; open until actually measured
    Given the registered public-corpus performance claim
    When the honesty gate reads its disposition
    Then the claim remains open rather than established by implementation work

  @UC-REC-01 @open
  Scenario: externally accepted compression record; open unless the external authority actually accepts it
    Given the registered external-record claim
    When the honesty gate reads its disposition
    Then the claim remains open without external acceptance evidence
