# Heap release-step factoring

Heap.releaseStep keeps its original pending-list, count, lookup and reference
validity decisions in order. Three source helpers share its last-reference,
deletion and surviving-reference branches. No guard, allocation, removal,
ownership operation, pending ticket, error, delta or work charge changes.
All other existing Heap declarations are identical in official parsed form.

The original Heap source is SHA256
`ee41c3fb11c1f60fc767448760e8020d934c253fc31eec2146413ec1a98b0e1b`.
The factored source is
`b3423752932ed8e385a35d947a86001ed4b329ef8009a6bd70ec5258f3a63918`.
RuntimeHeapReleaseReference preserves the complete old releaseStep body as a
non-executable definition. For every Arena, its source-owned reflexivity theorem
proves equality of the entire actual Result to that body, including all failure
cases and every returned state/accounting field. No wellformedness premise or
finite test domain is used. This is an implementation-equivalence bridge;
global ownership preservation remains a separate obligation.

Official formatting, source check/build and the affected generated Lean modules
pass. The three helpers, releaseStep, old-body reference and theorem have exact
axiom sets [Quot.sound, propext]. A deletion work1-to2 defect leaves the runtime
source kernel-valid but fails the general equality; exact source restoration
passes again. The [receipt](runtime-heap-release-factoring-evidence.json) records
the isolated diagnostic qualification and its unchanged language1.2 project.
Product language1.1 and locks are unchanged.

The main parsed body shrinks from428 dictionary objects/depth25 to115/depth12;
the three helpers have depths14,9 and13. These structural source measurements
are not certificate sizes, generated-code costs or performance measurements.
No certificate or export was attempted for this factoring. Earlier native
receipts retain their original ee41 Heap identity and are not relabeled as new
native executions.

Integrity's newer full19-module releaseStep certificate-A report, its narrower
two-source Heap.allocate certificate-B failure, the historical Sequence A
probe, and Product97's Builder B size refusal remain separate. This factoring
does not infer an emitted relation mapping or claim acceptance of any of them.
