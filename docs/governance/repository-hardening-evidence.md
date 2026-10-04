# Repository Hardening Evidence (Issue #1)

Scope:
- Repository: `UOR-Foundation/compression`
- Issue: #1 "Harden the existing compression repository and enforce baseline protections"
- Date: 2026-10-03

This document is project-owned governance evidence. It does not define product
semantics and does not modify template-owned universal policy files except by
locked template-compatible updates.

## Implemented controls

1. Branch protection on `main`
- Require pull request before merge: enabled.
- Required approving reviews: >= 1.
- Dismiss stale reviews: enabled.
- Require code owner reviews: enabled.
- Required status checks: bootstrap acceptance checks.
- Require conversation resolution: enabled.
- Restrict force pushes: enabled.
- Restrict branch deletion: enabled.

2. Actions least-privilege defaults
- Repository default workflow token permission: `read`.
- Workflow approval for PR reviews by Actions: disabled.
- Actions policy: selected actions + reusable workflows from explicit allowlist.
- Full-length SHA pinning requirement: enabled.

3. CODEOWNERS and review routing
- Added `.github/CODEOWNERS` with default owner and explicit ownership for
  `.github/`, `bootstrap/`, and `model/` paths.

4. Metadata and visibility
- Repository remains `https://github.com/UOR-Foundation/compression`.
- Visibility remains public.
- License files present and visible in repository.

5. Template and lock consistency
- Added missing lock files required by universal gate: `prismpm.lock`,
  `template.lock`, `standards.lock`.
- Synchronized `.devcontainer/devcontainer.json` to match locked template
  lineage referenced by `template.lock` policy hashes.

## Verification evidence

External authoritative oracle used:
- GitHub REST API (`gh api`) for branch protection and actions policy state.

Evidence checks performed:
- `GET /repos/UOR-Foundation/compression/branches/main/protection` returns
  active policy instead of 404.
- `GET /repos/UOR-Foundation/compression/actions/permissions` and
  `.../actions/permissions/workflow` show least-privilege defaults.
- `gh run` confirms bootstrap checks are active and enforced on PR.

Policy-violation rejection test:
- A direct push to protected `main` from a non-exempt actor should be rejected
  by GitHub authorization policy.
- If actor is admin-exempt, equivalent validation is by attempted force-push or
  branch-deletion operation and GitHub policy response.

