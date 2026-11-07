<!-- uorc:non-authoritative -->
# Bounded read and image attachment controller

Authorities are `Uorc.Product.ReadController`, its scenarios, ReadImage and
ResourceIO.Reader. `begin(run, arenaBinding, source, quantum, arena, usage)` creates
one moving Session. Its cursor, nested ReadImage cursor, native receipts and
consumer tickets retain no Arena or payload Bytes. The outer cursor and intake
owner have separately charged controls.

`demand` and `tick` emit source-owned read, payload and attachment commands. ReadImage
first reserves the exact image capacity, then commits the bounded immutable
payload to one actual image ticket. Nonempty data reaches ResourceIO.ReadReply
only after the exact attachment receipt. Empty EOF creates no mapping/image.

The native arenaBinding must name the actual resident moving owner cell. Token
comparison alone does not prove same-Arena lineage. Attachment registers a
same-instance native mapping for the already-owned image ticket without another
Heap.retain. Reader.held and ConsumerTicket alias that one aggregate. The raw
payload lease and image mapping remain distinct cleanup obligations.

Full consumer acknowledgement or actual consumer stop precedes native mapping
unregistration. Only matching `imageUnregistered` permits source Heap.release and
releaseStep. `unregisterImage` and image-valued `releaseIncoming` remove mappings;
they never release Heap roots. Raw leases have their own achieved release receipt.
Cancellation keeps pending read/payload/attachment/consumer custody until exact
quiescence or late completion. Mutation deltas remain until acknowledged. A fixed
stale-incoming slot provides backpressure; further rejected incoming custody must
remain with the enclosing dispatcher. `releaseDisposition` and
`transitionDisposition` include unconsumed outcomes.

The original canonical diagnostic has 22 selected source files, 25 compiled Lean
modules, 50/50 VM controls, 160 exact new-family policies and three actual mutants.
It imposes no total-input cap and makes no generic-host or native ABI claim.

The repository files were later converted through the official SDK to existing
Product data-form style. Both original and converted raw hashes are archived.
The subsequent official normalized language-1.2 rebuild reproduced the frozen
build/source/semantic IDs and byte-identical hashes for all 22 generated Lean
modules. The archived VM run remains the original run; no new execution is
claimed. Annotation-only equality is a separate earlier comparison. Production
language/locks stayed unchanged; native/NativeLibrary/Wasm, pinned-SDK and
whole-current integration remain unqualified. Raw in-place mutant source bytes
were not retained; exact hashes, generated source/code and executed transforms
and failures are preserved explicitly.
