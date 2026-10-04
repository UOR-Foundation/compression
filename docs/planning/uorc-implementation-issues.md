# UORC GitHub Issue Backlog (Spec 1.0-draft.2)

Purpose: define the complete implementation work as granular GitHub issues for
`UOR-Foundation/compression`, with explicit Definition of Done and acceptance
criteria for each issue.

Scope baseline:
- Specification input brief: `SPEC(3).md` (`UORC-IMPLEMENTATION-SPEC-001`, rev `1.0-draft.2`)
- Machine profile: `uorc/reference-machine/2` (profile byte `02`)
- Archive format: `uorc/archive/1`
- Delivery discipline: PrismPM rigor and template universal policy (`AGENTS.md`)

Authority boundary note:
- This backlog is planning content only. Repository semantics and acceptance are
	defined by committed repository authorities and generated evidence, not by an
	uncommitted external `SPEC*.md` file.

Issue authoring conventions:
- Labels suggested in each issue body are required for triage consistency.
- Every issue references owning claim IDs once the UORC register is populated.
- Every issue adds or updates at least one scenario under `features/suites/`.
- No issue closes without passing `just vv` in the locked SDK environment.

---

## Epic 0: Program setup and repository publication

### Issue 0001 - Create repository and enforce baseline protections

Suggested labels: `program`, `repo-setup`, `security`, `governance`

Problem:
The target repository `github.com/UOR-Foundation/compression` does not exist yet.
We need a governed repository baseline before implementation issues execute.

Tasks:
1. Create the GitHub repository under `UOR-Foundation` as `compression`.
2. Set default branch protections (review + required status checks + no force push).
3. Enable Actions with least privilege defaults.
4. Configure CODEOWNERS/review settings aligned with UOR policy.
5. Wire repository metadata (description/topics/license visibility).

Definition of Done:
- Repository exists and is accessible at the target URL.
- Branch protections are active and verified by attempted policy violation.
- Required checks include at least the bootstrap/verification gate intended for PRs.
- Repository settings are captured in a committed governance note.

Acceptance Criteria:
- `gh repo view UOR-Foundation/compression` resolves successfully.
- A test branch push violating policy is rejected with expected GitHub diagnostics.
- A PR to default branch requires configured review and checks.

---

### Issue 0002 - Initialize git history and push rebranded scaffold

Suggested labels: `program`, `repo-setup`, `bootstrap`

Problem:
The local scaffold must become the first canonical commit history for UORC.

Tasks:
1. Initialize git if missing and add remote `origin` to `UOR-Foundation/compression`.
2. Commit initial rebrand + planning artifacts.
3. Push default branch and verify clean clone/bootstrap behavior.

Definition of Done:
- The repository has an initial, reviewable commit history on default branch.
- A fresh clone can run baseline checks without hidden local dependencies.

Acceptance Criteria:
- `git ls-remote origin` resolves to the expected repository.
- Fresh clone from CI-like environment reaches intended bootstrap gate entrypoint.
- Initial commit references the implementation spec revision.

---

## Epic 1: Rebrand and project identity hardening

### Issue 0101 - Complete template-to-UORC naming and URL rebrand

Suggested labels: `rebrand`, `docs`, `config`

Problem:
Template naming leaks remain in docs/config and can cause trust-chain ambiguity.

Tasks:
1. Replace template-facing branding with UORC branding where project-owned.
2. Update repository/homepage URLs to `UOR-Foundation/compression`.
3. Keep universal policy files intact where required by contract.
4. Add explicit status note distinguishing inherited policy vs product model.

Definition of Done:
- Project-owned docs and metadata no longer present as generic template copy.
- Universal policy inheritance remains valid and unweakened.

Acceptance Criteria:
- Search for project-owned template placeholders returns zero unresolved hits.
- `template-check` continues to pass for immutable policy files.

---

### Issue 0102 - Author UORC project charter and implementation boundary docs

Suggested labels: `docs`, `governance`, `architecture`

Problem:
Developers need a clear authority map from spec to model/repo responsibilities.

Tasks:
1. Add project charter with scope/non-scope and claim boundaries.
2. Add implementation boundary doc: what is authoritative vs generated.
3. Document production-vs-validation dependency separation.

Definition of Done:
- New maintainers can identify where to add semantics, tests, and evidence.

Acceptance Criteria:
- Docs explicitly map to spec sections 0, 1, 3, 4, 9, 20.
- Review confirms no conflicting second source of authority in prose.

---

## Epic 2: Dependency and SDK lock baseline

### Issue 0201 - Execute mandatory bootstrap freeze and compatibility report

Suggested labels: `dependencies`, `sdk`, `risk`

Problem:
Spec requires exact dependency resolution/freeze and compatibility evidence.

Tasks:
1. Resolve/freeze selected template/LexLean/PrismPM revisions.
2. Record full commit hashes and source digests.
3. Produce compatibility report from executable probes, not assertions.
4. Commit lock/binding artifacts required by template contract.

Definition of Done:
- Immutable dependency identity is established and auditable.

Acceptance Criteria:
- Report includes actual capability probe results for required transport/export features.
- `prismpm lock check` and `prismpm template check` pass in CI.

---

### Issue 0202 - Prove offline, reproducible SDK-bound execution path

Suggested labels: `reproducibility`, `sdk`, `ci`

Problem:
Build/verify must run offline after explicit setup and remain reproducible.

Tasks:
1. Validate offline execution for model/build/verify pipeline.
2. Add reproducibility note for dual-path clean build comparison.
3. Ensure no hidden host toolchain fallback is used.

Definition of Done:
- Pipeline is demonstrably SDK-bound and offline-capable.

Acceptance Criteria:
- Two clean-path builds produce expected canonical artifact identity behavior.
- Failure mode for missing lock/setup is explicit and typed.

---

## Epic 3: Model registry and conformance skeleton for UORC claims

### Issue 0301 - Populate UORC claim ID register with full required families

Suggested labels: `model`, `conformance`, `core`

Problem:
Spec mandates broad claim classes (`UC-*`) with strict honesty levels.

Tasks:
1. Populate `model/ids.toml` with UORC claim IDs from section 24.
2. Assign levels (`some-true`, `build`, `open`) correctly.
3. Generate/update `CONFORMANCE.md` from model.

Definition of Done:
- Claim register is complete enough to drive scenario/test ownership.

Acceptance Criteria:
- Every mandatory claim family appears with intended ownership.
- Model gate fails when scenario/test mapping is broken.

---

### Issue 0302 - Populate authority register for all mandatory external sources

Suggested labels: `model`, `authorities`, `compliance`

Problem:
External authorities must be explicit, immutable, and role-scoped.

Tasks:
1. Fill `model/authorities.toml` for all required authorities in spec appendix/register.
2. Link authorities to realized claims without conflating proof with citation.
3. Record provenance limits where issuer digests are unavailable.

Definition of Done:
- Authority register supports independent reconstruction of inputs/tools.

Acceptance Criteria:
- Each `some-true` claim has resolvable authority references.
- No authority row implies local proof of external guarantee.

---

### Issue 0303 - Populate honesty ledger for open/performance/record claims

Suggested labels: `model`, `metrics`, `governance`

Problem:
Performance and record claims must remain open until measured/accepted.

Tasks:
1. Register open claims (`UC-PERF-01`, `UC-REC-01`) with explicit evidence requirements.
2. Add anti-mislabeling checks in project verification docs.

Definition of Done:
- Measurement claims cannot be accidentally promoted to established truth.

Acceptance Criteria:
- Gate catches intentional mutation that misclassifies open claims as proven.

---

## Epic 4: Wire format and parser/serializer correctness

### Issue 0401 - Implement shortest ULEB128 encode/decode with exact rejection rules

Suggested labels: `core`, `wire`, `proof`

Problem:
ULEB128 shortest-form rules are foundational and strict in profile 02.

Tasks:
1. Implement encode/decode and canonicality checks.
2. Add full boundary vector suite including malformed encodings.
3. Prove encode/decode and size theorems.

Definition of Done:
- ULEB128 behavior exactly matches section 7.1 and theorem inventory.

Acceptance Criteria:
- Positive and negative vectors pass/fail exactly as specified.
- Required varint theorems are kernel-checked.

---

### Issue 0402 - Implement complete archive frame parser/serializer for `uorc/archive/1`

Suggested labels: `core`, `wire`, `decoder`

Problem:
Frame correctness defines admission and identity boundary.

Tasks:
1. Implement parser/serializer for all frame fields and constraints.
2. Enforce conditional-basis field behavior and exact EOF rules.
3. Add parse-stage diagnostics with offsets/paths.

Definition of Done:
- Roundtrip and rejection behavior is deterministic and complete.

Acceptance Criteria:
- `wire_parse_serialize`, `wire_serialize_parse`, `wire_size_exact` pass.
- Field-specific negative tests reach expected validator ownership.

---

### Issue 0403 - Implement graph body encoding/decoding and static validation

Suggested labels: `core`, `wire`, `typing`

Problem:
Graph bodies require exact block/instruction/reference structural validation.

Tasks:
1. Encode/decode graph body structure and references.
2. Validate local/block back-reference constraints and arities.
3. Reject unknown opcode/type tags and malformed captures.

Definition of Done:
- Static graph validation is complete before dynamic execution.

Acceptance Criteria:
- Negative tests for references/arity/types map to typed diagnostics.
- Parser consumes exactly `body_length` and rejects trailing/short bytes.

---

## Epic 5: Semantics and executable evaluator

### Issue 0501 - Author independent `ReferenceSemantics` model

Suggested labels: `semantics`, `proof`, `core`

Problem:
Reference semantics must be independent from executable machine.

Tasks:
1. Define value types/instructions/evaluation relation and error taxonomy.
2. Encode exact evaluation order and iteration semantics.
3. Define eligibility/resolution relation used by proofs/certification.

Definition of Done:
- Semantics model is readable, complete, and not a wrapper around runtime.

Acceptance Criteria:
- Review confirms no circular dependence on executable evaluator.
- Semantics supports theorem statements in section 19.

---

### Issue 0502 - Implement bounded `StepMachine` evaluator with deterministic errors

Suggested labels: `runtime`, `semantics`, `core`

Problem:
Runtime evaluator must terminate under envelope and mirror semantics.

Tasks:
1. Implement finite-frame evaluator with explicit budgets.
2. Enforce deterministic error precedence and typed outcomes.
3. Separate operational failures from semantic errors.

Definition of Done:
- Evaluator can execute admitted programs and fail safely under limits.

Acceptance Criteria:
- `evaluator_terminates`, `step_reference_sound`, `step_reference_complete_bounded` pass.
- Error precedence matches appendix/rules with dedicated tests.

---

### Issue 0503 - Implement persistent store (`M`) and `Scan` semantics/refinement

Suggested labels: `semantics`, `data-structure`, `proof`

Problem:
Profile 02 introduces stateful store + sequential emission as core semantics.

Tasks:
1. Implement finite-map semantics and runtime sparse-trie representation.
2. Implement `StoreNew/Read/Write` and `Scan` with observe-before-update order.
3. Prove runtime refinement and alias preservation.

Definition of Done:
- Store and scan are both semantically and operationally validated.

Acceptance Criteria:
- `store_refinement`, `scan_linear_evolution` pass.
- Tests include final update execution/error propagation behavior.

---

### Issue 0504 - Implement static last-use liveness and reachable-heap accounting

Suggested labels: `resources`, `runtime`, `proof`

Problem:
Resource accounting requires explicit liveness release without dead-code elimination.

Tasks:
1. Compute static last-use schedule over operands/captures/return.
2. Implement release/tombstone behavior preserving slot identities.
3. Account live cells by reachable graph, not hash-equality inference.

Definition of Done:
- Liveness optimization preserves semantics and tightens live-cell metrics.

Acceptance Criteria:
- `last_use_safe` and resource correspondence tests pass.
- Unused failing instruction still executes and fails as required.

---

## Epic 6: Resource model, admission, and safety envelope

### Issue 0601 - Implement profile-02 resource ledger and counters

Suggested labels: `resources`, `core`, `verification`

Problem:
Draft.2 resource model differs materially from draft.1 and must be exact.

Tasks:
1. Implement operation-specific work/live accounting from Appendix D.
2. Track compulsory vs discretionary vs replay/proof/IO accounting separately.
3. Emit complete counters in receipts/results.

Definition of Done:
- Logical resource ledger behavior is deterministic and auditable.

Acceptance Criteria:
- `resource_accounting_exact` passes.
- Counter partitions are present and non-overlapping in outputs.

---

### Issue 0602 - Enforce decode envelope, feasibility gate, and safe rejection

Suggested labels: `safety`, `admission`, `decoder`

Problem:
Admission must reject unsafe/infeasible queries before search.

Tasks:
1. Implement `rawRequirement` and feasibility pre-check.
2. Enforce all decode envelope limits with checked arithmetic.
3. Validate indices/lengths pre-allocation and fail closed.

Definition of Done:
- Infeasible requests fail early with typed diagnostics.

Acceptance Criteria:
- `raw_roundtrip`/`raw_size_bound` and boundary admissions pass.
- Expand-to-huge malformed cases fail before unsafe allocation.

---

## Epic 7: Encoder synthesis engines and deterministic scheduling

### Issue 0701 - Implement deterministic encoder skeleton and incumbent management

Suggested labels: `encoder`, `core`, `determinism`

Problem:
Encoder must always produce checked incumbent and never report unchecked success.

Tasks:
1. Implement canonical raw incumbent bootstrap and mandatory verification.
2. Add candidate generation/evaluation/eligibility workflow.
3. Enforce monotonic incumbent updates on exact `(length, bytes)` key.

Definition of Done:
- Encoder success implies exact reconstruction of original snapshot.

Acceptance Criteria:
- `encoder_success_lossless`, `eligibility_reflects`, `incumbent_monotone` pass.

---

### Issue 0702 - Implement generic typed generation engine with coverage cursor

Suggested labels: `encoder`, `synthesis`, `search`

Problem:
Search cannot collapse to fixed codec portfolio; typed generation is required.

Tasks:
1. Build typed fragment enumeration in nondecreasing serialized cost order.
2. Implement deterministic structural ordering and resumable cursor.
3. Add coverage hooks for exhaustive/certification integration.

Definition of Done:
- Engine emits valid typed candidates beyond literal-only cases.

Acceptance Criteria:
- Structural fixture suite demonstrates produced nonliteral candidates.
- Generator inventory is not hardcoded codec-name dispatch.

---

### Issue 0703 - Implement address-chart and byte-ring synthesis families

Suggested labels: `encoder`, `synthesis`, `fixtures`

Problem:
Spec mandates these proposal families and concrete fixture success obligations.

Tasks:
1. Add chart/byte-ring proposal generation and parameter inference.
2. Validate against full-coordinate exact reconstruction.
3. Measure whole-archive cost including framing/layout changes.

Definition of Done:
- Required structural families produce verified improving archives on fixtures.

Acceptance Criteria:
- Family 1/2 obligations in section 23.2 are satisfied and reported.

---

### Issue 0704 - Implement typed anti-unification with substitution correctness

Suggested labels: `encoder`, `synthesis`, `proof`

Problem:
Generalization must be typed and proven, not heuristic-only.

Tasks:
1. Implement deterministic anti-unification over expression trees.
2. Track consistent parameter reuse across repeated mismatches.
3. Prove substitution restores original denotations in context.

Definition of Done:
- Shared abstractions are introduced with formal substitution guarantees.

Acceptance Criteria:
- `generator_substitution` passes.
- Fixture suite includes positive and type-mismatch non-generalization cases.

---

### Issue 0705 - Implement joint sharing and layout optimization with contextual dominance

Suggested labels: `encoder`, `optimization`, `proof`

Problem:
Sharing/layout decisions are context-sensitive and cannot use shallow keys.

Tasks:
1. Implement shared-vs-inline exploration and placement-sensitive costing.
2. Add layout exploration across legal topologies and reference-width boundaries.
3. Prove pruning/contextual dominance rules used by optimization.

Definition of Done:
- Engine can choose not to share when sharing costs more.

Acceptance Criteria:
- `contextual_dominance_sound` and layout boundary fixtures pass.

---

### Issue 0706 - Implement bounded-state synthesis (`Fold`/`Scan`) engine

Suggested labels: `encoder`, `stateful`, `synthesis`

Problem:
State synthesis is mandatory and must produce charged sequential references.

Tasks:
1. Add finite-state update/observer proposal search.
2. Prefer `Scan` for sequential emission when avoiding redundant recurrence.
3. Include store-backed state proposals within envelope.

Definition of Done:
- Stateful fixtures are solved by production synthesis path, not injected witness.

Acceptance Criteria:
- Family 4 and 5 obligations in section 23.2 pass.

---

### Issue 0707 - Implement fair deterministic scheduler and resumable tasks

Suggested labels: `scheduler`, `determinism`, `resume`

Problem:
Spec fixes phase order and fairness/commit ordering requirements.

Tasks:
1. Implement phase-round-robin scheduling with fixed quantum.
2. Preserve deterministic task ordering and commit sequence.
3. Validate same result under replayed resume boundaries.

Definition of Done:
- Equivalent work prefixes produce identical logical state/incumbent.

Acceptance Criteria:
- `checkpoint_resume` and scheduler invariance tests pass.

---

## Epic 8: Universe, exhaustive enumeration, and certificates

### Issue 0801 - Implement sealed universe descriptor and admission predicate

Suggested labels: `universe`, `certification`, `core`

Problem:
Certification depends on exact query-bound candidate universe.

Tasks:
1. Encode/validate `UniverseDescriptor` and length ceiling derivation.
2. Implement `Admitted` and `Eligible` as distinct predicates.
3. Bind objective/tie policy descriptors and identities.

Definition of Done:
- Universe definition is independent of discovery engine behavior.

Acceptance Criteria:
- `universe_enumerator_complete` readiness checks pass for bounded fixtures.

---

### Issue 0802 - Implement independent complete enumerator baseline

Suggested labels: `enumeration`, `certification`, `proof`

Problem:
Need independent complete path for small exact certification instances.

Tasks:
1. Enumerate candidate byte strings by length+lexicographic order.
2. Parse/type/evaluate/compare all candidates under sealed query.
3. Add resumable cursor and explicit incomplete status under budget.

Definition of Done:
- Enumerative minimum can be proven on designated micro-instances.

Acceptance Criteria:
- Certified fixtures in section 23.4 pass with explicit complete/incomplete cases.

---

### Issue 0803 - Implement prefix rejection and lower-bound coverage machinery

Suggested labels: `certification`, `proof`, `search`

Problem:
Prefix pruning must be sound, route-limited, and coverage-accounted.

Tasks:
1. Implement closed rejection reason set from section 12.2.
2. Implement sound completion lower bound updates.
3. Maintain exact coverage partition categories and invariants.

Definition of Done:
- Prefix route can prove full improving-region coverage when complete.

Acceptance Criteria:
- `prefix_rejection_sound`, `completion_lower_bound_sound`, `coverage_partition` pass.

---

### Issue 0804 - Implement certificate encoding (`UORP` v02) and checker

Suggested labels: `certification`, `wire`, `verification`

Problem:
Portable minimum/canonical-minimum certificates are mandatory.

Tasks:
1. Implement `UORP` envelope parse/serialize/check.
2. Implement route 00 (replay) and route 01 (prefix tree) checkers.
3. Enforce independent derivation of improving region by verifier.

Definition of Done:
- Certificate checker returns verified/refuted/incomplete with typed reasons.

Acceptance Criteria:
- `certificate_sound` and `canonical_tie_sound` pass.
- Malformed/stale/mismatched certificate negatives map to `UC400*` codes.

---

### Issue 0805 - Implement frontier analysis request/report (`uorc/frontier-receipt/2`)

Suggested labels: `frontier`, `analysis`, `verification`

Problem:
Bounded multiobjective frontier is required as a separate operation.

Tasks:
1. Implement frontier query schema and operation.
2. Preserve incomparable points and declared identity tie policy.
3. Implement completeness verification by bounded exhaustive replay.

Definition of Done:
- Frontier operation is explicit, bounded, and status-typed.

Acceptance Criteria:
- `frontier_sound_complete` passes on restricted complete fixtures.
- Incomplete budgets return `frontier_incomplete` with explicit outstanding region.

---

## Epic 9: Identity, receipts, and evidence surfaces

### Issue 0901 - Implement domain-separated identities and canonical descriptor serialization

Suggested labels: `identity`, `security`, `core`

Problem:
All archive/query/certificate identities depend on canonical domain-separated preimages.

Tasks:
1. Implement all mandatory identity preimages from section 9.1.
2. Implement restricted canonical JSON/JCS helpers for descriptor surfaces.
3. Bind wire identity boundaries explicitly in receipts.

Definition of Done:
- Identity computations are deterministic and collision-safe in behavior assumptions.

Acceptance Criteria:
- Identity fixtures and mismatch negatives pass.
- `UC-ID-01`/`UC-ID-02` scenarios map to tests.

---

### Issue 0902 - Implement reconstruction receipt schema `uorc/reconstruction-receipt/2`

Suggested labels: `evidence`, `api`, `verification`

Problem:
Receipts must separately represent decoding integrity, full comparison, and certificate status.

Tasks:
1. Implement receipt generation for compress/decompress/verify flows.
2. Ensure no unearned status fields are set.
3. Include abstract usage and deployment evidence references.

Definition of Done:
- Receipt is canonical, typed, and complete for required outcomes.

Acceptance Criteria:
- Positive and negative receipt fixtures validate against schema.
- Missing original input cannot produce `full_byte_equal` comparison claim.

---

## Epic 10: Checkpoint/replay and resume

### Issue 1001 - Implement replay checkpoint schema `uorc/replay-checkpoint/2`

Suggested labels: `resume`, `checkpoint`, `core`

Problem:
Checkpoint format is restricted to replay-safe fields only.

Tasks:
1. Implement checkpoint emit/parse/validate for allowed field set.
2. Reject trusted-state imports (visited sets, opaque cursors, etc.).
3. Bind checkpoint to exact query/policy/source identities.

Definition of Done:
- Checkpoint files are portable and auditable by design.

Acceptance Criteria:
- Stale/forged/mutated checkpoint tests fail with `UC4008` or `UC4007` as appropriate.

---

### Issue 1002 - Implement resume with deterministic replay reconstruction

Suggested labels: `resume`, `scheduler`, `verification`

Problem:
Resume must replay to prefix position before consuming new work.

Tasks:
1. Reconstruct from initial state and replay recorded logical work.
2. Enforce explicit replay budget and status semantics.
3. Continue search only after successful replay reconstruction.

Definition of Done:
- Resume preserves deterministic logical state law.

Acceptance Criteria:
- `checkpoint_resume` theorem/tests pass including before/after candidate commit boundaries.

---

## Epic 11: Public computational API and CLI

### Issue 1101 - Implement modeled computational API roots (`compress`, `decompress`, ...)

Suggested labels: `api`, `product`, `core`

Problem:
Spec defines exact semantic entry points and result surfaces.

Tasks:
1. Implement required API roots and typed inputs/outputs.
2. Preserve separation of semantic failures vs operational failures.
3. Add conformance tests for each root including negative branches.

Definition of Done:
- API roots match section 15 contracts and are generated/exported through PrismPM.

Acceptance Criteria:
- All roots are callable from generated interfaces with modeled result types.

---

### Issue 1102 - Implement native `uorc` command surface and argument semantics

Suggested labels: `cli`, `product`, `io`

Problem:
CLI grammar/exit behavior is normative and extensive.

Tasks:
1. Implement command parsing for all required commands/options.
2. Enforce option conflicts, typed decimal parsing, and stream constraints.
3. Emit exact exit categories + specific diagnostics.

Definition of Done:
- CLI behavior matches section 16 command and exit semantics.

Acceptance Criteria:
- Command matrix tests cover required success/failure permutations.
- Unknown/missing/conflicting args fail with correct typed codes.

---

### Issue 1103 - Implement transactional publication semantics (file and directory)

Suggested labels: `filesystem`, `safety`, `cli`

Problem:
Default client must publish only validated outputs with atomic no-replace semantics.

Tasks:
1. Stage outputs in temp path in same destination directory.
2. Publish only after full validation and integrity success.
3. Handle partial sidecar publication status exactly (`UC6002`).

Definition of Done:
- No partial or unsafe overwrite behavior under failure races.

Acceptance Criteria:
- Failure-injection tests show no published partial payloads under target names.

---

### Issue 1104 - Implement corpus manifest IO and `compress-many`/`decompress-many`

Suggested labels: `corpus`, `cli`, `io`

Problem:
Ordered object corpus operations are first-class and have strict path rules.

Tasks:
1. Implement `uorc/input-manifest/1` parser/validator.
2. Reject forbidden path semantics (symlink/path traversal unless authorized).
3. Preserve object order and boundary behavior; publish directory atomically.

Definition of Done:
- Corpus operations are complete and safe under profile defaults.

Acceptance Criteria:
- Manifest/path negative tests cover duplicates, aliasing, traversal, and symlink behavior.

---

## Epic 12: Prism View/Core-Wasm/Hologram integration

### Issue 1201 - Implement bounded View transport protocol (`C1/D1/I1`)

Suggested labels: `view`, `transport`, `hologram`

Problem:
View transport is bounded ASCII/hex protocol with strict validation behavior.

Tasks:
1. Implement request parser and bounded response serializer.
2. Enforce payload limits and lowercase-even-hex requirements.
3. Map errors to canonical JSON responses without binary stream mixing.

Definition of Done:
- View protocol behaves exactly per section 17.2/17.3.

Acceptance Criteria:
- Transport edge tests cover malformed forms, limits, and UTF-8 constraints.

---

### Issue 1202 - Complete generated Core-Wasm and Hologram acceptance vectors

Suggested labels: `wasm`, `integration`, `verification`

Problem:
Generated artifacts must be exercised through independent Hologram verification.

Tasks:
1. Wire required roots into application model exports.
2. Execute vectors via direct guest invocation and generated View path.
3. Include all 256-byte value transport and boundary/detach/shutdown cases.

Definition of Done:
- Application verification transcript demonstrates complete required vector coverage.

Acceptance Criteria:
- Missing/truncated response paths fail acceptance as required.

---

## Epic 13: External authorities and oracle harness

### Issue 1301 - Implement authority manifest acquisition and freezing process

Suggested labels: `oracles`, `supply-chain`, `compliance`

Problem:
External tools/corpora must be immutable and provenance-documented.

Tasks:
1. Implement machine-readable manifest schema and inventory fields.
2. Freeze source + digest + license + usage roles for each authority.
3. Enforce offline verification execution after setup.

Definition of Done:
- Authority inventory can be reconstructed and audited independently.

Acceptance Criteria:
- Missing digest/provenance constraints are rejected by gate.

---

### Issue 1302 - Integrate Lean kernel replay and axiom audit evidence

Suggested labels: `proof`, `oracles`, `lean`

Problem:
Theorem evidence requires real elaboration + separate-process checker + axiom policy.

Tasks:
1. Wire elaboration, `leanchecker`, and per-declaration axiom audit.
2. Fail on forbidden proof shortcuts and unresolved obligations.

Definition of Done:
- Proof evidence is deterministic and policy-enforced.

Acceptance Criteria:
- Intentional forbidden construct mutations are caught by owning gate.

---

### Issue 1303 - Integrate NIST SHA-256 vector oracle checks

Suggested labels: `crypto`, `oracles`, `validation`

Problem:
Identity and integrity surfaces rely on exact hashing implementation checks.

Tasks:
1. Acquire/lock required CAVP vectors.
2. Implement vector importer/completeness validation.
3. Execute vectors against runtime implementation path used by product.

Definition of Done:
- Hash implementation boundary has independent vector evidence.

Acceptance Criteria:
- Dropped/malformed vector subsets are rejected as failures.

---

### Issue 1304 - Integrate Z3 and cvc5 oracle suites and strict parsers

Suggested labels: `solver`, `oracles`, `validation`

Problem:
Two independent solver paths are required for arithmetic/semantic micro-checks.

Tasks:
1. Implement typed SMT request generation and strict result parsing.
2. Run required deterministic suites across both solvers.
3. Handle `unknown`/timeout/unavailable as incomplete, not success.

Definition of Done:
- Solver-backed validation is complete for mandated suites.

Acceptance Criteria:
- Expected sat/unsat disagreements are surfaced as gate failures.

---

### Issue 1305 - Integrate independent Wasm runtimes (spec interpreter + Wasmtime)

Suggested labels: `wasm`, `oracles`, `runtime`

Problem:
Generated Wasm behavior must match across independent runtime authorities.

Tasks:
1. Acquire/pin runtimes and applicable official tests.
2. Execute same UORC exports across runtimes and compare observables.
3. Capture feature-profile compatibility constraints.

Definition of Done:
- Runtime agreement evidence is explicit and reproducible.

Acceptance Criteria:
- Feature gaps produce target ineligibility failure, not skipped success.

---

### Issue 1306 - Integrate external byte comparison oracle for IO boundary checks

Suggested labels: `io`, `oracles`, `verification`

Problem:
Need external full-byte equality checks outside codec core for transport/filesystem mistakes.

Tasks:
1. Add immutable comparator binding (e.g., `cmp`) and hash utility binding.
2. Use comparator in reconstruction verification transcripts.

Definition of Done:
- File boundary validation cannot be reduced to hash-only checks.

Acceptance Criteria:
- Deliberate mismatch fixtures are caught with precise diagnostics.

---

## Epic 14: Benchmarking and performance program

### Issue 1401 - Implement benchmark plan schema `uorc/benchmark-plan/2`

Suggested labels: `benchmarks`, `schema`, `metrics`

Problem:
Benchmark execution and claims require a sealed, versioned plan surface.

Tasks:
1. Implement plan schema and parser/validator.
2. Encode accounting scopes/basis/delivery bindings explicitly.
3. Enforce run-order/repetition/cache policy declarations.

Definition of Done:
- Benchmarks cannot run with ambiguous scope or hidden comparator settings.

Acceptance Criteria:
- Invalid/missing plan fields fail closed with typed diagnostics.

---

### Issue 1402 - Implement required corpus acquisition and sealing workflow

Suggested labels: `benchmarks`, `data`, `compliance`

Problem:
Mandatory corpus suites must be immutable and verified before measurement.

Tasks:
1. Acquire and lock `wire-boundaries`, `native-structure`, `enwik8`, `silesia`, holdout suite.
2. Record identity checks and rights/provenance constraints.
3. Enforce no silent transformation or post-seal mutation.

Definition of Done:
- Benchmark input set is sealed and reproducible.

Acceptance Criteria:
- Corpus inventory gate detects missing/reordered/modified members.

---

### Issue 1403 - Integrate external comparator toolchain (zstd, brotli, xz, cmix)

Suggested labels: `benchmarks`, `comparators`, `oracles`

Problem:
Public-comparison plans require immutable comparator bindings and reconstruction checks.

Tasks:
1. Acquire pinning + license/provenance manifests for each comparator.
2. Implement invocation adapters for default and high-ratio plans.
3. Validate each comparator output by full reconstruction compare.

Definition of Done:
- Comparator results are complete, reproducible, and not silently omitted.

Acceptance Criteria:
- Failed/time-out comparators are reported explicitly, never dropped.

---

### Issue 1404 - Implement early feasibility gate and reporting

Suggested labels: `benchmarks`, `feasibility`, `risk`

Problem:
Spec requires pre-release feasibility evidence for synthesis/resource behavior.

Tasks:
1. Run required scale ladder and store trace tests.
2. Produce measured reports including candidate counts and work partitions.
3. Classify outcomes exactly (`compressed`, `raw_best_found`, etc.).

Definition of Done:
- Feasibility output identifies blockers without claim inflation.

Acceptance Criteria:
- Reports include both logical and physical observations with exact status semantics.

---

### Issue 1405 - Execute mandatory public benchmark plans and publish results

Suggested labels: `benchmarks`, `reporting`, `release`

Problem:
Implementation acceptance requires running complete mandatory plans with honest reporting.

Tasks:
1. Execute `public-default` and `public-ratio` plans.
2. Publish canonical results including accounting scope and claim disposition.
3. Update open-claim ledger statuses without overstatement.

Definition of Done:
- Full mandatory benchmark evidence exists in repository artifacts.

Acceptance Criteria:
- Results include unsuccessful outcomes where present.
- No benchmark omission occurs without explicit typed failure status.

---

## Epic 15: Diagnostics and negative coverage completeness

### Issue 1501 - Implement complete `UCxxxx` diagnostic inventory and schemas

Suggested labels: `diagnostics`, `api`, `quality`

Problem:
Public failures must be typed, stable, and field-scoped.

Tasks:
1. Implement all diagnostic classes/codes from section 18.2.
2. Add stable reason enums and stage/field metadata.
3. Document mapping to owning claims/scenarios.

Definition of Done:
- Public errors are deterministic, typed, and bounded.

Acceptance Criteria:
- No fallback generic error for covered paths.
- Error text/content does not leak unbounded external outputs.

---

### Issue 1502 - Add one owning negative test for every public diagnostic

Suggested labels: `testing`, `diagnostics`, `conformance`

Problem:
Every diagnostic must be reachable by its own intentionally targeted failure.

Tasks:
1. Add negative fixtures for each `UC` code.
2. Ensure failure reaches intended validator stage (not earlier parser failure).
3. Verify tests fail red/green on planted mutations.

Definition of Done:
- Diagnostic coverage is complete and non-vacuous.

Acceptance Criteria:
- Matrix report maps each diagnostic to at least one dedicated test case.

---

## Epic 16: Formal theorem closure

### Issue 1601 - Implement and check full theorem inventory from section 19

Suggested labels: `proof`, `formal`, `acceptance`

Problem:
Spec mandates named theorem inventory with specific statements.

Tasks:
1. Implement all required theorem statements and proofs.
2. Ensure theorem names/mappings remain traceable from docs/register.
3. Run kernel checking and axiom audits in gate.

Definition of Done:
- Theorem inventory is complete and checked under declared policy.

Acceptance Criteria:
- No missing theorem from table in section 19.
- No forbidden axioms/holes/tactic escapes.

---

### Issue 1602 - Add kernel-instance fixtures for required closed examples

Suggested labels: `proof`, `fixtures`, `evidence`

Problem:
Certain fixtures require closed instance modules verified via Lean and checker.

Tasks:
1. Generate instance modules binding exact query/archive/certificate bytes.
2. Verify with elaboration + `leanchecker`.
3. Report runtime-check vs kernel-instance evidence kinds distinctly.

Definition of Done:
- Required kernel-instance evidence exists and is reproducible.

Acceptance Criteria:
- Fixtures flagged `kernel_instance` include actual checked modules and transcripts.

---

## Epic 17: CI, release gate, reproducibility, mutation testing

### Issue 1701 - Expand `just vv` to include full UORC mandatory gates

Suggested labels: `ci`, `verification`, `release`

Problem:
Current scaffold gate does not yet include UORC-specific proof/oracle/benchmark obligations.

Tasks:
1. Add all mandatory phases from section 25 in logical dependency order.
2. Keep `just vv` as single complete acceptance boundary.
3. Provide subset commands clearly marked non-release.

Definition of Done:
- `just vv` enforces all required UORC acceptance obligations.

Acceptance Criteria:
- Release readiness cannot be asserted without green `just vv`.

---

### Issue 1702 - Implement mutation-evidence suite tied to owning gates

Suggested labels: `ci`, `mutation`, `quality`

Problem:
Every critical gate must prove it can fail on representative defects.

Tasks:
1. Define mutation classes per section 24/26 requirements.
2. Add automation that runs red/green mutation checks.
3. Record evidence mapping mutation to owning gate.

Definition of Done:
- Gate quality is demonstrably non-vacuous.

Acceptance Criteria:
- Mutation matrix includes expected rejecting gate for each class.

---

### Issue 1703 - Add clean-path reproducibility and identity comparison checks

Suggested labels: `reproducibility`, `ci`, `supply-chain`

Problem:
Build reproducibility and identity boundaries must be checked from clean roots.

Tasks:
1. Execute dual absolute-path clean builds with isolated outputs.
2. Compare canonical artifacts and declared platform-bound normalized artifacts.
3. Reject disallowed normalization masking semantic drift.

Definition of Done:
- Reproducibility checks are automated and policy-enforced.

Acceptance Criteria:
- Deliberate semantic drift mutation is detected despite normalization.

---

## Epic 18: Release readiness and publication evidence

### Issue 1801 - Produce UORC release acceptance dossier for v1 profile-02

Suggested labels: `release`, `compliance`, `documentation`

Problem:
Need one auditable package proving all mandatory implementation obligations are complete.

Tasks:
1. Assemble proof/oracle/benchmark/claim inventory summary.
2. Include unresolved/open claims explicitly.
3. Bind dossier to exact source/compiler/SDK identities.

Definition of Done:
- Reviewers can decide implementation acceptance from one canonical dossier.

Acceptance Criteria:
- Dossier includes explicit yes/no for each mandatory requirement class.
- Open performance/record claims remain open unless evidence changes their state.

---

## Issue creation sequencing

Recommended creation order:
1. Epic 0-2 first (repository + locks + baseline docs).
2. Epic 3-6 (model + wire + semantics + resource safety).
3. Epic 7-10 (encoder/search/cert/replay).
4. Epic 11-13 (APIs/integration/oracles).
5. Epic 14-18 (benchmarks, theorem closure, final acceptance).

Use GitHub milestones:
- `M0 Bootstrap and Rebrand`
- `M1 Semantic Core`
- `M2 Search and Certification`
- `M3 Product Interfaces`
- `M4 Oracle and Benchmark Evidence`
- `M5 Release Acceptance`

