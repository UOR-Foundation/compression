# `just vv` is the normative acceptance gate. Everything else is a slice of it.

# LexLean scheduling only (SPEC.md §22.11): these select how many proof
# processes run at once and within what summed expected memory; they are no
# input of any verification and change no artifact. The hints are the
# observed per-module wall time and peak of a complete verification.
export LEXLEAN_VERIFY_WIDTH := env_var_or_default("LEXLEAN_VERIFY_WIDTH", "4")
export LEXLEAN_VERIFY_MEMORY_KIB := env_var_or_default("LEXLEAN_VERIFY_MEMORY_KIB", "9437184")
export LEXLEAN_VERIFY_HINTS := env_var_or_default("LEXLEAN_VERIFY_HINTS", justfile_directory() + "/validation/lexlean-verify-hints.json")

default: vv

# The whole gate.
vv: lexlean-authority template-check fmt-check model lint test features bdd deny lexlean-artifacts claim-dispositions varint-proofs reproducibility integrity formal-tests
    @echo "vv: the acceptance gate passed"

# The UORC authority files are real LexLean source, not merely files with a
# LexLean suffix. The locked compiler must accept the exact project before the
# Rust projection/gate layer is allowed to consume it.
lexlean-authority:
    lexlean lock --check
    lexlean fmt --check --all
    lexlean check --all

# R1, R4, R5 --- the repository gates, each falsifiable.
model:
    cargo run --locked -q -p xtask -- validate

# The hand-reviewed trust root is checked independently of generated project
# content. PrismPM then validates the canonical contract and both locks.
template-check:
    cargo run --locked -q -p xtask -- audit-bootstrap
    prismpm template check
    prismpm lock check

# Regenerate every model/document projection owned by the LexLean authority graph.
model-write:
    cargo run --locked -q -p xtask -- check-model --write

# Direct R1 report: project prose cannot become a second UORC authority.
source-authority:
    cargo run --locked -q -p xtask -- audit-source-authority

# Without --all: that also walks path dependencies, and the exported client
# crates under generated/rust keep LexLean's layout, bound by their provenance.
fmt:
    cargo fmt

fmt-check:
    cargo fmt -- --check

lint:
    cargo clippy --locked --workspace --all-targets -- -D warnings

# The product and repository crates. The xtask suite drives the formal
# runtimes (tens of minutes) and is its own recipe, `formal-tests`.
test:
    cargo test --locked --workspace --exclude xtask

# The xtask suite: product-runtime, integrity, claims and offline harnesses.
formal-tests:
    cargo test --locked -p xtask

# A feature only its author has built is a feature that does not work: nothing
# else in the gate compiles a crate at anything but its default features, so a
# rename upstream of an optional dependency fails nowhere until someone turns
# the flag on. `--all-targets` because the tests behind a flag are code too.
#
# Every optional feature compiles, with its tests.
features:
    cargo check --locked --workspace --all-features --all-targets

# R3: every capability begins as a Gherkin scenario, and every scenario has a
# test whose name ends in its ID.
bdd:
    cargo test --locked -p repo-conformance

# R6: nothing shipped depends on a dev-only crate, no wildcard version
# requirement, no advisory against anything in the tree. `cargo-deny` is
# supplied by the locked SDK, so this is part of `just vv`.
#
# Advisories, bans, licences and sources, over the dependency graph.
deny:
    node scripts/offline-validation.mjs deny

# Full formal construction/replay of the declared charter authority.
lexlean-artifacts:
    lexlean build --all
    lexlean verify --all

# Execute every source-owned promotion and verify every restored control.
claim-dispositions:
    node scripts/claims-validation.mjs

# Always executed, including in the two complete offline CI invocations.
# This invokes model/build in clean roots, never recursively `just vv`.
reproducibility:
    node scripts/offline-validation.mjs reproduce

# Actual core-source defects must fail at their owning kernel declarations.
varint-proofs:
    node scripts/varint-validation.mjs

# Source-owned SHA, streaming identities, independent data and mutation controls.
integrity:
    node --test scripts/integrity-vectors.test.mjs
    node scripts/integrity-validation.mjs
    node --test scripts/integrity-stream.test.mjs
    node scripts/integrity-stream.mjs
    node --test scripts/integrity-binding.test.mjs
    node --test scripts/integrity-transcript.test.mjs
    node --test scripts/integrity-canonical.test.mjs
    node --test scripts/integrity-consumer-components.test.mjs
    node scripts/integrity-canonical-runtime.mjs
