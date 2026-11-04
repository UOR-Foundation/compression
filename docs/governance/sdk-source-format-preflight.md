<!-- uorc:non-authoritative -->
# Negative-only SDK source-format preflight (issue #86)

This is operational CI evidence, not a source of product semantics or an SDK
acceptance result. The actual locked compiler and complete `just vv` remain the
acceptance boundary.

## Observed incompatibility and authority

At source revision `5a01915fb467652b28a9fa11339bbea614f8b280`,
[bootstrap run 41931828522](https://github.com/UOR-Foundation/compression/actions/runs/41931828522)
and [offline run 41931828519](https://github.com/UOR-Foundation/compression/actions/runs/41931828519)
pulled SDK index
`sha256:60226bc791d4c0e5613402a6be7e63f4963d3faf7f327befcf56fc0e41d0ce21`
and failed `lexlean fmt` with `LLP2003: expected semanticdata, found inductive`
at `src/Uorc/Wire/ProgramParseScenarios.lex.tex:15`. The project selects
language 1.1, whereas this module begins its semantic body with native notation.

The official [LexLean specification, section 17.21.1](https://github.com/UOR-Foundation/LexLean/blob/71defb4c663aaec5b9a3aab04cbc6a03e7106227/SPEC.md#17211-where-it-is-written)
states that language 1.1 admits the data form only and refuses native notation
with `LLP2003`. This is a documented language distinction, not a claim that a
newer compiler or a source conversion has been qualified here. Official
[source discovery](https://github.com/UOR-Foundation/LexLean/blob/71defb4c663aaec5b9a3aab04cbc6a03e7106227/crates/lexlean/src/project.rs#L417-L479)
rejects source-root symlinks before checking the `.lex.tex` suffix.

## Deliberately incomplete prediction

`scripts/sdk-source-format-preflight.py` uses Python's standard TOML reader to
identify `lexlean/project/1`, language 1.1 and its configured source roots. It
recognizes only a complete anchored conventional header: `begin{lexlean}` with
an ASCII module name, zero or more `importmodule` lines, the exact
`title{Parameters}`, and `begin{semanticmodule}`, followed by spaces/newlines
and an initial `inductive{`, `structure{`, or `definition{` declaration.

It does not search arbitrary prose or payloads, strip comments, decode module
schemas, check declaration bodies, or decide whether any source is accepted.
The initial declaration token predicts the language refusal regardless of the
rest of that declaration. A different title/header, comments before a token,
quoted examples, unfamiliar declarations, malformed/unknown configuration,
other language versions and an incomplete inspected prefix delegate unchanged
to the real compiler. In particular, language 1.2 receives no source-format
verdict from this guard. A zero exit status never means format or semantic
acceptance.

The guard reports the first recognized mismatch in its bounded sorted walk.
For this revision it finds the same incompatibility earlier at
`src/Uorc/Core/Heap.lex.tex:5`; the saved independent fixture retains the
original `ProgramParseScenarios.lex.tex:15` failure.

## Inspection bounds and safety

Default inspection budgets are 64 KiB of configuration, at most 16 KiB per
source prefix, 4 MiB of total bytes read, 4,096 directory entries and depth 64.
Directory inventory is stopped before materializing unbounded entries.
Exhausting any inspection budget delegates to the unchanged SDK rather than
rejecting an input for its size. These are costs of this optional prediction,
not new product limits, and they do not change configured LexLean limits.

Source paths are opened relative to held directory descriptors with
`O_NOFOLLOW`; symlink ancestors, escaping/absolute/noncanonical root paths and
special source-root files are refused, without reading their target. Missing,
unreadable or resource-unavailable paths delegate rather than being labeled a
format mismatch. No proof/build/acquisition artifact is consumed as evidence.

Operational diagnostics are distinct:

- `UCF001`, exit 1: exact relative path, line and known language-1.1/native
  incompatibility. This is a negative prediction of `LLP2003`.
- `UCF002`, exit 2: unsafe path/type refusal, not a compiler format verdict.
- `UCF003`, exit 2: invalid preflight invocation; filters/options are unsupported.

## Integration and ownership limit

The project-owned `.github/workflows/offline-validation.yml` runs all guard
controls and then this negative preflight immediately after exact-revision,
credential-free checkout in both jobs. A rejection stops that job before its
Docker pull. A passing preflight continues to the existing exact-SDK projection
command or isolated full-acceptance runner. The review job and the two offline
matrix jobs therefore avoid three known-failing pulls for this incompatibility.
No required job identity, SDK command, network isolation, credential boundary,
proof gate or release protection is changed.

The two independently triggered bootstrap jobs remain unchanged. Their
`.github/workflows/bootstrap.yml` is universal template policy, byte-bound by
`template.lock`; altering it here would violate the ownership contract. This
is not a repository-wide before-Docker gate. No lock, template policy, SDK pin,
product source, generated projection or projection inventory is edited.
Workflows/scripts are project-owned, not outputs of the authority graph's
`crates/model/src/codegen.rs` projection inventory.

## Executed controls and limits

`python3 -B tests/sdk-preflight/run.py` runs the complete named set of 16 tests
and rejects zero/missing/duplicated cases, skipped/expected-failure results,
filters and unsuccessful execution. Controls cover the observed rejection,
language-1.1 data, language-1.2 delegation, comments/quoted text, unrecognized
and malformed headers, prefix/read/traversal budgets, symlink ancestors,
special files, unsafe roots and generic filesystem failures. A fake Docker
executable proves that a known rejection stops before Docker and delegation
continues. Workflow self-audits reject commented, conditional, omitted,
filtered or failure-suppressed guard invocations while retaining the actual
SDK projection and full acceptance commands.

The representative native declaration fails with `UCF001`; replacing it with
language-1.1 `semanticdata` restores delegation and reaches the fake Docker
boundary. All 16 focused tests and all 45 existing `node --test
scripts/*.test.mjs` tests passed locally, with no skipped cases. `git diff
--check` passed. These are host-side operational tests, not SDK qualification.
Cargo, Lean and Docker are unavailable in this executor; `just vv`, native
bootstrap audit, actual locked compiler execution and offline SDK acceptance
were not run here. The current mismatch remains a failure, never a green
unsupported-source result.
