<!-- @generated from src/Uorc/Specification.lex.tex and src/Uorc/Registry.lex.tex. -->
<!-- Regenerate with `just model-write`; do not edit by hand. -->

# UORC verification summary

This project-owned summary states evidence boundaries from the LexLean
authority graph. It does not replace the template-owned `VERIFICATION.md`.

## Current status

This repository currently establishes only the charter and source-authority slice represented by its registered conformance rows. Compression behavior, losslessness, optimality certificates, benchmark superiority, and record status are not established by this charter slice.

## Claim boundaries

| Claim | Meaning |
| --- | --- |
| Implementation acceptance | All mandatory current-build semantic, proof, oracle, integration, security, reproducibility, and inventory obligations pass. It does not imply a compression record. |
| Losslessness | A successful encoder result resolves to the original ordered byte objects under the formal UORC semantics; deployment retains the declared compiler and runtime trust boundary. |
| Scoped minimum certification | A checked certificate establishes minimum size only over its exact sealed admissible universe and objective. It is not an unrestricted shortest-program claim. |
| Benchmark improvement | Measured compression improvement is a result of an exact sealed benchmark plan and accounting scope. It is not implied by implementation acceptance or proof completion. |
| External record acceptance | Record status exists only when the identified external authority accepts the measured result under that authority's rules. |

## Evidence and dependency boundaries

| Class | Boundary |
| --- | --- |
| Production dependencies | Shipped execution may use only locked production dependencies explicitly bound by the SDK and release inventory. |
| Validation authorities | External solvers, runtimes, test vectors, comparators, byte-comparison tools, and prior-art sources are validation or context authorities only when separately modeled and immutably bound; they are never UORC production behavior unless explicitly modeled and locked as production dependencies. |
| Authority bindings | This charter classifies external-authority roles but binds no external authority by itself. Exact citations, revisions, digests, licenses, and oracle interfaces become repository-cited authorities only when authored as LexLean authority rows and projected into the generated authority register. |
| Measurements | Timing, peak memory, compression ratios, benchmark outcomes, and record attempts are measurements. They are not semantic definitions or mathematical proofs. |
| Open research claims | Compression superiority, novelty beyond cited prior art, and external record status remain open until their exact evidence requirements are met. |

## Source and projection checks

- `lexlean lock --check` binds the authority project to the exact compiler semantics and workspace inputs.
- `lexlean check --all` requires the locked LexLean compiler to accept every authority module before repository projection checks run.
- `cargo xtask check-model` extracts the admitted authority records and rejects stale generated projections.
- `cargo xtask audit-source-authority` rejects a second handwritten UORC specification or BCP-14 semantic rule in project prose.
- `just bdd` reconciles every registered ID with its scenario, honesty level, and executed test root.
- `just vv` remains the only complete implementation-acceptance boundary.
