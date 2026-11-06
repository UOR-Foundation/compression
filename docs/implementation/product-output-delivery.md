<!-- uorc:non-authoritative -->
# Reporting achieved output delivery

Authorities are `Uorc.Product.OutputDelivery`, `OutputDeliveryScenarios`, and their
imported WriteImage, ResourceIO, Storage, StreamProtocol and Control owners.

`start(context, outputTransition)` retains one exact WriteImage transition in an
Envelope. ReportContext freezes run, destination, enclosing Accounting, work
prefix and WorkLane. Enclosing wire/admission/hash/caller-root controls remain
caller-owned without an Arena and stay pinned. No second Arena or Wire Session
is created.

`deliver(state, currentControl, observations)` retains the exact envelope for
waiting, recoverable refusal, early source failure requiring cleanup, rejected
callbacks, stale delivery, incomplete custody, missing/scoped measurements and
arithmetic errors. `takeRetained` moves it back once for continuation or retired
cleanup. A stale settlement cannot replace the new run's ledger or erase old
nonzero Storage.

Observations explicitly supply cumulative enclosing-operation transport and basis
transfer totals through the delivery, bound to run and destination, plus optional
physical observations and actual StreamProtocol.Publication. Missing data never
becomes zero traffic or notPublished. Checked work-prefix subtraction and checked
lane addition fold the suffix once. Exact archive size, basis size, object count,
input and produced-output meanings remain frozen. Writer.offset is acknowledged
destination progress, not an inferred archive/transport/basis metric. Logical
peak Arena bytes use the larger observed peak; actual Writer Storage replaces the
prior ledger snapshot. Physical scope/sequence and absence remain explicit.

A completed fold moves the envelope into a Receipt once. For a terminal failure,
`adoptWriter` calls `Control.applyStorageState` only after matching the current run,
then `Control.runtimeFailed` preserves achieved accounting and publication. Direct
`Control.fail` is not used because it defaults to notPublished. A substantive
recorded I/O or Storage failure survives a later cancellation.

A successful settled writer continues its enclosing operation. It creates no
Control.completed report and no verified/archive-proof facts. If an enclosing
publication request is already pending, its explicit observation uses the existing
Control.publication owner. A failed staged-output write and a failed
publicationCopy remain distinct Storage categories. Failed/cancelled writing does
not prove rollback. Unknown publication remains unknown and prevents blind retry;
publication reconciliation remains the enclosing publisher's responsibility.

The checkpoint has 32/32 actual writer-lifecycle VM controls, 29 compiled kernel
modules, 103 exact new-family axiom policies, and five owning defect rejections.
It uses the earlier Core diagnostic composition and exact WriteImage checkpoint,
plus current factored Control raw source `f5634512…` and hash-bound Api/Storage/
ResourceIO inputs. It is not whole-current-integration or accepted-export evidence.
Native/SDK outer-failure retention, NativeLibrary/Wasm binding, closed-archive
execution, pinned-SDK acceptance and physical resource qualification remain open.
