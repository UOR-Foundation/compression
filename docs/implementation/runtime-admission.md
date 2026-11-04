# Owned program admission interfaces

The source-owned components in `src/Uorc/Core/Runtime*.lex.tex` separate syntax, typed admission, immutable metadata construction and the eventual admitted-session boundary. The module sources, rather than this integration guide, define their behavior.

## Trust and ownership

`RuntimeProgram.Ready` is a fixed carrier of an arena, program/basis tickets, caller root partitions and explicitly byte-denominated usage. A record is not an unforgeable proof token. The untrusted-input product path must obtain it by completing actual admission, not accept a caller-supplied record, Boolean validation flag or saved cache as evidence. A trusted machine API needs the concrete admitted-session contract. Reader failures remain typed for malformed metadata and references.

Program and basis roots are only two partitions. Input, archive, wire, search, checkpoint, certificate and verification owners stay separately inventoried. Repeated handle IDs are repeated tickets, never a set deduplication rule. A borrowed literal span stays backed by its charged wire/search owner until full acknowledgment. Admission copies its ordered bytes through the owned canonical builder before acknowledging it; it does not retain a borrowed image as an uncharged literal owner. A completed production admission must not retain a parallel mathematical graph, Rope or Store outside this inventory.

## Bounded source components

- `RuntimeAdmissionOrder.start/advance` checks event order and counts only. It is not typed validation or a Ready producer. Scan's observer capture lane must finish before its updater reference scalar. Literal spans consume the declared length exactly; impossible span totals are rejected without wrapping arithmetic. Completing this preflight is insufficient to acknowledge an event before its typed checks and owned mutations finish.
- `RuntimeAdmissionChecks` resolves typed local distances and prior-block distances; checks immediate domains; checks Apply/Tabulate/Fold/Scan parameter prefixes, results and capture parameter types; and checks return and entry signatures. Any of the five value types can be a loop state. Zero iteration counts do not skip static checks.
- `RuntimeInstructionCheck.start/scalar/finish/captureContract` stores four fixed scalar positions, result/state types and at most two borrowed capture contracts. Each scalar returns an optional absolute local slot for a separately performed last-use write. Scan's sequence result does not overwrite its arbitrary state type. Choice branches must have the same type.
- `RuntimeLifetime.start/step` reads one indexed last-use cell per planner event. Unused parameters enter the initial release bucket. A produced value with no later use enters its creation instruction's post-instruction bucket. Recorded uses are checked against creation and body bounds. The return slot remains held until the frame root is dropped. Timestamps are zero-based instruction indices, not `n + 1` sentinels.
- `RuntimeRecordBuilder.start/step/rootInventory` replaces one owned fixed-depth metadata record. It keeps the old ticket until the replacement path is complete, then performs explicit resumable arena release. Failed writes retain the actual partial arena and temporary root inventory. No detached release cursor can become stale.

These components are composed by RuntimeAdmission into streamed admission. Source composition alone does not establish that the complete production admission, execution and archive pipeline has been qualified.

## Metadata ABI

`RuntimeMetadata.kindCode` defines internal kinds for block index, block record, instruction index, instruction record, lifetime index and release bucket. These identifiers are not archive wire opcodes. `RuntimeOpcode` owns 39 normalized symbolic operations; the two mathematical literal constructors normalize to the same charged literal operation.

A block record has fields 0 parameter count, 1 result type, 2 instruction count, 3 return distance, 4 parameter type table, 5 instruction index, 6 post-instruction lifetime index and 7 initial release bucket.

An instruction record has field 0 normalized opcode; fields 1–4 its required scalar operands; fields 5–6 declared capture counts; fields 8–9 capture tables; and field 10 a literal sequence handle when applicable. Unused fields are absent. Empty required sequence/table links are explicitly null. Readers never turn an erased in-range runtime slot into a default natural value.

A release bucket has field 0 count and field 1 its indexed absolute-slot table. A missing post-instruction bucket means no releases. `RuntimeProgram` readers validate record kind, tag, field type and range instead of trusting cache claims.

## Evidence distinction

The modules contain executable scenario owners and general theorem candidates. Scratch interpretation of actual source terms is diagnostic, not official compiler, kernel, generated-runtime or locked-SDK evidence. Planted defects have been checked against the same source owners for operand typing, Choice arm equality, missing versus timestamp-zero uses, premature Scan captures and omitted old-record reclamation. `RuntimeRecordTests` independently checks the actual heap inventory at each intermediate cursor state. Official qualification and complete release acceptance remain separate requirements.

`RuntimeFixtures` is test-only source construction over the resumable table API. Its fixed diagnostic driver does not create a product-level input-size or event-budget limit, and must not enter the production decode closure.

## Diagnostic machine prepayment planning

The older mathematical-value StepMachine exposes `planningWork(context,machine)`
and `plannedWorkBound(context,machine)` as read-only probes. They do not execute,
copy an output, allocate a semantic value, or validate an entire graph. Planning
costs 1 search unit, or 129 when a single fixed-depth return-slot lookup is needed.

The bound is **search-effort escrow**, not decoder admissibility work. The
ExecutionCredit owner must retain unused escrow and report actual decoder usage
separately. A loose bound delays execution until sufficient search credit is
available; it must not declare a program semantically invalid or replace the
configured canonical decoder budget.

The finite charged call graph gives these inclusive upper bounds for one step:
- pending operation: 322, plus the current ByteBuilder event work for a builder;
- pending release or comparison: 129;
- instruction dispatch: 641;
- return to a continuation: 451;
- final return: 131 plus the selected immutable sequence's cached length.

The derivation counts every reachable Resources.charge in the step call graph,
maximizes mutually exclusive branches, and sums charges that can occur in one
branch. The largest instruction-read charge is 512; dispatch adds 1 and the
possible atomic slot write adds 128. Pending builder work additionally permits
321 subsequent frame/control work. Return charges are dispatch 1, indexed read
128, root drop 1, and output-boundary 1 or continuation work at most 321.
Resource rejection can commit only a prefix of these charges. These are logical
source costs, not claims about CPU instructions, allocator traffic, or RSS.

`PlanningTests.run` invokes actual step traces and rejects a planted zero-bound
mutation. A separate diagnostic AST run covered 24 source graphs / 446 steps,
including 22 successful and 2 failed executions; every observed actual work
prefix was within the proposed bound. This is finite evidence, not a checked
general bound theorem or SDK qualification. The general bound obligation remains
open until ordinarily verified against the actual source definitions.

### Span ticket transfer

A nonempty `RuntimeSequence` terminal `Outcome.span` transfers one owned image
root ticket to its returned `Span.image`. The leaf navigation first retains the
image; the terminal phase clears its cursor without releasing that ticket. The
producer must represent the returned ticket in its wire/search owner aggregate
until full admission acknowledgment, then explicitly release or transfer it.
The all-null terminal cursor has no additional cleanup obligation. An empty
span retains no image. A terminal cursor's empty inventory must never be used to
infer that its separately returned result is borrowed.

## Owned admission, cancellation and producer transfer

RuntimeAdmission now implements the complete event handlers, typed scalar and
capture checks, persistent indexed metadata, literal image spans, incremental
last-use schedules, entry checks and final Ready ownership move. It retains no
Graph AST or mathematical Rope/Store alongside the arena. `offer` merely queues
an event; only a later awaiting outcome acknowledges complete consumption.
RuntimeAdmissionCredit stores an uncomputed intent and pays its whole declared
source task cost before calling start/offer/step. Partial credits preserve the
current arena, not a materialized future allocation. Source scenario owners
check the1023/1024 initialization boundary, resource rejection with paid attempt,
and rejection of a planted early-initialization mutation.

RuntimeAdmissionCancel detaches the bounded session ticket inventory after any
existing arena release drains, clears its former owner slots, then releases one
bounded ownership event per tick. It preserves caller/basis owners and achieved
work on every failure. `tickWork` must be prepaid by a fair-search caller. An
exhausted cleanup work budget returns a retained typed state; cleanup is neither
free nor permission to discard roots. The controller must reserve resources for
cleanup or resume it under an explicitly extended run policy.

RuntimeAdmissionBoundary.take is legal only at a fully acknowledged boundary.
It allocates a unique112-owned-byte leaf lease. AwaitingState has no Arena,
current Usage or current CallerRoots. A consuming Taken move carries the one
arena, achieved usage and current caller partitions separately. Only the chosen
wire/search producer aggregate may change. Reattachment checks counters,
watermark, all fixed ticket multiplicities and the live unique lease, then
removes that lease in a bounded return step. Reusing the lease against the
achieved arena is rejected. Numeric checks do not authenticate arbitrary forged
or retained historical arena snapshots; the paired carrier lineage is an
internal trusted consuming-root contract, never a public validation proof.

RuntimeAdmissionCreditBoundary performs the same move for the prepaid session.
Its Detached carrier contains AwaitingState plus funding counters and NO Arena.
Reattachment takes that Detached state and only the achieved arena/usage/caller,
so it never retains an old Taken arena alongside the producer's new arena.
Boundary work is charged operational work and must be prepaid separately by the
producer/controller, not silently counted as admission's own funding credits.

RuntimeOwnerRoots centrally registers searchOwner/wireOwner metadata kinds6/7.
`beginControl` reserves4096 bytes for a fixed scalar/tag producer state, not a
Graph, byte buffer or variable collection. `pinBorrowed` moves the control ticket
and pins a borrowed immutable payload by an edge. `joinOwned` moves two owned
root tickets into one producer aggregate. `startSplit`/`splitStep` retain both
children before releasing their aggregate and transfer exactly those two tickets
back. Duplicate handles require duplicate tickets. All intermediate split roots
are explicit; the terminal pair is separately owned. The source scenario owner
audits every split state and rejects a planted unconsumed-root mutation.

## Operational versus canonical semantic cost

RuntimeResources.work and uorc.logical-owned-bytes/1 are operational source
work/storage, including actual transport partition, normalization, ownership and
copy operations. They are not a canonical semantic certificate budget. Literal admission now streams each offered span through RuntimeByteBuilder,
producing full4096-byte leaves and a largest-to-smallest power-of-two residual
tail independently of offered span boundaries. The borrowed image stays pinned
until the whole span is copied and acknowledged. Builder reservations, copies,
intermediate roots and cancellation are explicit and operationally charged.
CanonicalLiteralTests invokes both one-span and split-span admission, checks
ordered bytes and canonical tail shape, and audits final root tickets. This
finite control is not a general normalization/refinement proof. This checkpoint
makes no semantic-budget minimum claim. A distinct canonical cost profile and an
ordinarily verified reference/owned event bridge are required before such a
claim. The4096-byte policy is a leaf policy, never a
whole-input eligibility limit. All real extra normalization work remains charged
operationally. Source checking, Rust export and pinned-SDK qualification of this
new admission closure are still separate open checks.

Canonical builder state is included in the fixed session/transfer inventory.
Its twelve bins, completed sequence and control ticket remain owned between
acknowledged spans. While a native builder task is pending, that task owns the
builder tickets; they are not also counted as session roots. Cancellation can
drain a pending release and reclaim reservations without dropping caller roots.
The source task fee prepays the known4096-byte maximum copy per native builder
step; this is a bounded leaf operation, not a whole-object limit.

### Bounded credit funding

`RuntimeAdmissionCredit.fund(cursor, availableCredits)` pays at most the one
current operation in a bounded fixed-width update. It returns `Funded` with the
resulting `Event` and `remainingCredits`. Partial payment does not execute the
operation. Full payment permits exactly one operation, and excess credit is
returned instead of funding its successor implicitly. Zero credit only exposes
the current observation. `tick` delegates to `fund(cursor, 1)`.

Counter overflow retains the current arena and all unspent credit. If the final
payment succeeded but the attempt counter cannot advance, the rejected state
retains that paid prefix without executing. Finite source controls cover these
cases and reject a planted premature-initialization mutation. The general
zero-credit observation law is source-authored and awaits ordinary proof/axiom
qualification; the local scenario evaluator is not that qualification.

### Backward-reference convention and cross-model control

Local operands, captures, block returns and prior-block operands use the same
one-based backward distances as Graph and ReferenceSemantics: valid distances
are1 through the current count, and the absolute index is count minus distance.
Distance0 is invalid. Stored wire/source distances are not normalized to a
separate zero-based dialect. Absolute slot/index keys, entry indices and
instruction/lifetime timestamps remain zero-based.

RuntimeProgram.backwardIndex owns this rule; localIndex/priorBlockIndex and the
owned reader/admission helpers delegate to it. RuntimeReferenceTests compares
actual helper results with Graph.localType/priorBlock on distinct-type slots
and blocks, and rejects a planted zero-based implementation. This control was
added after a real empty-template integration exposed a mismatch that the older
owned-only fixtures had missed. Correcting those fixtures is not a change to
Graph, ReferenceSemantics or the wire language. General refinement and ordinary
SDK/kernel qualification remain separate obligations.
