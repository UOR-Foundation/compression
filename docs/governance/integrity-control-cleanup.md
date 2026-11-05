<!-- uorc:non-authoritative -->
# Actual childless control cleanup

This note records the historical `8188c3d` qualification with Core `8bb13182`. Its receipts retain that exact scope. [Current-source requalification](integrity-current-source-requalification.md) records the separate replay against restored Core and current Wire.

The authority is `Integrity.HashControlCleanupLaws`. Its observed-read trie
frame uses genuine prior lookup observations; it does not assume that different
numeric keys necessarily take different paths. Actual `Heap.release` preserves
the index and schedules the singleton release. Successful `releaseStep` on the
actual one-element root-step queue, with a control-reservation lookup and the
separate image lookup, preserves that image entry and completes the empty queue.
Both deletion and incoming-reference decrement branches are covered.

The local predicates are structural facts about actual nodes and the actual
queue. Root presence or equal lengths do not establish them. The composed
`OwnedControlLifetime` proof establishes them from actual allocation, span
validation, retain, reservation, commit and hashing transitions. The helper
increment also proves the actual local frames those transitions need.

All 34 declarations passed scoped source build, generated kernel checks,
exact axiom audit and leanchecker. A real delete-branch defect clearing the
whole index makes protected-image reads fail and invalidates the unchanged
frame proof; exact restoration passes. A separate wrong-key diagnostic was
terminated with exit -9 and no kernel diagnostic. It is recorded as resource
termination, not semantic rejection evidence; no verifier limit was increased.
Exact receipts are under `validation/integrity/control-cleanup`. General Core
ownership, production export and full pinned-SDK acceptance are separate.
