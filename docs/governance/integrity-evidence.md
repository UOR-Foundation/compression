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

## Export refinement and native execution

The original producer failed LLB6005 at the call of `Sha256.digest` from the
recursive finalization driver. This matches upstream [LexLean #279](https://github.com/UOR-Foundation/LexLean/issues/279), fixed by [#300](https://github.com/UOR-Foundation/LexLean/pull/300). Its completed branch now directly emits the
validated digest snapshot. `completed_finalize_refines_digest` proves the source
refinement; there is no empty-copy workaround or changed digest behavior.

The next producer failure was the depth-132 round-constant list subtree in the
package manifest, matching upstream [LexLean #378](https://github.com/UOR-Foundation/LexLean/issues/378). A balanced scalar selector now supplies the execution closure.
The original literal list remains the mathematical reference for all 64 values
and the first invalid index. This reduces the lookup walk and removes that deep
execution subtree without changing compiler limits or validation policy.

The separate official language-1.2 diagnostic projection now builds its certified
`rust-std` production bundle and passes `production verify-artifact`. An
independent validation client compiles the exact generated library bytes and
executes its shared SHA-256 implementation and exported chain root: all 129 CAVP
cases, 2,919 partitions, seven chain fixtures, repeated finalization, typed errors
and maximum-count rejection pass. `validation/integrity/evidence.json` binds the
source, client, generated library, package and producer identities; its function
numbers apply only to that exact artifact. The client contains no hash algorithm.

The production projection adds executable/effect declarations only in a separate
migrated copy. The repository language selection, release locks and compiler
policy remain unchanged. These observations are diagnostic production evidence;
they do not qualify a different immutable SDK or an integrated BinaryProgram.

## Segment framing review

`src/Uorc/Integrity/SegmentReview.lex.tex` owns the explicitly proposed candidate
profiles, their exact formulas, domain-bound observations, reference vectors and
conditional collision-binding argument. They are not wired into the archive or
substituted for `UORC-SHA256-CHAIN-v1`.

The preferred candidate is `UORC-SHA256-STREAM-MAX-v1`: first and continuation
frames have distinct domain prefixes; full and terminal frames have explicit
role/BE64-length suffixes. The frame size is bounded by SHA-256's own standard
domain, with no accumulated whole-stream counter or retained segment payload.
It avoids a separate seed hash. For a 1 MiB input its analytical compression-block
count is 16,385, versus 32,770 for the current 64-byte chain. The source record also
retains the earlier seeded MAX and 1 MiB alternatives rather than reusing their
identities for a changed formula. Reference fixtures are not runtime acceptance
or a security proof, and no universal collision freedom is claimed.

## Acceptance boundary

The saved official compiler and Lean 4.32.1 provide source, kernel, generated-model
execution, diagnostic certified production export/replay and native execution
evidence. Complete acceptance still uses the immutable SDK in `prismpm.lock`.
Full pinned-SDK `just vv` and integrated BinaryProgram product acceptance remain
outstanding. Issue #43 remains open; this branch does not close it or claim the
complete compression product is release-qualified.
