<!-- uorc:non-authoritative -->
# Actual owned digest lifetime and publication

The authority is `Integrity.OwnedControlLifetime`. The final
`completed_view_canonical_publication` theorem combines actual `CompletedView`
provenance, the actual closing phase and a continuous actual successful
`FinishPublished` suffix. It derives the chosen canonical profile digest and
the bytes read from the published image. The caller supplies neither canonical
hash success nor an output-preservation flag.

Actual control allocation establishes the reservation entry and ticket metadata.
The invariant follows real prefix/update steps, validated ordinary input spans,
retain and input release, output reservation and commit, and final control
cleanup. The generic local Heap frames are proved separately. Retain success
alone is insufficient: a control reservation can itself be retained. The
consumer uses the actual validated ordinary-image observation to establish
the needed separation. The ghost control handle remains available after the
runtime cursor clears it while the singleton cleanup queue drains.

Actual commit storage and actual cleanup establish the output node frame;
actual profile driver totality supplies termination. All histories and lifetime
witnesses are propositions, not extra runtime buffers or archive fields.

The theorem covers the bytes admitted by the completed ordered view. Relating
that observed sequence to an initial archive layout still requires the real
reader/consumer shape and retained-source contract. It does not assert parser
provenance, full decoder byte reconstruction or source-layout correspondence
merely from root presence, equal lengths or hash equality.

All 73 declarations passed scoped source build, generated kernel checks,
exact axiom audits and leanchecker. An actual initialization defect allocating
a runtime reservation in place of the control reservation invalidates the
unchanged allocation/lifetime proof; exact restoration passes. The receipt under
`validation/integrity/owned-control-lifetime` binds all 30 source dependencies.
Original locks, verifier limits, Core implementation and NIST bytes are unchanged;
full pinned-SDK, production export and whole-product acceptance remain separate.
