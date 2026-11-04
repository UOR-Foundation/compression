# Exact Sequence certificate probe

The smallest confirmed source closure consists of RuntimeValue, Heap and
RuntimeSequence. The failed relation belongs to concatDown inside the step
program. Direct concatDown has a smaller closure and has not been established
as a failing reproduction; changing the root would change the experiment.

Materialize either exact public source into a fresh directory:

```sh
# Original control: Heap8bb13182 and Sequencec352ecaf.
python3 tests/heap/export-sequence-repro.py \
  e87cd22372037374f7ce28b6ab0ea3e619129610 /absolute/new/sequence-control

# Structural factoring: fused Heapee41c3fb and Sequencece9ae14f.
python3 tests/heap/export-sequence-repro.py \
  ad4dc72155e0e32a1068f8bfb25afce3278def4c /absolute/new/sequence-factored
```

The generator verifies the immutable Git revision, extracts source bytes
without modification, checks that all module imports are supplied, and records
every source/configuration SHA256 in fixture-manifest.json. Its diagnostic
configuration is byte-exact from the isolated source-origin investigation.
Only a fresh compiler-specific diagnostic lock is generated; the product lock
and compiler are untouched.

From the selected fixture, use the existing official compiler:

```sh
/absolute/existing/lexlean --version
/absolute/existing/lexlean --project lexlean.toml lock --diagnostic-format json > lock-result.json
timeout -k 10s 360s /absolute/existing/lexlean --project lexlean.toml verify src/Uorc/Core/RuntimeSequence.lex.tex --diagnostic-format json > verify-result.json 2> verify.stderr
```

Record the actual compiler binary hash/version, generated lock, process exit
and both verification streams. The 360-second outer bound does not change the
SDK's 300-second child limit or 4 MiB certificate-source limit. A timeout is not
a kernel rejection or acceptance. No new compiler build or download is needed.

This ordinary verify selection includes the imported production roots. Even
an export request naming only step does not erase that preceding verification
closure. The confirmed diagnostic71defb4 control maps __rel_16 to concatDown;
new program/layout identities and relation mappings must be recorded rather
than assumed. The new source's general equality proofs establish unchanged
runtime meaning, including logical work, but do not imply certificate-A
acceptance. The existing #393 compiler can distinguish its actual ordinary
verification result without conflating the earlier certificate-B repair.

The original nine-suite/577-policy native receipt is separate. Neither this
fixture nor the narrower local dependency-prefix experiment establishes locked
product SDK acceptance, physical costs or complete release readiness.
