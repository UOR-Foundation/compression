<!-- uorc:non-authoritative -->
# Scoped integrity proof evidence

The authorities are `Integrity.ByteEquality`, `CollisionLaws`, `StateLaws`,
`ShaTransitionLaws`, `OwnedLifecycleLaws`, `IncrementalLaws`, `ShaLifecycleLaws`,
`PartitionLaws`, `FuelLaws` and `PartitionTotalLaws`.
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

`ShaTransitionLaws` proves checked-schedule preservation by arbitrary word
replacement, validity and count preservation after successful compression work,
and count preservation after every successful finalization step. Its byte
decoder and low-bit proofs establish the exact count remainder modulo 64 for
every UInt64 value. A separate four-module diagnostic verification and replay
passed with attestation
`f577ab5bceb8c469594c93a30e98cd56ccab615ccd00467f67e0ae94fb510e9f`.
Dropping a replacement tail, advancing a round twice, resetting a finished
message count, and corrupting the remainder at 64 each fail their new proof
owner. Restored source reproduces the same attestation.

`OwnedLifecycleLaws` proves exact ordered root-list preservation, actual
one-byte read/admission correspondence, retained input during update, and
canonical cancellation/publication ownership shapes. Its read bridge requires
the explicit cursor/state profile agreement and actual checked read success.
At a successful release request, the span ticket transfers to the Heap release
cursor and the outer input field clears. `spanComplete` waits for actual Heap
release completion. These structural laws rely on the independently maintained
Heap ownership invariant for genuine root ownership; they do not infer that
invariant from list membership or from arbitrary forged cursor records.

The owned lifecycle source compiles from fresh official generated modules,
passes all 28 exact declaration audits and separate kernel replay, and rejects
an actual source mutation reporting two copied bytes for its one-byte admission.
Restoration passes compilation and replay. Its complete scoped SDK verification
still fails inherited certificate A at `RuntimeSequence.step.__rel_16` with
`LLV7013` and kernel deep recursion. Those results are recorded separately; no
owned lifecycle SDK attestation is claimed.

`IncrementalLaws` decomposes every successful actual update step into the
actual compression-work step, absorption of the byte at the current input
index, or exact input exhaustion. Its finite primitive trace theorem follows
the actual `runUpdate` and `update` functions. With in-range starting offsets,
equal remaining lengths and indexed suffix bytes imply equal actual `runUpdate` results at every fuel,
including errors. This establishes continuation equivalence under reindexing;
it does not yet establish arbitrary chunk-partition equivalence to one-shot
`Sha256.hash` or the engine-to-authenticated-frame-trace bridge.

Its two-module scoped SDK verification, exact audit and separate kernel replay
pass with attestation
`d1325f324cbe0016507ba02daa67365ec6f2e46d209a9666c5fcfb026160fbf5`.
Changing the actual consuming offset increment from one to two rejects the new
source bridge; restoration reproduces the exact original command result.

`ShaLifecycleLaws` proves successful byte absorption preserves validity and adds
exactly one to the natural message count. Successful finalization and update
steps preserve their specified state/count invariants, actual digest output is
32 bytes, and finished finalization is stable. Its 76 native declarations pass
complete scoped verification and independent replay with attestation
`bb031e677b47abef3185e9a0e7a6bd2dbc4c71d3084d1726316898d51b876036`.
Five actual implementation defects reject; the receipts identify the first
owner, including earlier prerequisite proofs and the precise kernel recursion
failure for the absorption defect. Restored verification is identical.

`PartitionLaws` connects actual chunk updates to concatenated execution and
one-shot finalization when the named operations succeed. `FuelLaws` proves the
actual source budgets suffice: update uses `3*n + 132`, and finalization uses
`300`. Every message in SHA-256's standard byte-count domain therefore succeeds
in the actual source hash. `PartitionTotalLaws` supplies exact message-count
postconditions and unconditional two-chunk agreement from the standard initial
state for combined length at most `2^61-1` bytes, including identical actual
one-shot/partitioned digest bytes and width 32. This is the standard SHA domain,
not a total archive input-size limit; STREAM-MAX is the separately named
composition for arbitrary finite total input.

The partition, fuel and total-partition scopes verify and independently replay
with attestations `d1afca219e5dcc21d56aca894be255283799bbd893d89dadf8540a9d52540d63`,
`4179b4059d55fb903e32cae3b032ff7988e2cb838a0e2353e39c17ca16874f9e`
and `01f6b2c235fac59c3027eb44fdd7210b98f536f53466c4bb8b60aee85aefb38a`.
Actual input loss, insufficient update/finalization budgets and count-reset
defects reject; all restored results reproduce their original attestations.
These are source-Nat results. Target storage, overflow and physical-resource
refinement remain separate claims.

The combined six-module diagnostic verification and kernel replay passed with
saved official compiler `71defb4c663aaec5b9a3aab04cbc6a03e7106227`, binary SHA-256
`38f8502e5c108a362265663253283a0f60f14538951e592b2d5dd4db9e67be0a`,
under language 1.2 and Lean 4.32.1. Its attestation is
`73f07f639a30fa463b3ae372d7ac01679fac80a7940456c104f18d7dce7122b5`.
The project proof declarations use only their exact traced `Quot.sound` and
`propext` foundations; no project proof uses a custom axiom, `Classical.choice`,
`sorryAx`, `native_decide` or proof skip. Imported SDK declarations retain their
own exact audited policies.

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
SHA implementation-to-standard functional correctness, general engine/Heap
ownership preservation and integrated BinaryProgram/full archive acceptance are
separate remaining obligations. Hash equality is not reconstruction equality;
full byte reconstruction comparison remains necessary.
