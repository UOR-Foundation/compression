<!-- uorc:non-authoritative -->
# Scoped integrity proof evidence

The authorities are `Integrity.ByteEquality`, `CollisionLaws` and `StateLaws`.
They are proof modules; ordinary archive hashing, compression and decompression
add no proof metadata or dependency on proof completion.

`checkCandidate_sound` concerns the actual source checker. A returned verified
witness has distinct canonical messages and both actual `Sha256.hash` calls
succeed with its digest. `frame_codec` proves the actual prefix/payload/footer
encoding distinguishes the fields needed for backward reasoning. The byte
encoder has one-byte output for every UInt8 value, the encoded length has eight
bytes, and the role marker at offset 26 distinguishes first and continuation
frames. The equality bridge proves the existing byte comparison lawful; it does
not replace its computation.

`extract_collision_for_canonical_frames` constructs a candidate and accepted
witness for arbitrary finite supplied traces satisfying shape validity,
successful recomputed source-SHA values, equal root digests and distinct
concatenated payload bytes. Both witness messages are distinct and within the
standard SHA byte-domain bound. Its framing-codec property is proved rather
than assumed, and no SHA injectivity assumption occurs. Here trace authenticity
means agreement with recomputed hash results; it is not origin authentication.
Trace retention is nonproduction proof evidence, not the production stream state.

`StateLaws` establishes general preservation of message count, buffer and phase
for compression-work constructor shapes, rejection without work, message-count
preservation by `appendRaw` and `finalize`, and finished stream-state stability.
Preservation predicates are vacuous on typed errors. These are kernel statements
over arbitrary typed fields, not finite vectors or a full valid-state theorem.

The combined six-module diagnostic verification and kernel replay passed with
saved official compiler `71defb4c663aaec5b9a3aab04cbc6a03e7106227`, binary SHA-256
`38f8502e5c108a362265663253283a0f60f14538951e592b2d5dd4db9e67be0a`,
under language 1.2 and Lean 4.32.1. Its attestation is
`73f07f639a30fa463b3ae372d7ac01679fac80a7940456c104f18d7dce7122b5`.
Only the exact traced `Quot.sound` and `propext` foundations occur; no custom
axiom, `Classical.choice`, `sorryAx`, `native_decide` or proof skip is used.

Removing only the checker's digest guard fails the new checker proof while the
old acceptance-guard truth table remains unchanged. Corrupting a returned digest
also fails; a labeled diagnostic isolation attributes the latter specifically to
`checkCandidate_sound`. Incrementing compression-work message counts fails the
new state-law source bridge. Restored source reproduces the original scoped
attestations. These controls supplement the required complete verification gate.

Actual command results, diagnostic project/lock identity, full normalized combined
attestation, individual replay process records, source/helper digests and exact
signatures are under `validation/integrity/proofs`. Native kernel terms in the
`.lex.tex` files are the authority. Temporary authoring helpers are outside the
repository; generated Lean is not committed or a production fallback.

This saved component compiler result does not change or qualify the original
product pinned SDK. The actual decoder/owned engine-to-authenticated-trace bridge,
SHA implementation-to-standard functional correctness, general valid-state/Heap
ownership preservation and integrated BinaryProgram/full archive acceptance are
separate remaining obligations. Hash equality is not reconstruction equality;
full byte reconstruction comparison remains necessary.
