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

## Notes

- This evidence records validation outcomes only.
- Product semantics remain model-owned per `AGENTS.md` and generated
  conformance outputs.
