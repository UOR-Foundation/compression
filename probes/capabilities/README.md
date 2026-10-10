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
oracle after removing only the conflicting positive-case theorem. The
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
failed with `ERR_MODULE_NOT_FOUND`. After implementation, eight reporting tests
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
The source proof is being checked with the admitted `reflexivity` proof form;
this is not yet a passing verification result. The store project genuinely
rejected language 1.2 with LLC0001, advertising only languages 1.0 and 1.1.
Exporter and runtime stages did not execute in that run.

The preserved artifact is `sdk-capability-observations`, ID `11659925500`,
SHA-256 `4c77bd61c9041b2497bcbbb8c7c7f6bc6d31849247aa134b0c90727beb73b618`.
[Workflow run](https://github.com/UOR-Foundation/compression/actions/runs/38023397472)
records the exact source and uploaded evidence. The runner now retains every
regular LexLean build artifact before verification, including on failure.
