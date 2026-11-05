<!-- uorc:non-authoritative -->
# Actual childless control cleanup

The authority is `Integrity.HashControlCleanupLaws`. Its observed-read trie
frame uses genuine prior lookup observations; it does not assume that different
numeric keys necessarily take different paths. Actual `Heap.release` preserves
the index and schedules the singleton release. Successful `releaseStep` on the
actual one-element root-step queue, with a control-reservation lookup and the
separate image lookup, preserves that image entry and completes the empty queue.
Both deletion and incoming-reference decrement branches are covered.

The local predicates are structural facts about actual nodes and the actual
queue. Root presence or equal lengths do not establish them. Integrity is also
proving `OwnedControlLifetime`: actual allocation, span validation, retain,
reservation, commit and hashing transitions must establish and preserve the
control entry and input-release queue before this cleanup theorem applies.

All eleven declarations passed scoped source build, generated kernel checks,
exact axiom audit and leanchecker. A real delete-branch defect clearing the
whole index makes protected-image reads fail and invalidates the unchanged
frame proof; exact restoration passes. A separate wrong-key diagnostic was
terminated with exit -9 and no kernel diagnostic. It is recorded as resource
termination, not semantic rejection evidence; no verifier limit was increased.
Exact receipts are under `validation/integrity/control-cleanup`. General Core
ownership, production export and full pinned-SDK acceptance are separate.
