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
# PrismPM's native regeneration runs model-write in a copy of the tree staged
# in the SDK's /tmp, a 2 GiB tmpfs of which CARGO_HOME already holds 1.1 GiB,
# with a cleared environment. The note .tmp/model-write-target travels with
# the copy and sends its build to a target directory of its own on this
# workspace's disk. It is not the workspace's target: a binary built from the
# copy carries the copy's paths, and cargo would take it as current here.
template-check:
    cargo run --locked -q -p xtask -- audit-bootstrap
    mkdir -p .tmp && printf '%s' "{{justfile_directory()}}/target/regeneration" > .tmp/model-write-target
    prismpm template check
    rm -f .tmp/model-write-target
    prismpm lock check

# Regenerate every model/document projection owned by the LexLean authority graph.
model-write:
    cargo run --locked -q -p xtask --target-dir "$(cat .tmp/model-write-target 2>/dev/null || echo target)" -- check-model --write

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
    node --test scripts/site.test.mjs

# UC-WEB-01: build the Pages site as the release does (a stand-in benchmark
# page beside it) and walk every journey of the page's Prism product model in
# headless Chrome. CI runs the same steps in pages-client.yml.
pages:
    cargo build --locked --release -p uorc-wasm --target wasm32-unknown-unknown
    mkdir -p .tmp/bench && for f in bench.html bench.json bench.md; do printf 'benchmarks are published with each release\n' > .tmp/bench/$f; done
    node scripts/site.mjs .tmp/site --wasm target/wasm32-unknown-unknown/release/uorc_wasm.wasm --version "$(git describe --tags --always)-dev" --bench .tmp/bench
    node scripts/page-acceptance.mjs .tmp/site --report .tmp/page-acceptance.json

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

# Source-owned SHA-256 and bounded chain: independent data and mutation controls.
integrity:
    node --test scripts/integrity-vectors.test.mjs
    node scripts/integrity-validation.mjs

# Developer and agent measurements (scripts/lab.mjs; no CI job runs them).
# Host use: wall, CPU, busy cores over time and peak RSS at 1 and N threads.
lab-hosts *ARGS:
    node scripts/lab.mjs hosts {{ARGS}}

# A/B against a reference rev or binary: `just lab-ab --ref main [--identical]`.
lab-ab *ARGS:
    node scripts/lab.mjs ab {{ARGS}}

# Per-line instruction cost of a 512 KiB decode under cachegrind.
lab-cost *ARGS:
    node scripts/lab.mjs cost {{ARGS}}
