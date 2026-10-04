# UORC (Universal Object Reference Compression)

Reference implementation workspace for UORC, built with LexLean and PrismPM.
Repository semantics and acceptance are defined only by committed repository
authorities and generated artifacts.

## Status

Repository bootstrap and rebrand are in progress. The implementation backlog,
acceptance mapping, and issue plan are tracked in
`docs/planning/uorc-implementation-issues.md`.

Current phase goals:

1. Rebrand scaffold artifacts for UORC and `UOR-Foundation/compression`.
2. Establish complete claim/authority/register scaffolding for UORC profile 02.
3. Implement the full product and verification obligations under PrismPM gates.

## Working Contract

- Universal repository policy: [AGENTS.md](AGENTS.md)
- Template inheritance/process contract: [TEMPLATE-CONTRACT.md](TEMPLATE-CONTRACT.md)
- Acceptance boundary: [VERIFICATION.md](VERIFICATION.md)

External planning briefs may inform implementation work, but are not repository
authority unless committed and modeled through the repository's declared
authoritative sources.

## Execution Discipline

- Authoritative behavior, claims, and diagnostics are model-owned.
- `just vv` remains the only complete repository acceptance gate.
- No handwritten fallback implementation is allowed outside the modeled graph.
- External tools are validation-only unless explicitly modeled as production dependencies.

Licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).
