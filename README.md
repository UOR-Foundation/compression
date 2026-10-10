<!-- @generated from src/Uorc/Specification.lex.tex and src/Uorc/Registry.lex.tex. -->
<!-- Regenerate with `just model-write`; do not edit by hand. -->

# UORC — Universal Object Reference Compression

A lossless compression system whose compressed object is an exact executable Universal Object Reference synthesized under explicit accounting and evidence.

## Current status

This revision implements the charter, source-authority, claim-disposition, and bounded U64 ULEB128 slices in the LexLean graph. The codec has kernel-checked round-trip, shortest-length, exact-size, and prefix-consumption properties. Full archive, machine, search, and production-product acceptance remain separate registered work; performance and record claims remain open.

## Authority

The project-owned semantic and claim authority is the LexLean graph rooted at src/Uorc/Specification.lex.tex and src/Uorc/Registry.lex.tex. Generated Markdown and model TOML files are projections, not independent authority.

Inherited template policy and the implementation-language restriction precede imported authorities; UORC LexLean semantics precede generated projections; observations and measurements cannot strengthen definitions or proof claims.

Product semantics, claim distinctions, test and report intent, and generated project documentation originate in LexLean. PrismPM and template tooling may compile, project, package, and validate that graph but may not add UORC behavior.

## Scope

Lossless byte-object compression, exact reconstruction, typed reference synthesis, explicit accounting, scoped certification, and reproducible validation.

### Non-scope

Version 1 does not assert a world record, an unrestricted Kolmogorov minimum, universal runtime optimality, learned neural weights, online object lookup, encryption, or a compiler theorem not supplied by the locked upstream toolchain.

## Claim boundaries

| Claim | Meaning |
| --- | --- |
| Implementation acceptance | All mandatory current-build semantic, proof, oracle, integration, security, reproducibility, and inventory obligations pass. It does not imply a compression record. |
| Losslessness | A successful encoder result resolves to the original ordered byte objects under the formal UORC semantics; deployment retains the declared compiler and runtime trust boundary. |
| Scoped minimum certification | A checked certificate establishes minimum size only over its exact sealed admissible universe and objective. It is not an unrestricted shortest-program claim. |
| Benchmark improvement | Measured compression improvement is a result of an exact sealed benchmark plan and accounting scope. It is not implied by implementation acceptance or proof completion. |
| External record acceptance | Record status exists only when the identified external authority accepts the measured result under that authority's rules. |

## Evidence classification

This table classifies evidence within its verified scope. It does not validate an evidence artifact or set whole-release acceptance. Registered ledger entries are obligations, not current-build execution receipts.

| Evidence | Supported claim | Level | Research disposition | Implementation evidence eligible |
| --- | --- | --- | --- | --- |
| `none` | `open_claim` | `open` | `not_measured` | false |
| `imported_fact` | `imported_fact` | `some-true` | `not_measured` | true |
| `build_evidence` | `build_evidence` | `build` | `not_measured` | true |
| `kernel_theorem` | `kernel_theorem` | `build` | `not_measured` | true |
| `fixture_result` | `reconstruction_checked` | `build` | `not_measured` | true |
| `scoped_minimum_certificate` | `minimum_certified` | `build` | `not_measured` | true |
| `benchmark_target_unmet` | `benchmark_result` | `open` | `measured_target_unmet` | false |
| `benchmark_target_met` | `benchmark_result` | `open` | `measured_target_met` | false |
| `external_record_acceptance` | `record_validated` | `open` | `externally_accepted_record` | false |
| `version_metadata` | `open_claim` | `open` | `not_measured` | false |

## Current research dispositions

- `UC-PERF-01`: `not_measured`; evidence `none`; honesty level `open`.
- `UC-REC-01`: `not_measured`; evidence `none`; honesty level `open`.

The source-owned bounded ULEB128 codec, errors, and proof inventory are projected in [UORC-VARINT.md](UORC-VARINT.md).


## Evidence and dependency boundaries

| Class | Boundary |
| --- | --- |
| Production dependencies | Shipped execution may use only locked production dependencies explicitly bound by the SDK and release inventory. |
| Validation authorities | External solvers, runtimes, test vectors, comparators, byte-comparison tools, and prior-art sources are validation or context authorities only when separately modeled and immutably bound; they are never UORC production behavior unless explicitly modeled and locked as production dependencies. |
| Authority bindings | This charter classifies external-authority roles but binds no external authority by itself. Exact citations, revisions, digests, licenses, and oracle interfaces become repository-cited authorities only when authored as LexLean authority rows and projected into the generated authority register. |
| Measurements | Timing, peak memory, compression ratios, benchmark outcomes, and record attempts are measurements. They are not semantic definitions or mathematical proofs. |
| Open research claims | Compression superiority, novelty beyond cited prior art, and external record status remain open until their exact evidence requirements are met. |

## Research position

SEQUITUR and Re-Pair are the named grammar-compression prior-art context; DreamCoder and Stitch are the named abstraction and library-learning prior-art context. This charter records the novelty boundary only: exact citation and provenance bindings are separate authority rows, and UORC does not claim generative programs, shared rules, or anti-unification alone are novel.

The research hypothesis is that joint synthesis of typed address generators, sequential state evolution, shared parameters, and serialized residuals under complete representation accounting can improve lossless compression on declared workloads.

Novelty is not asserted by implementation acceptance and requires separate evidence against the cited prior art.

Compression superiority and record status are not asserted by correctness or proof completion; they require sealed benchmark evidence or external record acceptance.

## Repository evidence

- [CONFORMANCE.md](CONFORMANCE.md) is the generated claim projection.
- [UORC-VERIFICATION.md](UORC-VERIFICATION.md) is the generated project verification summary.
- [AGENTS.md](AGENTS.md), [TEMPLATE-CONTRACT.md](TEMPLATE-CONTRACT.md), and [VERIFICATION.md](VERIFICATION.md) are inherited repository policy.
- `just vv` is the complete repository acceptance gate; targeted commands are subsets only.
