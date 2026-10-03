# UORC Rebrand and Publish Checklist

Goal: finalize repository identity and governance before first push to
`github.com/UOR-Foundation/compression`.

## 1. Identity and naming

- [ ] Repository URL fields point to `UOR-Foundation/compression`.
- [ ] README states UORC purpose and current status.
- [ ] Project docs distinguish inherited template policy from product semantics.
- [ ] No unresolved project-owned template placeholders remain.

## 2. GitHub repository setup

- [ ] Create `UOR-Foundation/compression`.
- [ ] Set branch protections (default branch, PR review, required checks).
- [ ] Configure least-privilege Actions defaults.
- [ ] Add initial labels and milestones used by implementation backlog.

## 3. Backlog import

- [ ] Create issues from `docs/planning/uorc-implementation-issues.md`.
- [ ] Assign milestones and owner labels.
- [ ] Link issue IDs into model claim planning docs when claims are populated.

## 4. PrismPM rigor checks

- [ ] Confirm `just vv` remains single complete acceptance gate.
- [ ] Confirm no product behavior authored outside modeled/allowed sources.
- [ ] Confirm authority/claim discipline remains intact.

## 5. Publication readiness

- [ ] Initialize git history and set `origin` to new repository.
- [ ] Push initial branch and open bootstrap PR if branch protection requires.
- [ ] Record immutable commit used as UORC bootstrap baseline.

