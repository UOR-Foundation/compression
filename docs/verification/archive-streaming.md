# Owned archive parsing and object boundaries

This implementation increment uses shared Core source
`f947c988c6769fac2d94a850afee19015d18653c` and the unchanged production
`Integrity.Profile`, `OwnedHash` and `ArchiveBinding` modules from public
`2c929d7482b8c2b98a5b965c684790abf8b8995f` (#84). It continues wire #11,
parser #80, emitter #83 and integration #78. Historical archive-v1/profile02
issue wording is not a compatibility authority.

## Implemented source

`ArchiveParse.begin(arena, usage)`, `feed`, `advance`, `finishInput`,
`acknowledge`, `cancel` and `cleanup` implement bounded frame syntax over
already charged image spans. Events use `FrameEventContract.Event`. There is
one pending event, explicit input demand, exactly one EOF, sticky typed failures
and prepaid resumable cleanup. Header numbers use checked minimal ULEB; raw
length sums, preimage/body/trailer boundaries and operational addressability are
checked. Program body events end exactly at body length, before integrity bytes.

`Integrity.Profile.select` owns format dispatch: format 2/machine 1 preserves
CHAIN-v1; format 3/machine 1 selects STREAM-MAX-v1. The frame contract's dispatch
now returns that selected identity as data. It does not certify an archive.
No checksum identity or certification fields are added here. The separately
owned integrity layer remains responsible for hashing and digest comparison.

`LengthStore` commits every ordered length, including zeros, through the Core
reserved `RuntimeTable.PutCursor`. It stores natural-value slots with wire role.
The parser refuses to acknowledge an object length until `lengthStored(table)`
can read the actual indexed value. `FrameEventContract.lengthAt` dispatches on
the declared table kind: wire natural slots or the earlier metadata-record
carrier. The emitter uses the same accessor, preserving its earlier carrier
and wire bytes. This is internal metadata representation dispatch, not a
second wire parser or a diagnostic fallback.

`ObjectDemux` reads that charged table, emits begin/end for every object,
including empty objects, and fences each byte span to its object. It advances
the index only after acknowledgment of endObject. Exact reconstruction EOF,
aggregate output size, missing lengths and trailing/short output are checked.
Its sink must finish and prepay each actual byte comparison/copy before a
consumed acknowledgment; hash equality cannot replace reconstruction equality.

Each component control contains fixed scalar/handle state. LengthStore embeds
only the Core table cursor, never its Arena-bearing transition. Driving
sessions move the same actual Arena and Usage between operations. Input owners,
length owners, pending transfers and Core cursor owners are explicit. No
uncharged length list, recursive Graph/Rope, byte buffer, signature cache or
replacement table cursor is retained by these components. The existing
GraphParse/FrameParse remain independent diagnostic references.

## Executed checks and limits

The first runtime suites exercise seven format-2/3 frame fixtures across 494
split positions, with empty chunks, conditional basis spans, raw/program bodies,
empty objects and multi-byte lengths. Thirteen malformed-header, minimality,
overflow, truncation, trailing-byte and length-arithmetic cases reach their
typed failures. The ordered-object suite exercises five fixtures across 141
split positions, checks every payload byte and boundary, and rejects short and
trailing reconstruction. Cleanup audits retain exactly the original test image
ticket after all parser/table/demultiplexer owners are released.

These are source-owned finite runtime scenarios, executed with the existing
LexLean #393 compiler at `3fe13b50e8d3b20e7e24d7fc8cd6b7a232c1471e` and Lean
4.32.1. The computational diagnostic projection excludes inherited theorems and
uncalled dependency declarations. It does not establish general refinement,
physical allocation/copy elision, production certificates or SDK acceptance.
Operational U64 refusal does not narrow the mathematical finite wire language.

The new frame syntax calls the actual integrity selector. That public source
currently lacks an executable declaration on `Profile.select`; asserting an
executable caller produces diagnostic `LLT4001`, "calls non-executable
Uorc.Integrity.Profile::select". The wire source does not change the integrity
owner's annotations or duplicate its selector. Executable/export eligibility
must be qualified across that dependency before declaring a production root.
The implemented source definitions remain available for native computational
execution. The exact pinned SDK and complete `just vv` have not been run.

## Continuing composition

This checkpoint backs up implemented frame/length/object components while
closed composition continues. The next increment connects ProgramParse and
Core's actual admission credit/detach/reattach interface, then the owned
integrity length fences and digest check over the retained archive snapshot.
Pending span tickets must remain inventoried through acknowledgment; admission
and producer controls must never retain separate Arena snapshots. Encoded
one-based backward distances pass through unchanged. No successful Ready or
accepted-archive token is constructed by these syntax components.

Mid-cursor cancellation, pause/transfer failure prefixes, physical multi-image
partitions, complete negative reconciliation, general proofs and end-to-end
admission/hash/execution/reconstruction remain qualification work in this same
lane. They are not replaced by the finite syntax controls above.

## Additional ownership and admission controls

The borrowed feed additions to ProgramParse/ArchiveParse validate an external
live image ticket or wire ownerRoot and retain no second input ticket.
ProgramParse.acknowledgeCopied is an internal synchronous-copy acknowledgment;
the admission controller invokes it only after actual Core awaiting confirms
that the offered event has finished. A pending literal is not acknowledged merely
because its event has been offered. BorrowedInputScenarios executes pending
literal lifetime, byte reads, pause/resume counters, cancellation cleanup and
invalid-owner controls.

LengthStoreScenarios cancels at every actual mutation prefix through the first
stored length (138 prefixes). This found a real final-drain seam: calling Core
RuntimeTable.cancel while its normal control release is in progress can report
cancelled without returning or releasing the completed subtree. TableCancelRepro
isolates that behavior without LengthStore: 133 Core steps reach draining with
null control and a live subtree; the cancelled result then fails HeapAudit.
The wire controller now advances an existing Core release before initiating
cancellation, handles the actual complete(table) result if returned, and releases
that table. The Core cursor algorithm is unchanged. All cancellation prefixes
and the earlier parser/demultiplexer/emitter suites pass with that controller fix.

ProgramAdmission implements ProgramParse -> RuntimeAdmissionCredit -> actual
Core Ready, moving the single Arena through Core's actual Taken/Returning
boundary. Its control contains no Arena. One literal-program control reaches
actual Ready and passes the complete root audit in native diagnostic execution.
This is a narrower implemented composition than complete archive verification.
Cancellation retains the actual Core phase for its cancellation owner; a complete
controller cancellation API and lifecycle qualification remain open.

Exact shared Core source blocks this consumer's source check: RuntimeAdmissionBoundary
omits direct imports of RuntimeAdmissionOrder, RuntimeInstructionCheck and
RuntimeLifetime. After those imports are added only in an isolated diagnostic
projection, RuntimeAdmissionCredit also needs hygienic arena binders (the module
has a declaration named arena). Native admission execution above used those
explicit dependency-only import/alpha-renaming repairs. They are not published
Core fixes or exact-source acceptance. The wire parser/controller declaration
payloads are preserved. No successful Ready was fabricated.

See frame-domain-audit.md for the separately executed Nat header-domain controls.
The generic U64 varint contract must not be confused with FrameParse's own Nat
header parser. Full archive hashing, admission, machine output, object boundaries
and every reconstructed byte still need one closed production controller; the
current checkpoint does not claim that result or pinned-SDK acceptance.

The later public Core import delta `765742af7aaa3e1ee409c408847773b9eedbbef7`
is now applied as exactly three header lines, with this branch's semanticdata
payload unchanged. The unpatched admission consumer check progresses to the
RuntimeAdmissionCredit arena-binder collision, recorded in
`admission-public-header-consumer-diagnostic.json`. No private binder repair is
substituted into authoritative Core source.


## Current shared-Core qualification

The earlier f947 missing-import/binder receipts above are historical and are
superseded by the actual public dependency closure at
`ad70ad9a32bff5cca1a039a60794ac5eb0e7c079`, with the unchanged Boundary semanticdata
and latest public imports from `95683d8ecda91a17ee0655686a9f42063312ecb9`.
The credit module is byte-verified as
`b9cbd54aed802621167824c59abdf32d64e47b38f4c16194193976349abbb42e`.
The full imported Core closure is recorded in archive-shared-core-provenance.json.
No private alpha-renaming or Core algorithm repair is used by the current run.

The current source consumer check and generated native execution pass. The
admission owner checks actual Ready and also rejects encoded return distance
zero after the identical earlier literal-event prefix. That retained failure
passes the complete root audit, then the actual Core cancellation owner cleans
it to the original caller image. Encoded distances remain unchanged.
FrameOwnershipScenarios additionally checks pause, cancellation with a pending
body span, explicit retained-ticket transfer and final release. These controls
and all earlier frame, object, emission and 138 length-cancellation-prefix
controls pass on current shared Core. Exact source/build identities are in
archive-current-shared-core-diagnostic.json. Diagnostic theorem/dependency
slicing remains explicit; locked-SDK acceptance and the full archive controller
are still open.
