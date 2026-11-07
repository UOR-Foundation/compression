<!-- uorc:non-authoritative -->
# Product API and View component evidence

The implementation is in the source modules below. This document records the
component boundary and executed evidence; the LexLean graph remains authoritative.

| Source | Implemented component |
| --- | --- |
| `Uorc.Product.Api` | Typed requests for all ten operations, limits, resource/span leases, failures, accounting, result details, report admission and evidence projection |
| `Uorc.Product.InputModel` | Ordered object and arbitrary-byte-span control, empty objects and explicit EOF |
| `Uorc.Product.Control` | Operation roots, run ownership, transitions/effects, work extension, checkpoints, staged output, publication authorization/receipts and cleanup |
| `Uorc.Product.Workspace` | File-selection tokens, retained draft resources, request construction, dispatch, checkpoint continuation and shutdown |
| `Uorc.Product.Presentation` | Generated typed labels, actions, file fields, enabled states, exact metrics, evidence/scope/search values, focus and live-region behavior |
| `Uorc.Product.Scenarios` | Typed trace owners, positive/negative cases, generated presentation observations and mutation ownership |
| `Uorc.Product.Storage` | Per-category and total requested quotas, reservations, achieved writes/deletions, transfer accounting and typed refusal |
| `Uorc.Product.ResourceIO` | Run-bound bounded read/write demands, retained image tickets, unknown-length spool/seal, partial failure, cancellation and cleanup receipts |
| `Uorc.Product.Selection` | Ordered unique item IDs, display metadata, removal and reordering |
| `Uorc.Product.HostContract` | Self-contained source-owned generic adapter/root/renderer contract projected to JSON |

The existing `Session`, `Evidence`, `StreamProtocol` and `CompletionModel` files
are unchanged. The controller calls their actual admission, input, completion and
publication functions. `Details` carries operation-specific archive, basis,
frontier, comparison and developer reports. `compareExactArchiveBytes` compares
the archive field; `archivePlusBasisTransfer` exposes the separate charged sum.

Selected object rows expose safe display names, observed lengths, empty objects,
order and source-owned removal/reordering controls. Immutable collection changes
retain the previous owner until a matching callback, restore it on failure and
reject stale replies. Display names are separate from archive semantics: this
archive model preserves ordered bytes and boundaries, without implying restored
filenames, directory paths or filesystem metadata.

Ordinary operations request no additional output-size quota. Advanced numeric
fields expose work, output, logical arena and external storage requests. Their
natural-number parser, formatting, optional clearing, stale-binding checks,
messages and focus are modeled. Real host capacity and runtime addressability
remain separate admission boundaries. A transport quantum is never a total
input or product feature limit.

The bounded preparation owner reads and writes a single chunk at a time, spools
single-pass or unknown-length sources under storage admission, then seals the
same lease at its exact achieved length for reread. Known immutable seekable
sources can be reread directly. Partial failed writes retain achieved storage;
cancellation waits for a valid achieved write reply before releasing the held
image ticket. Deletion receipts account actual freed bytes and allow partial
cleanup retry. Retired jobs retain run-targeted storage accounting without
regaining publication authority. Cleanup cannot forget nonzero reservations or
retained storage. Logical arena bytes, staging/spool/spill/copy storage, archive
bytes, basis transfer and optional scoped physical observations stay separate.

In-app panel navigation retains current jobs and results. Input edits,
cancellation, detachment and shutdown revoke ownership before cleanup.

The operation roots return `Transition`, including the typed command that starts
the actual operation. They do not fabricate an executor or a successful archive.
`runAcceptance` and `runCoreRequest` remain available to tooling. Their operation
controls are separated from the ordinary operation list in the presentation.

## Execution evidence

`scripts/product-control-validation.py` verifies a temporary dependency closure,
compiles its exact generated Lean modules, and runs the source-owned trace and
presentation inventories. The harness only iterates cases and reports their
Boolean results. It contains no UORC transition, codec or presentation behavior.
`validation/product/evidence.json` binds the transcript, compiler, source hashes,
gate hash, complete inventory counts, kernel attestation and axiom audit.

The saved official diagnostic compiler is LexLean 0.4.0, with Lean 4.32.1. This
is separate from the repository's pinned SDK. In the explicitly requested
compatibility projection, only the existing `Session.ownsRun` and
`Session.finish` executable annotations are omitted: the newer compiler rejects
those annotations in the repository's language-1.1 data notation. Their terms
are unchanged, and the repository files and locks are untouched. The receipt
records these omissions. This is component diagnostic evidence, not exact-source
pinned-SDK acceptance.

The finite inventory exercises repeated edits, double start/cancel/download,
obsolete completions, ordered empty objects, EOF/demand errors, retry after
failure, active effort extension, wrong basis, publication failure/retry/unknown
outcomes, transferred download ownership, navigation/back/close, detachment,
idempotent shutdown, navigation that retains current results, picker races, request construction and generated View
values. Typed failure fixtures test propagation at this boundary; they do not
replace the parser, basis checker, search or execution workers' own tests.
Synthetic `Report` observations are explicitly control fixtures, not measured
compression results.

A deliberately ill-typed user-intent probe is rejected with `LLT4001`: public
Workspace intents cannot carry a forged executor completion. Runtime notices
enter through the separate `Workspace.receiveRuntime` root, which is bound to
the trusted generated executor adapter.

Four defects are planted in the temporary source and restored byte-for-byte:

- Bypassing the current-run guard in `Control.completed` fails the general
  `completion_guard_binding` kernel theorem.
- Inventing original-input availability in `Api.evidenceView` passes linking
  and kernel checks but fails the generated runtime observation
  `decompression-does-not-claim-equality`.
- Bypassing storage admission in `Storage.fits` fails
  `storage-admission-achieved-failure-transfer` in the generated runtime.
- Accepting cancellation before the pending write acknowledgment fails
  `double-cancel-waits-for-achieved-write` in the bounded I/O runtime owner.

The current full component inventory contains 112 unique runtime cases and
observations. General kernel theorems bind stale-completion admission and show
that canceling a reader clears its demand and releases a held ticket at most
once. Decimal formatting uses the standard natural quotient/remainder semantics;
its exact `propext` dependency is declared only on the affected definitions and
observed closure. No universal proof policy was changed.

The restored source is verified and executed again in a clean runtime directory.
The Rust receipt tests only bind that executed diagnostic to the current files
and preserve the open qualification obligations. They do not upgrade this
component observation to native, browser or release acceptance.

## Exact integration dependencies

The source API names are `Uorc.Product.Control.{compress,decompress,inspect,
verify,resume,buildBasis,analyzeFrontier,compare,runAcceptance,runCoreRequest,step}`,
`Workspace.invokeUser`, `Workspace.editLimitText`, trusted resource `Workspace.dispatch`,
`Workspace.receiveRuntime`, `Presentation.present`, and
`Control.authorizePublication`. With the current `module_prefix`, the diagnostic
compiler qualifies emitted names as `Uorc.Uorc.Product.*`. The actual extracted
`Api.ResourceRef`/`Api.sameResource` production probe exported
`Uorc.Uorc.Product.Api.sameResource`; requesting the single-prefix name failed
with `LLC0202`. Its emitted manifest is
`validation/product/root-probe/result.json`, result ID
`6a1b2108040af0ffd75ecc399e9cbff4fd96cc893e22d293da7f7d82441c0621`.
Artifact replay passed and its actual generated Rust library passed five native
identity/mismatch controls. This establishes the configured module-prefix
behavior for that extracted root. It does not qualify the complete product,
PrismPM Library/V1, its outer ComputeError or the pinned SDK. Source qualification
has therefore been preserved rather than globally renamed.

The self-contained generic host contract is the actual projection at
`validation/product/host-contract.json`, generated by
`scripts/product-host-projection.py --write` from `HostContract.contract`.
The component gate rejects a stale projection. It specifies typed endpoint
signatures, Workspace ownership, raw decimal-field callbacks, safe selected-item
rendering, focus epochs, keyboard/live behavior, bounded I/O, storage and cleanup
receipts, the single-arena transfer and span-ticket ownership. Its qualification
fields remain false until those production boundaries execute.

The production runtime/wire/search/integrity owners still supply independently
checked observations and actual charged resources. The product layer supplies
control decisions and commands. The committed RuntimeProgram event and caller-root
APIs are not treated as completed executors. The inspected runtime branch
`277d0f6b8664ba4b2dff4df4564839cd80049ad3` still exposes preparation,
preflight, captures and iteration controls; it supplies no closed start/step
owner. Parser revision `9e0b052064a18f84e6d79c47f0d9947c742c8500` exposes
owned parser demand/acknowledgment, but the announced boundary-only admission
AwaitingState and detach/reattach pair were not present in the inspected source.
The product operation bindings require those exact roots before moving the
single actual Arena through producer/admission and then into RuntimeMachine.
A separate parser arena snapshot or fabricated successful executor has not been
introduced.

The generic persistent host boundary needs:

- Instance/slot/generation-checked leases and charged borrowed spans; actual
  basis/source bytes; ordered object/demand/EOF forwarding; bounded work ticks;
  progress, checkpoint, staged-byte, failure and completion events.
- Request-root retention before disposal of a previous job. A retained job owns
  its report metadata, including borrowed frontier/scope roots. Product-owned
  output/checkpoint leases have explicit release commands. A successful output
  receipt transfers custody to the destination.
- Publication cancellation and a serialized final check through
  `authorizePublication`. Known failed attempts retain staged bytes. Unknown
  disposition is distinct and prevents blind retry. No atomic rollback promise
  is inferred from the old adapter.
- The actual generated outer ComputeError, distinct modeled failures, explicit
  fatal/unknown disposition, release/cancel/stale-handle handling, and no generic
  allocation-trap recovery promise.
- A generic safe-text renderer for the typed View and exact Workspace intents,
  preserving ordered multi-object selection, enabled states, keyboard behavior,
  focus and live regions. File-picker receipts include the modeled tokens.

BinaryProgram/1's whole-buffer transport and fresh-instance Core-Wasm do not
supply this persistent contract. The upstream adapter and generic View host are
still integration dependencies. Native/CLI/browser product qualification and
actual generated View execution have not been observed here.

Pinned SDK acceptance has not been run. The required digest-selected SDK guard
remains intact; these component diagnostics do not satisfy that gate.
Workspace BDD still reports the inherited runtime, wire,
integrity, synthesis, basis, checkpoint and real-execution test-name obligations;
those lanes have not been replaced with artificial passing tests.

## Source review

These reviews informed concrete boundaries, without importing their results as
compression evidence:

| Repository / inspected revision | Relevant source observation |
| --- | --- |
| `afflom/rule30` / `d3b39a1` | `rule30-api/src/Application.lex.tex` and `Presentation.lex.tex` own application, navigation and presentation values |
| `UOR-Foundation/template` / `8e1af26` | `app/src/Contract.lex.tex` and `Foundation/View/Text/V1/Model.lex.tex` keep model, scenarios and presentation under source ownership |
| `UOR-Foundation/UOR-Framework` / `51c0138` | `foundation/src/enforcement.rs` separates validated witness construction and declarative data; inhabitance evidence has explicit restricted scope |
| `UOR-Foundation/Prism` / `507995b` | `crates/uor-prism/src/lib.rs` exposes standard-library composition and sealed validated/certified boundaries; it supplies no compression optimality evidence |
| `UOR-Foundation/MNIST` / `515f75f` | Source/generated rendering and measured head-to-head evidence distinguish computational semantics from empirical accuracy and timing; neither establishes compression effectiveness |
| `afflom/lexlean-matmul` / `d2ed5e6` | `src/ResearchModel.lex.tex` binds research admission to the actual dispatcher; presentation does not invent a second computational path |

No archive/profile or integrity identity was changed, and no historical hex,
media-type or whole-buffer limit was added to the product model.
