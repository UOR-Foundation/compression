<!-- uorc:non-authoritative -->

# SDK capability execution evidence

The test and report intents are authored in
`src/Uorc/CapabilityContract.lex.tex`. The adjacent LexLean modules contain
actual finite cases, the typed TextApplication bridge, and rejecting cases.
The Node runner only invokes public SDK tools, binds their outputs, and
projects a compatibility report. It does not implement UORC operations,
compile LexLean, supply a replacement exporter, or manufacture SDK results.

Run inside the exact SDK selected by `prismpm.lock`:

```sh
node --test scripts/capability-probes.test.mjs
node scripts/capability-probes.mjs /workspace/.capability-run
```

The output directory must not already exist. The workflow mounts the report
back to the checkout and retains reports, exact command logs and verified SDK
artifacts even when the probe gate fails. It pulls the immutable image before
starting the container with `--network none`.

Every project is initialized and locked by the real LexLean executable. The
TextApplication source and glossary are taken from the digest-checked installed
SDK archive; no local upstream checkout supplies runtime behavior. Actual
`lexlean check/build/verify` and `prismpm check/build/verify` commands produce
separate language, exporter and runtime observations. PrismPM's application
verification runs its generated native corpus, Core-Wasm, browser and Hologram
oracle boundaries. A command exit alone does not establish runtime acceptance:
the report also requires and hashes complete SDK acceptance artifacts.

The arbitrary-binary-response case expects PP2009 from the public text profile.
It establishes that profile's response restriction, not the absence of every
possible SDK byte API. Every registered core acceptance root is negated in its own recorded runtime
fixture; its deliberately false computation is checked by the independent
oracle without removing any declaration. Request-dependent assertions also
exercise parameterized operations explicitly included in the exported root set.
The
corrupted response case reaches the real oracle
only after its unchanged baseline successfully verifies. Missing API bindings,
SDK dependency failures, compiler failures and executed negative cases remain
separate. The mandatory CLI/filesystem adapter binding is still unbound; the
report therefore cannot report complete M0 acceptance.

The persistent-map case is language 1.2. The old locked SDK's LexLean CLI only
advertises languages 1.0 and 1.1, so its actual init rejection is retained.
Even a later successful map case would demonstrate only a pure persistent map;
it would not demonstrate durable filesystem storage. All physical large-input
feasibility remains unmeasured. Large scalar keys and rejected large indices
are never counted as large physical allocations.

## Falsifiability evidence

Before the runner existed, `node --test scripts/capability-probes.test.mjs`
failed with `ERR_MODULE_NOT_FOUND`. After implementation and receipt hardening, seventeen reporting tests
passed. They reject empty observations, a check-only report, zero-exit output
without complete SDK artifacts, and promotion of text-profile acceptance to
arbitrary binary transport. This is reporting-infrastructure evidence only;
it is not an SDK execution result. The tests also reject parent-path and
symlink escapes, preserve every non-selected assertion in a falsifier, and
refuse runtime support when a per-root falsifier has not passed.

The local environment has no `/opt/prismpm/share/inventory.json`. An actual
runner invocation retained that dependency blocker, returned exit 1 and marked
no capability supported. Real compiler/runtime results are supplied only by
the pinned-image workflow, never inferred from these local tests.

## First pinned SDK observation

Actions run `38023397472`, source `4ebae9a8b8e98ed890c288a820e8d04b11ebd550`,
executed in SDK image `60226bc791d4c0e5613402a6be7e63f4963d3faf7f327befcf56fc0e41d0ce21`.
Core initialization, lock, check and build succeeded. Verification rejected
`finiteCases` with LLV7002 because `decide` could not reduce `acceptance = true`.
The store project genuinely
rejected language 1.2 with LLC0001, advertising only languages 1.0 and 1.1.
Exporter and runtime stages did not execute in that run.

The preserved artifact is `sdk-capability-observations`, ID `11659925500`,
SHA-256 `4c77bd61c9041b2497bcbbb8c7c7f6bc6d31849247aa134b0c90727beb73b618`.
[Workflow run](https://github.com/UOR-Foundation/compression/actions/runs/38023397472)
records the exact source and uploaded evidence. The runner now retains every
regular LexLean build artifact before verification, including on failure.

## Kernel-definition and runtime claim boundary

Actions retry `38023861488`, source `eafeeb5a91381e05bc04d29b63de376d61678ad9`,
also rejected the closed equality theorem with LLV7002: `reflexivity` could not
establish `acceptance = true`. Artifact `11659871294`, SHA-256
`7b31ba08bbbb04093b6f5f981a6500b36977b9821a877ceadc527da12de8e210`, preserves
the emitted Lean and exact diagnostics. These two attempted proof forms do
not establish failure of the executable primitives.

The probe now asks the SDK to kernel-check its definitions and then execute
its source-authored assertions. It does not claim a kernel equality theorem
for the observed runtime values. Across the isolated core projects, requests exercise parameterized
bytes, checked U64 arithmetic, records/variants, immutable indexing/slicing
and Scan; the public root list also retains their parameterized operations.
A separate store profile only names declarations present in the store module.
The two new source-shape tests first failed on the extra theorem and missing
store profile, respectively, and pass after these changes. SDK execution of
this updated fixture remains required; the local source-shape tests are not
compiler or runtime evidence.

The TextApplication vectors now also fit the browser request bound of eight
bytes. The quota regression first rejected the previous four-byte browser
bound. The installed profile separately binds Core-Wasm input and output
allocation caps; an exact eight-byte input-allocation vector remains mandatory.
No summed request-plus-response memory guarantee is inferred from those caps.

## Complete-gate integration

The source Registry imports the capability contract and owns UC-SDK-01/02.
Their Rust conformance wrappers execute every reporting test; those tests
never substitute for the separate, mandatory `sdk-capabilities` prerequisite
of `just vv`. The recipe creates a fresh retained observation directory for
every invocation and returns the actual probe exit. Missing mandatory public
bindings remain dependency blockers. A planted omission of this prerequisite
made the full reporting tests fail; the restored configuration passed.

The separate review-only Actions projection job regenerates Registry
projections using the exact SDK, even when full acceptance is blocked. Its
outputs must be imported and checked; no local implementation regenerates
LexLean or model projections.

The next pinned run, `38024870129` at source
`85e53329081cd2a436215dac3ad462345b4144fa`, elaborated the definitions but the
strict axiom audit rejected `UorcProbe.Main.addU64`: LLV7005 reported exactly
`Quot.sound` and `propext` against its default empty declaration. The source
now declares that exact dependency set for checked-U64 helpers and their
transitive callers; verification still requires exact equality with the
kernel's observed set. No allow-all policy or kernel bypass is introduced.
Artifact `11659432434`, SHA-256
`bea22f601139074dcf65ea47b18a5a5aae52c04214c37a04d8c19c023b106063`, retains
the diagnostics and generated definitions. Confirmation of every declared
set and all runtime observations remains required in the next real SDK run.

The review-only job in run `38024870117` generated the three new Registry
projections with the locked SDK. Artifact `11659823271`, SHA-256
`9425e47421c4ea6f5dae4e7362d145695ae1a0778a18f056b2a4fa0b172ee5af`, supplies
the committed CONFORMANCE, ID-register and SDK-capabilities scenario bytes.
Both full offline architectures rejected the probe workflow's missing
`fetch-depth: 0` at the existing bootstrap audit; the workflow now supplies
complete history while still disabling persisted checkout credentials.

## Independent exporter observations

At source `f12db1b5164f25c4dc2f58c51af443fc63c43177`, run `38025277002`
passed complete core kernel verification, imported the locked public
TextApplication model, and passed its model check. The binary-response
negative actually produced PP2009. Export then rejected the checked-U64
specialization used by `countSteps` with PP5004: the call to
`LexLeanRuntime.checkedAdd._at_.UorcProbe.Main.addU64.spec_0` remained external.
Artifact `11659694049`, SHA-256
`a6ac0c9c15284531de430e0cc1624632b182fd59c3734a10ce9b208c10e2e914`, retains
the exact observation. This is an exporter failure, not failed kernel
verification or observed runtime behavior.

Each capability now selects its own authored application and named export
roots in a separate real SDK project. A checked-U64 export failure therefore
cannot prevent an independent bytes or record probe from running. Byte-buffer
assertions use append, length and exact fixture equality without importing
indexing or U64 arithmetic. Every project still kernel-verifies the complete
common Core module, and each selected registered assertion has its own
runtime falsifier. The report attributes each outcome to its modeled project.
The isolation regression first failed on the old shared project, then passed.
Both complete build artifacts and successful kernel-verification artifacts
are now retained and hashed. Execution of these isolated profiles remains
required; a local reporting test never establishes SDK runtime support.

## Receipt identity and process closure

The reporting validator binds the unchanged application build result to the
verify result, acceptance and manifest. The returned attestation ID must hash
the raw manifest bytes, which bind the raw acceptance, LexLean attestation and
model bytes. Kernel source and semantic identities must match the build.
Every one of the thirteen processes emitted by the selected SDK application
verifier must occur once, exit successfully and carry an executable digest.
The oracle process must also contain its verified-footer result. These are
checks of imported SDK evidence; the runner does not reproduce verification.

Three synthetic reporting regressions first failed against the earlier
validator, which admitted unrelated process records and unbound build IDs.
They passed after identity and process closure checks were added. A fourth
rejects kernel-source and oracle mismatches even when synthetic receipt hashes
are recomputed. Seventeen capability/reporting tests and fifteen inherited
offline-boundary tests pass locally. Synthetic receipts establish only
validator behavior, never successful SDK execution.
