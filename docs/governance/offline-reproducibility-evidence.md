<!-- uorc:non-authoritative -->
# SDK offline reproduction evidence (issue #6)

This document describes the validation procedure, not a successful run. Actual
results come from the exact-commit `SDK offline reproducibility` workflow and
its uploaded `offline-evidence-*` artifacts. A candidate projection artifact is
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
content-addressed result trees, missing/wrong artifact paths, differing bytes,
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
| URP001 | Symlink/non-regular source or artifact; adjacent-file unit plant |
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

Local Node unit tests exercise artifact/copy/setup comparators and gate wiring.
Full SDK compilation, proof replay, container isolation and planted execution
results are pending the workflow result for the final commit. Do not describe
this issue as complete while those jobs are pending or failing. The generated
projection job is deliberately separate from acceptance and may provide a
reviewable correction commit before acceptance is rerun.

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
