<!-- uorc:non-authoritative -->

# Frozen current-Core enumeration diagnostic

The original seven-file producer checkpoint and its receipt remain unchanged in
`0c812074d6039aa2b6fb77b4a3dd77d07827f7c3`. The original receipt is
`owned-enumeration-diagnostic-evidence.json`. This additional receipt records a
new execution of those same producer sources against exact newer public Core
inputs. It does not relabel the original result.

## Exact source union

- Base sources: `0c812074d6039aa2b6fb77b4a3dd77d07827f7c3`.
- Core source checkpoint: `3609a3624f7bf0f0d7c83b181fd88317868bdad5`.
- Authentic parsed Core cache commit: `eb56990bf53cd0c483b40df19557bbe0bfa120be`.
- Cache SHA-256: `06c2988f850938019f5454439cf130a736aaa81c636c425dff29ce21015e2b04`.

The runner overlays only the 34 named Core source/cache pairs on the reviewed
base. Every source SHA-256 and canonical parsed-module SHA-256 is checked before
execution. The cache retains the official reader's byte-exact repeat-read and
notation-round-trip evidence. This run consumes that evidence; it does not run
the official reader again. All remaining source modules retain their base bytes.
The JSON receipt inventories every source in this union, its SHA-256 and source
commit, all 34 parsed Core identities, exact commands and actual outputs.

The sole evaluator projection is substitution of the old exact cache fingerprint
with the new exact cache fingerprint. Producer bodies, diagnostic assertions,
mutants and test inventory are unchanged. The original and projected evaluator
hashes are recorded separately. No Core production source is rewritten or
published by this diagnostic workflow.

A direct attempt to overlay the producer onto the *whole* Core checkout failed
before a producer event: that checkout's native-notation RuntimeResources is not
included in the parsed cache, and the legacy evaluator cannot find zeroUsage.
The explicit source union retains the base RuntimeResources. This composition
limit is visible in the receipt. It is not a producer failure, whole-Core test,
compiler result or native execution.

## Reproduction and actual controls

In a Git object repository containing all three exact commits:

```sh
python3 -B tests/enumeration/test_current_core_qualification.py
python3 -B tests/enumeration/qualify_current_core.py \
  --repository . \
  --output /tmp/owned-enumeration-current-core-evidence.json
```

The fresh union execution passes empty plus all 256 one-byte candidates and the
carry to 0000/0001/0002, all 60 saved successor cancellation prefixes, 120
zero/refusal/split-credit/repeated-cancel prefixes, failed control allocation
recovery, nine actual allocation-refusal/exact-retry prefixes and unresolved
checked length overflow. All four actual loaded-source-IR mutants fail at their
intended semantic assertion and pass after exact restoration.

Eleven harness controls reject omitted or duplicate diagnostic results, failed
commands or stderr, unregistered commands, omitted/duplicate/reordered mutants,
unrelated semantic assertion failures, missing/duplicate restoration results,
wrong frozen cache bytes and optimized Python. The harness accepts no test
filters and checks its complete duplicate-free inventory and unskipped results.
These are diagnostic self-quality controls, not a substitute release gate.

## Remaining proof and consumer boundary

The producer still requires actual-source shortlex successor/no-omission proofs,
reachable arena/builder/root-multiset simulation and official compiler/native/
kernel qualification. Local phase laws and a separate mathematical byte-word
model do not by themselves establish those stronger refinements.

The general-byte consumer must read the real candidate archive through independent
ArchiveParse, versioned integrity, ProgramAdmission, the actual machine and
ordered-output/full-byte comparison. The TemplateAdmission event producer is not
an archive-membership substitute. Candidate sequence ownership must remain pinned
through the complete trial and every outstanding span acknowledgment. Operational
failure leaves the predecessor unresolved. Custody or EOF acknowledgment alone
never establishes membership, exclusion, predecessor coverage or a minimum.

The discovery scheduler must remain fair and independent of exhaustive
certificate search. Ordinary best-known compression can produce useful output
without waiting for complete minimum certification.

All evidence here is finite generic source evaluation. Official compiler checks,
generated-native execution, kernel proofs, canonical membership, physical resource
measurements and the unchanged full pinned-SDK acceptance boundary remain separate.
