# Runtime byte builder

`RuntimeByteBuilder.lex.tex` is the authoritative executable implementation. It imports the handle arena and `RuntimeSequence`; it does not import mathematical `Bytes.Rope`, `ByteBuilder`, a store trie, or reference semantics. Its continuation fields contain only fixed records, handles, word counters, a byte and flags. Immutable byte payloads reside in arena image entries.

## Ownership interface

- `begin(arena) : Result BuilderTransition BuilderFailure` allocates one fixed control-capacity owner and returns `Outcome.builder`.
- `beginAppendOwned(arena, builder, byteValue)` transfers every builder ticket to a pending cursor and checks total-length overflow before any allocation.
- `beginFinishOwned(arena, builder)` transfers every builder ticket and begins the final descending bin scan.
- `step(arena, cursor)` executes one continuation event.
- `cancel(arena, cursor)` transfers all nested sequence-owned tickets into four fixed cleanup slots, clears the nested cursor, and starts cancellation. Repeated `step` calls produce `Outcome.cancelled`.
- `builderRoots(builder, outside)` and `ownedRoots(cursor)` expose the external ticket inventory for composition with independently tracked program, input, archive, search, checkpoint and other roots. These bounded diagnostic lists are never saved in a product cursor.

`BuilderTransition` contains `arena`, `cursor`, `delta`, `work`, `copied` and `outcome`. Outcomes are `pending`, `builder(Builder)`, `sequence(Handle)` and `cancelled`. A terminal transition clears its cursor and transfers the returned builder/sequence tickets into its outcome. Save the returned arena and continuation after every successful event. Owned APIs do not retain or duplicate record tickets. Keeping a checkpoint requires explicitly retaining each nonzero record ticket before transferring the other copy. There is no borrowed builder entry point.

Every event performs at most one arena entry allocation. An `allocateMoved` event additionally consumes a temporary root after attaching its parent edge; release events advance at most one reference. A failed event returns a typed error without replacing the pre-event arena or cursor. Earlier successful events remain committed and their `Delta`s remain part of caller usage accounting. Cancel that **current** cursor after a failure; reusing the original begin state would discard the achieved prefix. There is no hidden rollback or drain loop.

`BuilderFailure` distinguishes heap failures, nested sequence failures, invalid builders, invalid cursors, invalid images and total-length overflow. Heap errors preserve their exact typed cause. An error from a nested sequence event similarly preserves that event's prior cursor, including its already allocated plan prefix.

## Fixed records and their logical charge

A builder is explicitly packable as fifteen 64-bit words: twelve bin handles, one completed sequence handle, one total-byte count and one control handle (120 bytes). A bin at position `k` owns one immutable runtime image of length `2^k`, for `k = 0..11`. A zero bin owns no ticket. The completed sequence has no input-count bound other than its checked UInt64 length.

The nested sequence cursor uses 51 UInt64 words plus a byte; packing that byte as a full word gives 52 words. The builder cursor is 84 words (672 bytes): builder 15, nested sequence 52, phase/mode 2, five operation image/leaf tickets 5, four nested cancellation tickets 4, level/length/resume/cleanup 4, byte 1, active flag 1.

The maximum saved transition/result record adds outcome tag and builder payload (16 words = 128 bytes), nine Delta words (72 bytes), work and copied counters (16 bytes), an arena reference (8 bytes), and the Result tag (8 bytes). Together this is 904 modeled fixed bytes. One actual arena control-capacity entry charges `48 + 904 = 952` bytes, leaving 48 bytes beyond this explicitly packed record. `Heap.allocateControl` uses the existing capacity-node shape `Node.imageReservation(ImageKind.control, 904)`. It is retainable/parentable for checkpoints, is not a byte-image reservation, cannot be committed into a byte image, and is rejected as a sequence backing. No padding payload is materialized or copied. This is a deterministic logical charge, not a host-layout, allocator, stack-use or RSS claim; arena/index representations and transient borrowed compiler values retain the separate qualifications in `runtime-heap.md`.

Initialization allocates that single owner in one event and produces the stable builder directly. Normal finish and cancellation release control last; because it is one entry, its full charge remains until the exact release step that emits the terminal outcome. A multi-entry control charge would incorrectly erode during resumable final reclamation.


## Reserve, copy, commit and release

Singleton construction selects one of 256 literal one-byte images through exactly eight binary decisions. It does not construct or index a 256-element runtime list. One event reserves a one-byte runtime image; the next commits the selected literal. The committed image becomes the carry ticket.

An occupied bin causes an event that reserves `2 * carry.length` bytes. Only the following merge event preflights commitment and then reads the two old images locally, validates their runtime kind and exact lengths, evaluates immutable `Bytes.append`, and commits the promised new image. Both old image root tickets remain owned until commitment succeeds. The fresh image becomes the carry, and the two old tickets move to explicit cleanup fields. Subsequent events release them through the arena's resumable release cursor. A release step processes one reference. No allocation occurs while that release is active.

`Heap.preflightImageCommit` validates the live unique reservation, exact promised length and charge, release quiescence, replacement budget/counters and fresh handle availability before any payload is constructed. Thus a reservation that used the last available ID fails before singleton construction or `Bytes.append`, even when the arena changed between reserve and commit events. Both preflight and commitment use the same immutable arena within that event.

`copied` is zero for reservation/sequence/release events, one for singleton commitment, and the sum of the actual two validated input lengths for merge commitment. Thus every event copies at most 4096 payload bytes. For `n` appended bytes the successful total is `n + sum(k=1..12, floor(n/2^k)*2^k)`. Surviving bins keep their image identities.

A full 4096-byte carry is attached to one sequence leaf with `allocateMoved`; its backing bytes are not copied. `RuntimeSequence.beginConcat` and its bounded `step` events join that leaf to the completed sequence. This builder contains no second balancing implementation. Original completed-sequence and leaf tickets stay owned through the nested join and are released only after the new sequence root is obtained. Finish visits bins 11 down to 0, hence older bytes before newer bytes, converts each surviving backing into a zero-copy leaf and uses the same sequence join.

Cancellation preserves every pending reservation, fresh image, old input, leaf, completed sequence, bin and nested sequence owner. If a release is already active, cancellation drains that transferred ticket first without counting it again or skipping a cleanup slot. It then releases the fixed ticket inventory in bounded events, with control last. During the final control release, the existing leaf field holds a borrowed identity of that control owner, and both ticket inventories exclude it. Cancellation verifies the builder phase, cleared control field, initial root-step shape and exact pending root ID before calling `Heap.withdrawRootRelease`. This transfers the same unprocessed root back to the builder without allocation or refcount changes. The output is then released before control. A foreign pending release is rejected, and a release that has already processed an edge cannot be withdrawn. This keeps the fixed charge live throughout cancellation of the final release. Callers must serialize arena release ownership as required by the heap protocol.

## Validation scope

The 35 source-owned scenarios compare interpreted output to binary literals, independently inventory external tickets at each observed event, and exercise singleton values, carry order, 4095/4096/4097/8193-byte lengths, exact merge reservation budgets and cancellation checkpoints. These are executable invariant tests, not a general refinement proof.

The isolated language-1.2 diagnostic fixture and generated Lean execution do not replace the repository's locked-SDK `just vv`. Production locks and the main project remain unchanged by this worker. Successful source linking, scoped kernel compilation and official generated Lean execution must be reported separately from generated Rust, pinned SDK and full release acceptance. The later [source factoring](runtime-byte-builder-factoring.md) preserves full step results and has its own current-source checks; Product97's certificate-B size refusal remains separately scoped.

The final isolated source snapshot passes official SDK formatting, exact-byte SDK roundtrip, linking and build. Its six generated Lean modules compile with Lean 4.32.1; the final generated files are byte-identical to the kernel/native inputs. All 35 scenarios pass in the native diagnostic executable built solely from those generated modules and the official Lean runtime. The handwritten diagnostic main only asserts the source-owned Boolean scenarios. Exact actual axiom sets match all 102 definition policies and are empty or `{Quot.sound, propext}`.

The representative mutation reverses the actual `Bytes.append` operands in the runtime merge. The literal-output gate rejects that source mutation; restoration is checked by SHA-256 and the restored gate passes. Worker receipts, source inventories, commands, module byte-equality checks and axiom traces are retained in `/tmp/runtime-byte-builder-evidence.json` and `/tmp/runtime-byte-builder-axioms.json` for integration. The superseded interpreted 32-case run was stopped after the current 35-case native diagnostic passed; no complete interpreted-suite result is claimed.
