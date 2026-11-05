<!-- uorc:non-authoritative -->

These five files are immutable historical source snapshots from commit
`2f1e26af784043c7782f07068f3cd1ead8059b07`. Their exact bytes match every
`source_sha256` row in `../stream-evidence.json`. They are archived inputs to
that historical diagnostic native observation, not current production sources,
entrypoints, generated implementations, or a fallback.

`../source-equivalence-v1.json` pins the current source bytes separately and
lists each permitted addition of `executable: true`. The official SDK reader
compared the historical and current modules when producing that receipt.
`scripts/integrity-source-provenance.mjs` rechecks every historical hash and
current pin, then compares the complete semantic models after reversing only
the listed annotations and the semantic-module/1 to /2 migration. Bodies,
proofs, types, signatures, policies and all other fields must remain identical.

The old native result, artifact hashes and source hashes stay unchanged. This
bridge establishes computational-model equivalence only; it does not claim the
current source bytes were used for that native execution or replace fresh
native observations.
