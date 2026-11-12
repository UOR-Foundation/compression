<!-- uorc:non-authoritative -->
# Streaming integrity evidence (issue #43)

The authoritative definitions are `src/Uorc/Integrity/Sha256.lex.tex`,
`BoundedChain.lex.tex` and `Contract.lex.tex`. This document records their
implementation and evidence; it supplies no independent archive-format rule.

`Sha256` implements FIPS 180-4's byte-aligned SHA-256 domain with eight UInt32
chaining words, a 16-word rolling schedule, 64 compression rounds, big-endian
word output, padding and a checked 64-bit bit-length domain. It stores at most
63 uncompressed bytes. Updates retain a caller-supplied chunk and offset;
`updateStep` consumes one byte or advances one compression round/feed-forward.
`finalize` starts padding, `finalizeStep` advances bounded work, and `digest`
returns the exact 32 bytes when complete. Repeated finalization preserves the
result. The source models invalid states, offsets, late updates, pending work,
unavailable digests, exhausted convenience-driver fuel and domain overflow.
Malformed schedule validation examines at most 17 list cells, so a forged
oversized schedule cannot turn an event into an unbounded traversal.

The byte-aligned SHA-256 domain ends at 2^61-1 bytes because the standard bit
length is strictly below 2^64. This does not define an archive-size limit.
`BoundedChain` supplies the separate `UORC-SHA256-CHAIN-v1` identity defined in
`Contract.contract.compositionFrame`. Its nested SHA messages are at most 118
bytes; it carries no whole-stream counter. Its explicit data/terminal domains
and final partial-block length are tested with seven independent hashlib/OpenSSL
fixtures, including empty, aligned and partial-block inputs. This construction
has finite functional evidence, not NIST certification or a security proof.
Hash agreement never replaces the product's exact reconstruction comparison.

## Independent data

The unmodified NIST CAVP files under `standards/nist-sha256` contain all 65
SHA256ShortMsg and 64 SHA256LongMsg byte-oriented known-answer cases. The
long-message lengths cover all 64 residues modulo the block length. The source
contract records the exact official URL, acquisition mirror commit, archive and
individual file digests, upstream Readme digest, NIST terms, and validation-only
role. The generated authority register preserves the two cited file identities. Imported facts keep their `some-true` classification; the local implementation and fixture evidence retain their separate classification.

The source-authority gate recognizes imported prose only at a source-bound path
and exact digest. Its negative test changes the upstream notice, observes the
digest rejection, restores passing bytes and rejects unregistered prose.

Acquisition used the existing public repository mirror recorded in that contract.
No direct NIST ZIP acquisition is claimed. The complete ZIP matched SHA-256
`929ef80b7b3418aca026643f6f248815913b60e01741a44bba9e118067f4c9b8`.
Only the scoped SHA-256 short/long files and upstream Readme are imported. No
bit-oriented or Monte Carlo certification is implied. Per-directory Git
attributes preserve the exact upstream bytes across checkout conventions.

The importer checks the locked file digests, byte sizes, complete ordered length
inventories, field order, hex widths and the zero-length placeholder. Tests reject
malformed, changed, dropped, duplicated and reordered records, including when the
structural parser is exercised independently of the byte-digest check.

## Executed procedure

`just integrity` runs the unfiltered importer/harness tests and
`scripts/integrity-validation.mjs`. It is required by the existing complete
`just vv`, which retains every earlier prerequisite. The harness verifies the
source, plants both source-owned arithmetic/length defects, checks the exact
owning generated theorem location, verifies restored controls, then builds and
executes the generated Lean model against every locked vector. It has no
handwritten hash implementation or production fallback. The generated client
also checks 2,919 partitions, SHA-256 abc, finalization/error behavior, pending
work, custom-chain fixtures and chain partition invariance.

Source dependency tracing found exactly `Quot.sound` and `propext` in fixed-width
runtime instances. Each declaration records its observed exact dependency set;
axiom audits remain exact. The closed kernel observations cover wrapping arithmetic, maximum bit-length
encoding, typed failure boundaries and every round-constant index (plus its first
invalid index). A general kernel theorem equates the completed finalization exit
with `digest` for every state and positive remaining fuel.
Full-hash reflexivity exhausted the diagnostic executor, so full hash equality
is supported by runtime fixtures and is not advertised as a universal theorem.

Diagnostic logs, original compiler identities, source hashes, attestations,
mutation receipts and runtime output are retained outside the repository in the
local integrity evidence directory. The end-to-end harness also writes a receipt
under `.prism/uorc/integrity.*`. The release locks have not been replaced.

