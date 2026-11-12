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
  Scenario: closed public-corpus claims with a target: every release runs crates/frontier over the Canterbury and Silesia corpora, each pinned by its κ in frontier.toml, records the exact archive bytes and decode steps of its codec point as closed claims the factory decides, and publishes them in the release notes and the release's frontier.toml; the target is that on every public corpus the archive is no larger than each pinned comparator's (gzip, bzip2, xz, zstd, brotli, zpaq and 7-Zip PPMd from the v0.3.0 release run), decided per comparator as a bound claim; open with the target unmet while any bound is refuted (Silesia against zpaq -m5, 41726363 against 39112562 bytes at v0.3.0); scripts/bench.mjs still reports times against the host's core count and memory copy rate
    Given the registered public-corpus performance claim
    When the honesty gate reads its disposition
    Then the claim remains open with its target unmet and its closed claims reported, rather than becoming implementation evidence

  @UC-REC-01 @open
  Scenario: externally accepted compression record; open unless the external authority actually accepts it
    Given the registered external-record claim
    When the honesty gate reads its disposition without external record-acceptance evidence
    Then the claim remains open rather than being promoted by local correctness or benchmark evidence
