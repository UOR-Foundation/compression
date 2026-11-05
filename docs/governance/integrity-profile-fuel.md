<!-- uorc:non-authoritative -->
# Canonical profile driver totality

The authority is `Integrity.ProfileFuelLaws`. `profile_hash_succeeds` proves
that actual `Profile.hash kind purpose input` succeeds for either kind, every
purpose and every finite source input, returning 32 bytes. The proof establishes
the actual canonical update budget `600 * input.size + 1200` and finalization
budget `1200`. It does not substitute the smaller convenience-wrapper budgets.

The phase invariants come from actual initialization and transitions. They
include SHA validity and buffer/work bounds, exact prefix and footer accounting,
full-frame rollover conservation and the settled update boundary. They do not
assume that the public validity Boolean suffices for arbitrary forged states.
The actual STREAM prefixes have 27 and 59 bytes, with a nine-byte footer.
SHA's message-length bound applies independently to each frame. There is no
whole-input length bound or accumulated total counter in the construction.

All 222 declarations passed exact axiom audits, scoped diagnostic SDK
verification, independent generated-kernel checks and leanchecker. Replacing
the actual canonical finalization fuel with zero rejects the unchanged theorem;
restoration reproduces the same attestation. Exact receipts are under
`validation/integrity/profile-fuel`. This proves source driver progress, not
physical memory bounds, universal SHA correctness against FIPS or full original
pinned-SDK acceptance.
