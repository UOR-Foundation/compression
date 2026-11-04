<!-- uorc:non-authoritative -->

# Machine native qualification handoff

`production-request.json` requests the eight actual RuntimeMachineEngine API
roots, with the official rust-std target descriptor reported by LexLean 0.4.0.
Its SHA256 is
`973b82219bc70cf278c30b6559d923a76a3db470b0caaad9a7741b78d1bd1618`.
This is export preparation, not an accepted package or replacement SDK lock.
The existing compiler qualification route must run the supported
`lexlean production build --request ...` and `production verify-artifact` on the
exact completed composition. If that compiler's registered target descriptor
changes, regenerate the request from its official `production targets`; do not
silently substitute descriptors, source IDs or acceptance records.

No new compiler is built by the machine lane. The locally acquired official
0.4.0 release predates the recursive-ADT preservation correction tracked by
LexLean #310 / #393. The fixed source was published at
3fe13b50e8d3b20e7e24d7fc8cd6b7a232c1471e, but its complete upstream acceptance
and the consuming compiler artifact are coordinated separately. No source-only
fix is treated as a compiler binary or accepted machine export here.

After the actual package is available, retain its compiler/source/lock/target
identities, certificate and replay results, package archive hashes and canonical
artifact URL. Inspect the generated State, frame-list, table-path, sequence,
Heap index and image carriers before writing a host measurement consumer. Its
field layout and Clone/Rc behavior must come from that exact generated package.
No guessed Rust mirror or handwritten decoder is a measurement substitute.

The consumer should execute the source-owned typed controls through the genuine
export and report each result independently of timing and memory measurements.
Measure allocation/copy activity and peak RSS for at least: growing literals at
4095/4096/4097 bytes, branched sequence equality and concatenation, nested
Apply/Tabulate/Fold/Scan, table partial-allocation refusal and resume, retained
caller aliases, detached output after cleanup, and cancellation in every owner
phase. Retain one live state at a time for normal execution; separately measure
intentional host checkpoint retention and report the retained-state count.
Record OS/architecture, compiler flags, allocator/instrumentation, input bytes,
logical Usage/peak, achieved delta receipts, physical allocation/copy totals,
peak RSS and repeatability. A failed or incomplete operational run supplies no
semantic predecessor-exclusion evidence.

Logical owned bytes, operational work credits, canonical semantic admissibility
and physical measurements are four distinct records. Differences caused by
transport spans or representation traversal remain real operational costs;
only the source-owned canonical profile and its refinement may define
partition-independent semantic eligibility. This document creates no physical
memory bound or completed performance result.
