<!-- uorc:non-authoritative -->
# Claim-disposition implementation evidence (issue #9)

The authority is `src/Uorc/Claims.lex.tex`, imported by the existing registry.
This document records validation procedure and results; it supplies no additional
product specification. The source graph remains the only semantic authority.

## Scope and generated outputs

The source defines ten evidence classes, eight supported claim classes, four
research dispositions, and an exhaustive classification for each evidence kind.
Its 48 closed theorem cases apply the actual classification functions and check
their results, measurement validity, and protected research dispositions. The
theorem declarations have empty axiom allowances. The locked SDK builds and
kernel-verifies the complete graph.

`model/claim-policy.toml`, every registered row in `model/ledger.toml`, the
README and verification summaries, scenarios, and conformance table are generated
from the graph. Rust only projects and reconciles these records. It supplies no
production classifier, evaluator, compiler replacement, or compression runtime.
Classification does not acquire or validate an external evidence artifact. No
measurement or external record receipt exists in this slice, and the generated
`UC-PERF-01` and `UC-REC-01` rows remain `open`/`none`/`not_measured`.

The policy distinguishes an unsuccessful measured result from an implementation
failure. The closed `measurementComplete` cases accept both measured target
outcomes, while neither benchmark outcome becomes implementation evidence.
The ledger is a register of source obligations, not a claim that every row ran
successfully in the current build.

## Falsification

Before implementation, the registered `conformance_uc_hon_02` test failed with
an empty ledger (zero rows against eight registered IDs). The completed projection
has exactly one row per ID. Structural tests reject missing/duplicate rows,
wrong scenario bindings, and coordinated registry/ledger promotion of the two
open claims. Classification inventories reject missing/duplicate constructors
and implementation eligibility for open evidence.

`just claim-dispositions` executes every `ClaimMutation` authored in the source,
in isolated source-only roots. Each mutant first builds with the locked SDK,
then `lexlean verify --all` rejects it with LLV7002 at its exact owning generated
theorem. The original source is restored, generated outputs are removed, and
full verification is repeated successfully for every control.

| Source mutation | Expected owning theorem |
| --- | --- |
| Fixture promoted to kernel theorem | `claimForEvidence_fixture_result` |
| Scoped minimum certificate promoted to external record | `claimForEvidence_scoped_minimum_certificate` |
| Version metadata promoted to kernel theorem | `claimForEvidence_version_metadata` |
| Losing benchmark relabeled target met | `researchDisposition_benchmark_target_unmet` |
| Open performance claim relabeled build | `performance_level_unpromoted` |
| Successful benchmark promoted to kernel theorem | `claimForEvidence_benchmark_target_met` |

The SDK diagnostic omits theorem names, so attribution uses the current mutant's
generated Lean and the diagnostic's generated file/line location. An unrelated
proof, wrong source file, syntax error, tool failure, timeout, signal, or successful
exit cannot qualify. The harness's five executed tests exercise these rejected
outcomes, omitted/filtered gates, malformed mutation inventories, and precise
single-field mutation. The harness inspects its own mandatory invocation and
unfiltered iteration. The Rust test runner invokes all harness tests and rejects
skips, pending tests, or an empty run.

## Acceptance and retained evidence

The full `just vv` includes the six executed mutations, complete LexLean
build/verification, exact projection checks, and clean-root reproduction. The
same path runs in both native bootstrap jobs and twice per architecture in the
offline workflow. It retains `claims.*/mutations.json`, current mutant generated
Lean, and build/rejection/restored command logs beneath `.prism/uorc`. Each
receipt binds the original and mutant source hashes, generated Lean hash,
owning theorem, build IDs, restored attestation, and log hashes.

The immutable SDK is the one selected by `prismpm.lock`; this change neither
reconciles a newer SDK nor resolves issue #5's separate runtime capability gaps.
M0 and the full M1 milestone remain incomplete. Hosted results belong to the
exact PR revision and its Actions artifacts, not to this prose.
