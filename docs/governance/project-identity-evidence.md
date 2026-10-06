<!-- uorc:non-authoritative -->
# UORC Project Identity Hardening Evidence (Issue #3)

Scope:
- Repository: `UOR-Foundation/compression`
- Issue: #3 "Complete UORC project identity hardening without modifying universal policy"
- Base authority model: LexLean sources under `src/Uorc/`

This file records validation evidence only. It is not UORC semantic, claim, or
acceptance authority.

## Identity checks

- Canonical repository identity: `UOR-Foundation/compression`.
- Workspace package metadata in `Cargo.toml` uses:
  - repository: `https://github.com/UOR-Foundation/compression`
  - homepage: `https://github.com/UOR-Foundation/compression`
  - authors: `UOR Foundation`
- The LexLean charter identifies the product as UORC — Universal Object
  Reference Compression.
- `README.md` is generated from the LexLean authority graph rather than
  maintained as an independent identity or semantic source.

## Authority-boundary checks

- Project semantics and claim boundaries originate in
  `src/Uorc/Specification.lex.tex` and `src/Uorc/Registry.lex.tex`.
- `model/*.toml`, `CONFORMANCE.md`, `README.md`,
  `UORC-VERIFICATION.md`, and generated feature suites are checked
  projections, not independent sources.
- Handwritten planning and governance prose is explicitly marked
  non-authoritative.
- Inherited universal-policy paths remain owned by the template contract and
  are not modified by this issue.

## Stale-identity audit

Project-owned prose and metadata were reviewed for:
- unresolved template placeholder identity,
- obsolete external implementation-brief references,
- repository or homepage URLs that do not resolve to the canonical repository,
- prose that would create a competing semantic or proof authority.

The reviewed project-owned surfaces contain no remaining instance in those
categories.

## Validation

The authoritative acceptance path is `just vv`. In particular, the relevant
gates are:
- `lexlean lock --check`
- `lexlean fmt --check --all`
- `lexlean check --all`
- `cargo xtask check-model`
- `cargo xtask audit-source-authority`
- `prismpm template check`
- `prismpm lock check`

The CI workflow runs the complete acceptance gate on both declared Linux
architectures. Issue #3 is complete only when those checks pass for the PR head
and the branch is mergeable with `main`.
