# Bootstrap Lineage and Clean-Clone Evidence (Issue #2)

Scope:
- Repository: `UOR-Foundation/compression`
- Issue: #2 "Validate canonical template lineage and fresh-clone bootstrap"
- Date: 2026-10-03

This is project-owned evidence. It does not introduce independent product
semantics or acceptance authority.

## Objectives verified

1. Repository identity and origin
- `git ls-remote origin` resolves to `https://github.com/UOR-Foundation/compression.git`.
- Default branch remains `main`.

2. Template lineage and universal-policy consistency
- Universal policy files are governed by `template.lock` and checked via
  `prismpm template check` in gate execution.
- Lock lineage artifacts are present: `prismpm.lock`, `template.lock`,
  `standards.lock`.

3. Fresh-clone bootstrap reproducibility
- A clean clone with no local build state reaches the same baseline gate
  entrypoint (`just vv`) through the locked SDK workflow path.
- No adjacent checkout or external unpublished host tool is required to
  interpret repository semantics.

4. External SPEC file boundary
- README and planning prose do not claim uncommitted `SPEC*.md` files as
  repository authority.

## External validation oracles used

- GitHub REST API (`gh api`) for repository identity, branch metadata,
  protection state, and workflow/check status.
- Git transport (`git ls-remote`) for canonical remote resolution.

## Audit trail (adversarial replay)

- Canonical remote identity:
  - `git ls-remote origin`
  - Expected authority-boundary result: remote URL resolves to
    `https://github.com/UOR-Foundation/compression.git`.
- Repository identity/default branch:
  - `gh api repos/UOR-Foundation/compression`
  - Expected result: repository identity matches and `default_branch` is `main`.
- Universal-policy drift gate:
  - `prismpm template check`
  - Expected result: locked universal policy bytes are enforced (drift rejected).
- Clean-clone baseline entrypoint:
  - `just vv` from an isolated fresh clone path
  - Expected result: reaches the same declared baseline gate entrypoint as this
    checkout, with failures only from declared SDK-environment requirements.

## Acceptance-criteria mapping (Issue #2)

- `git ls-remote origin` resolves to `UOR-Foundation/compression`:
  - Satisfied by "Repository identity and origin" verification.
- Clean-clone bootstrap reaches the same declared baseline gate:
  - Satisfied by "Fresh-clone bootstrap reproducibility" verification.
- Universal-policy drift is rejected:
  - Satisfied by lock-lineage + `prismpm template check` verification.
- README/planning references do not claim absent or uncommitted `SPEC*.md` as repository authority:
  - Satisfied by "External SPEC file boundary" verification.

## Notes

- This evidence records validation outcomes only.
- Product semantics remain model-owned per `AGENTS.md` and generated
  conformance outputs.
