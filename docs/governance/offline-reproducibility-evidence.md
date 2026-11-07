<!-- uorc:non-authoritative -->
# SDK offline reproduction evidence (issue #6)

This document describes the validation procedure and the exact-revision results
recorded below. Actual results come from the `SDK offline reproducibility`
workflow and its uploaded `offline-evidence-*` artifacts. A candidate projection artifact is
review material only; the acceptance job never installs it to make a check pass.

## Declared pipeline

The current product slice is the charter authority rooted in `lexlean.toml`.
There is no root PrismPM product manifest or compression implementation. Its
pipeline is the existing full `just vv`, strengthened by locked Cargo calls,
`lexlean build --all`, `lexlean verify --all`, and always-executed clean-root
reproduction and dependency/tool negatives. No TaskScorer example stands in for
UORC evidence. `prismpm lock check` and `prismpm template check` remain the SDK's
own lock/template validators.

`just reproducibility` is part of every `just vv`. It creates two distinct clean
absolute roots, copies source without proof/build/acquisition outputs, and runs
model generation plus LexLean lock/format/check/build/verify in each. It compares
every renderer-declared projection against the committed original and compares
every file in the returned build and verified artifact trees. It rejects extra
content-addressed result trees, missing/wrong artifact paths, symlinked artifact
or projection parents, empty/duplicate projection inventories, differing bytes,
and embedded checkout paths. It does not invoke `just vv` recursively.

The read-only CI job invokes `node scripts/offline-validation.mjs isolated`
inside the exact image from `prismpm.lock`. That orchestrator:

1. Uses explicit online acquisition (`cargo fetch --locked` and `cargo deny
   --locked --all-features fetch`) with the SDK's binaries. It seals source,
   actual SDK inventory, and every acquisition-cache file into `setup.json`.
2. Creates two more distinct clean roots and runs the complete `just vv` in each
   with `--network=none`, no Docker socket, no credentials or adjacent-checkout
   mount, and independently copied caches and application/proof output paths.
3. Checks the sealed setup before compilation and verifies that only the
   loopback interface is visible. The SDK validates its own PATH and lock.
4. Compares complete current-build canonical inventories and execution identity.
   Reusing dependencies never replaces proof construction or replay.
5. Executes missing/stale/damaged setup negatives in no-network SDK containers.

The orchestrator alone sees a Docker daemon for launching containers. Online
universal bootstrap remains supported; it runs the same always-on reproduction
and negative gates. Its network availability is not reported as offline evidence.

## Canonical inventory and normalization

The renderer exports its own exact projection list with `cargo xtask
projection-inventory`; the harness has no duplicate hand-maintained list.
`lexlean/command-result/1` supplies the exact build and attestation IDs and
artifact roots. Every file beneath both roots is compared, including manifests,
Lean, normalized source, maps, coverage, proof artifacts, and attestations.
No fields or bytes are normalized by this harness. This intentionally fails if
an SDK change introduces host-bound bytes that need a separately reviewed,
field-specific policy. The same architecture and image are used for each pair;
this is not a claim of cross-architecture byte identity.

Raw command logs retain host roots and are retained as operational evidence;
they are not source-derived canonical artifacts. Source identity, SDK identity,
command, exit status, network mode and acquisition identity remain in the
compared execution record. There are no timing fields to discard.

## Typed failures and planted defects

| Code | Meaning and negative coverage |
| --- | --- |
| URD001 | SDK or explicit setup missing; missing receipt is executed in CI |
| URD002 | Setup source/SDK identity stale; changed receipt is executed in CI |
| URD003 | Acquisition cache changed or incomplete; extra cache file is executed in CI |
| URD004 | Offline boundary has non-loopback network or Docker socket |
| URP001 | Symlink/non-regular source or artifact, including projection/artifact parent directories; adjacent-file and parent-directory plants |
| URP002 | Claimed clean root already has build/proof output; unit plants for each output root |
| URP003 | Canonical inventory/byte mismatch or absolute checkout path; changed, wrong-file and extra-tree unit plants |
| URP004 | Missing/wrong projection, artifact, result ID or current-build binding; unit plants |
| URP005 | A planted dependency/tool defect did not fail at its owning gate |
| URX001 | A mandatory command failed, timed out or could not execute |
| URX002 | Invalid harness operation |

Every inner run removes the local SDK lock while a valid adjacent checkout is
present, substitutes an adjacent lock symlink, changes a native SDK inventory
binding, removes and stales `lexlean.lock`, substitutes an adjacent source-directory
symlink, removes `Cargo.lock`, and injects a
successful fake Cargo executable both ahead of SDK PATH and as its only entry.
The SDK command is invoked by its actual SDK path so a fake PrismPM cannot
answer on its behalf. Expected owning diagnostics are PP5401/PP7601 for SDK
lock/PATH violations, LLC0102 for LexLean lock violations, and LLS8001 for
adjacent source substitution. Missing Cargo.lock
must fail locked, offline Cargo metadata. The fake tool must never execute;
then the restored SDK control must pass.

## Validation status

### Pre-merge confinement review, 2026-10-10

Review found that leaf-file checks followed symlinks in projection and SDK
artifact parent directories. The inventory also accepted an empty projection
list when called directly. Two regression tests failed against the original
implementation with `Missing expected exception`. After checking each path
component and validating the inventory at its consumption boundary, all 17
Node tests passed with zero skips or pending tests. The symlink test moves each
of six real fixture directories into an adjacent root, requires URP001, restores
the original directory, and requires the original three-file inventory again.
Empty, duplicate, absolute and traversing projection paths require URP004.

These tests strengthen the existing UC-REP-01/02 scenarios. They are validation
harness evidence; full SDK acceptance remains the unchanged `just vv` and
dual-platform offline workflow. The prior revision's results below do not
qualify this correction or a future SDK reconciliation.

At commit `68054e4cdea921b2a16a176ece6cf4fc98b66c05`, both native platforms
passed full offline execution in
[workflow run 38024057953](https://github.com/UOR-Foundation/compression/actions/runs/38024057953):
[linux/amd64](https://github.com/UOR-Foundation/compression/actions/runs/38024057953/job/114131015606)
and
[linux/arm64](https://github.com/UOR-Foundation/compression/actions/runs/38024057953/job/114131015557).
Both native universal-bootstrap jobs also passed in
[run 38024057898](https://github.com/UOR-Foundation/compression/actions/runs/38024057898)
on the same head. The 15 local Node tests passed with zero skips or pending tests.

Downloaded evidence archives were checked against their published SHA-256:

| Platform | Evidence artifact | ZIP SHA-256 |
| --- | --- | --- |
| linux/amd64 | [11659632072](https://github.com/UOR-Foundation/compression/actions/runs/38024057953/artifacts/11659632072) | `e1e784c76b5f369a3707ef8aead252cc560a826dc97d371c38715969836ee3ff` |
| linux/arm64 | [11660022009](https://github.com/UOR-Foundation/compression/actions/runs/38024057953/artifacts/11660022009) | `c51babe49834636c743c9aaa9877aa270435f64c762918d1e6f874193a03946f` |

Each archive contains two complete `just vv` pass logs and two byte-identical
canonical inventories covering 44 files. Each pair's execution receipts also
match byte-for-byte: exit 0, network `none`, no Docker socket, the exact image
from `prismpm.lock`, actual platform inventory, acquisition and source hashes,
and an empty normalization list. The comparison records distinct roots, no
shared build outputs, and `status: passed`.

Each full run recorded nine rejected dependency/tool defects with the owning
diagnostics described above. Each platform additionally recorded the actual
missing, stale and damaged-cache setup executions with exit 1 and URD001,
URD002 and URD003 respectively. Both modules were freshly built and verified;
all verified results bind to build
`9243ecf49a368ef5d5e8c848ea753b66a9b888ea9a1bddfe49402be914484946`.
Attestations are platform-specific:

- linux/amd64: `48820793d557926b6ab84c82b4a2b205daafbe5bbe642b4623fbe87ecd0cda34`
- linux/arm64: `9044c3a432a9f952db2c2d89b221d6ab68d76c81b56fbdb7ddced74f9e3917d3`

The separate SDK projection-regeneration job passed too. Its reviewable output
had been committed before acceptance ran; acceptance never installs candidate
projections to repair itself. These observations apply to the named revision
and SDK, not arbitrary environments or future compression behavior. Later
source or SDK changes must rerun the same complete checks. Issue #5's
latest-main dependency reconciliation remains a separate M0 requirement.

### Initial SDK execution, 2026-10-10

At commit `af0e42fde446af2d57932ff0bf75705678238671`, workflow run
[38023013882](https://github.com/UOR-Foundation/compression/actions/runs/38023013882)
produced reviewable projections with the locked SDK. Both native architectures
completed acquisition and entered the no-network acceptance container. Both
then rejected stale `CONFORMANCE.md` with PP1101, as expected before the generated
projection correction. This is observed negative evidence, not a full pass.
Artifact `11658789763` contains the projection candidates; its ZIP SHA-256 is
`b8f441a620b9814dc5b3941da3268a2e9479814cc2a6d311bc8e240bc4a35b86`.
The projection files in the subsequent correction are those exact artifact
bytes. Acceptance must be rerun against that correction.

### Formal verification exposed a source identifier defect

At commit `88969519401e8c283bb045be71e26b96fd8eff09`, the strengthened gate
reached full `lexlean verify --all` after the Rust gates and LexLean build.
The locked compiler's generated Registry Lean was rejected with LLV7002 because
`then` was emitted as an unquoted Lean field name. The project source field is
renamed to `expected`, and the projection reader maps that field to the unchanged
Gherkin `Then` step. No compiler, verifier, generated Lean, or SDK implementation
is replaced. The SDK regeneration job checks the projection bytes again, and
full formal verification remains mandatory.
