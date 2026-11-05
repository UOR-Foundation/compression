# Full-control checkpoint decomposition: unqualified source proposal

This change has not been run through the source compiler, Lean, a VM control,
`just model-write`, or the aggregate `just vv` boundary. It is not accepted
implementation or release evidence. The existing Search qualification owner
must independently decide admission of this exact source under the unchanged
60-second / 2,048 MiB external diagnostic guard and normal compiler settings.
No compiler/bootstrap, dependency change, or upstream LexLean edit was made
while preparing this proposal.

## Exact starting point and retained obligation

The parent is public Compression commit
`11f84f35760135909ffaebb32a1ae990befd3b76`. The original control source SHA256 is
`5deb09e497f6aa6d57eb5e8109f74917b39475caabc56ce44b9fff6e995dc44f`.
Its declaration 12 is `twoImages_all_spans_released_after_ack`, with no
parameters, statement `twoImagesCompletes() = true`, and proof `reflexivity`.
The statement and domain are unchanged. Every other original declaration,
including all executable bodies, original assertions, cancellation controls,
and the complete State/Arena waiting comparison, is structurally unchanged.
The original `run(500, startTrial(fixture))` remains the executed control.

The existing full generated module SHA256 is
`4f95f86f5bf93f99864dd0c7a96e095974ee65da95853fc52146461c719c6aeb`.
The retained complete-module run reached the marker immediately before the
original theorem, then was terminated externally with return code -15 at
57.141630695 seconds and sampled descendant RSS 2,099,780 KiB. That is a
resource barrier, not proof rejection or acceptance. Earlier decide and
record-match experiments are not adopted here.

## Why this decomposition is different

The original expression is:

- `twoImagesCompletes` matches the five-allocation `twoImages` fixture.
- It computes `auditComplete(run 500 (startTrial fixture))`.
- `run` reads the previous Trial's done/failed fields and passes that full
  Trial through `advance` while neither flag is set.
- Each `advance` performs tick/acknowledge, carries every state and arena
  field, and conjoins a fresh HeapAudit over fixture plus navigation roots
  into the previous `waitStable` value.
- The terminal audit checks done, not failed, accumulated waitStable, exact
  owned-byte restoration, and the complete original fixture-root audit.

A single reflexivity proof can repeatedly normalize the preceding execution
through these record projections and retained conditions. Merely naming the
same nested calls would retain that work. This proposal instead introduces:

1. Five explicit Allocation-result values and five bounded allocation
   equality obligations.
2. A complete explicit Fixture and a composed fixture equality obligation.
3. 102 complete explicit Trial snapshots, numbered 0 through 101, with all
   seven Trial fields and their complete nested State/Arena values.
4. An initial-state equality and 101 bounded one-advance equality
   obligations. Each is a reflexivity proof against the next explicit value.
5. A terminal audit equality and a new proof of the original theorem using
   named equality simplification plus the unchanged `run` equation.

Twenty complete Arena values are shared as closed immutable literals. Their
bodies contain no function calls. Other value definitions refer only to
prior closed Arena literals, never to run, advance, allocation, or an earlier
execution. No field is dropped. The helpers are marked non-executable.
Three direct imports expose constructors already used transitively by the
unchanged implementation.

If formally admitted, each local reflexivity obligation normalizes a single
bounded operation on a concrete input, rather than normalizing the full
previous execution. The final proof then uses checked equalities to replace
those operations. Every allocation, intermediate audit, release, and final
audit still requires a kernel-checked obligation. No finite evaluator result
can discharge any of them. Performance improvement and even admission of
this exact proof remain unverified until the complete generated module runs.

## Source mechanism and construction provenance

This uses existing source proof syntax only. At the same parent commit,
`Uorc.Core.RuntimeOwnershipProofs.retain_release_restores_image_arena`
composes explicit transition equalities under nested matches using
`simplify{definitions:[named_lemma,...]}`. `RuntimeIndexProofs` also composes
named equalities with simplify. No new rewrite or raw Lean facility is assumed.

The literal candidates were synthesized by an untrusted source-AST constructor
routine using an existing official-reader cache. Every native-Core source
SHA256 was checked against the exact parent source before use. The routine
is not a proof producer or an acceptance authority. Incorrect candidate data
must cause a local equality obligation to fail in the official kernel.
The checkpoint definitions are ordinary data, and their claimed identities
have proof bodies requiring verification; they are not postulates.

The companion JSON binds old/new source, semantic-data diagnostic hashes,
statement/proof hashes, all 26 original declarations, every closed literal's
dependency list, and the exact original executable call graph. These
canonical JSON hashes are not official LexLean semantic IDs. Official
source/semantic/build IDs must be recorded by the existing qualification
owner after it processes this candidate.

## Known uncertainty and required qualification

The proof shapes have source precedents, but this exact source was not checked
for schema/type/link admission. In particular, success of simplify rewriting
through the full fixture and 101 run steps, and resource usage of all new
reflexivity obligations, are pending. A remaining failure must be reported as
its exact observed phase; it must not be turned into a pass or bypassed.

The qualifier must retain every original theorem and all new obligations in
the complete module. It must use the ordinary source/build/kernel/axiom and
control workflow, then the complete current aggregate and pinned-SDK
acceptance gates. No root skipping, theorem filtering, disabled assertion,
opaque trusted computation, raised limit, or altered acceptance boundary is
permitted. Later original controls have not been established merely by
getting past the previously blocked theorem.

## False controls and exact restoration

Before accepting the decomposition, retain the exact positive source SHA256
from the JSON and use isolated mutant copies, never the canonical branch:

1. Change only `twoImagesTrialCheckpoint1.state.consumed` by one. Keep all
   other snapshot fields and all proof bodies unchanged. The bounded
   `twoImages_advance_checkpoint_0` obligation must be rejected by the real
   kernel, not skipped or evaluated by a VM in its place.
2. Independently change only the original final theorem's right-hand side
   from true to false, retaining the positive helpers and proof. The exact
   theorem must be rejected; a resource termination is not rejection proof.
3. Plant a representative actual executable ownership defect, such as
   omitting a required ticket-root release in the original implementation,
   while keeping these literal expectations unchanged. Require the ordinary
   relevant ownership/control gate to reject the mutation and identify the
   failing local identity or original assertion.
4. Restore every changed file byte-for-byte from the positive candidate,
   verify its source and executable-body hashes, and rerun the full positive
   module and required aggregate controls. Record actual command, exit,
   diagnostic, resource use, exact artifact hashes, and restore result.

If any mutation is accepted, or restoration fails, the candidate is not
qualified. If stock resource limits still block it, preserve that terminal
receipt and seek another source-level proof decomposition without changing
the product theorem or resource policy.
