# @generated from src/Uorc/Registry.lex.tex by `cargo xtask check-model --write`.
Feature: source-authority

  @UC-CHR-01 @build
  Scenario: UORC charter, claim-boundary, evidence-boundary, and research-position records are accepted by the locked LexLean project and generate the committed project documentation projections.
    Given the locked UORC LexLean authority project, canonical source files, and generated projection inventory
    When the acceptance gate checks the LexLean lock, canonical formatting, and every authority module before regenerating the project projections
    Then the locked LexLean compiler accepts the authority graph and every committed generated projection is byte-identical to the projection derived from that graph

  @UC-CHR-02 @build
  Scenario: Handwritten project prose cannot define a second UORC specification or BCP-14 semantic rule, while non-authoritative evidence and planning prose remain permitted.
    Given ordinary handwritten project evidence marked non-authoritative and a planted handwritten rule that attempts to define UORC semantics
    When the source-authority audit classifies project-owned prose and checks it against the LexLean-authored authority policy
    Then the marked evidence remains non-authoritative and the conflicting handwritten rule is rejected by the R1 source-authority gate

  @UC-PERF-01 @open
  Scenario: measured public-corpus compression improvement, with exact target and scope; open until actually measured
    Given the registered public-corpus performance claim
    When the honesty gate reads its disposition before any sealed benchmark result has established the target
    Then the claim remains open rather than becoming implementation evidence

  @UC-REC-01 @open
  Scenario: externally accepted compression record; open unless the external authority actually accepts it
    Given the registered external-record claim
    When the honesty gate reads its disposition without external record-acceptance evidence
    Then the claim remains open rather than being promoted by local correctness or benchmark evidence
