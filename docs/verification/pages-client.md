<!-- uorc:non-authoritative -->
# Pages client against its product model (UC-WEB-01)

`src/Uorc/Page.lex.tex` is the authority. This file records what was found on
the v0.3 page, the evidence the gate produces and the planted defects that show
each gate can fail.

## The v0.3 page, driven in headless Chromium

Measured on main `a4e2222` plus PR #129, with real files and CLI-written archives.

| Control | Result |
| --- | --- |
| Verify, with or without an original | WebAssembly trap (`unreachable`) |
| Every example button | trap: each example decompresses in memory |
| Decompress of a folder or multi-file archive | trap |
| Any action after a trap | trap: the module's state stays poisoned until reload |
| Compress of an 18 MB file | correct bytes, 90 s with the page frozen between 8 MiB frames, no cancel |
| A file and a folder both chosen | the file is silently ignored |
| "Extra program-search steps" | no effect a visitor can see |

Cause of the traps: in-memory decoding ran `decode_stream`, which always spawned
worker threads, and `wasm32-unknown-unknown` has none. One core now decodes in
stream order with no thread (`threads <= 1`, which is what a WebAssembly host
reports), the same schedule `FrameDecoder` runs.

## Gates

- `cargo xtask check-model`: the model loads, passes the Prism checks (PP2014 to
  PP2017 for features and journeys, one initial state, closed triggers, every
  state reachable, closed operations, controls, field kinds, error codes and
  tables, links to GitHub or the site only, measures naming requirements) and
  `web/index.html` equals its rendering.
- `planted_model_defects_are_rejected_uc_web_01`: eleven planted model defects,
  each rejected by its check.
- `scripts/page-acceptance.mjs` (CI: `pages-client.yml`; local: `just pages`):
  walks the three journeys in fresh pages, runs the 18 acceptance cases, records
  every state change and checks every control against `enabledIn` at each one,
  and holds the gated measures.

## Planted defects in the browser gate

| Plant | Command | Diagnostic |
| --- | --- | --- |
| Cancel enabled in `done` (built `app.js` edited) | `node scripts/page-acceptance.mjs <site>` | `done: cancel is enabled` |
| The v0.3 wasm (threaded decode) beside the new page | same | `decompress folder: The browser stopped the operation.` and `example text: The browser stopped the operation.` |

Restored: the unmodified site passes, `17 cases, 11 transitions, 8 measures`.

The acceptance also requires every `uorc.*` call a feature names to be defined
in the shipped `uorc.js`.

## Folders stream like files

Folders and multi-file archives go through the frame operations
(`compressTreeBlob`, `decompressTreeBlob`). The page reads each file a frame at
a time and splits each decoded frame at its object lengths.
`tree_exports_stream_the_library_tree` (uorc-wasm) checks that a tree pushed a
piece at a time writes the library's `compress_tree` bytes and restores the same
entries. In Node, a folder holding a 24 MiB file round-trips with 217 MiB of
WebAssembly memory to compress and 189 MiB to decompress.

Planting the v0.3 wasm first showed a page defect: an async trap in the worker
raised no `error` event, so the page stayed in `running`. The worker now reports
any non-uorc failure as `stopped` and closes, and the page starts a new one.

## The page uses the visitor's cores

The page's worker plans each frame with the library (`FrameEncoder::plan`), and
a pool of workers codes the frame's two portfolio strategies side by side, then
the library's own rule (`uorc::choose`) keeps one. The pool holds one worker per
core, at most 16 and two per GiB of device memory; every worker shares one
compiled module. Bytes equal the library's: the acceptance compares every page
archive against `uorc::compress` (differingArchives 0), and the wasm test
`planned_frames_coded_by_strategy_are_the_library_stream` checks the same path
in Node.

Progress moves within a frame: ncm reports every 16 KiB it codes, so the bar
never stands still for long on an 8 MiB frame (measure `progressGap`). The gate
found two stalls of over 2 s, both fixed: the first file waited behind the pool
workers' start (the page's worker now codes alone until two have loaded), and a
short last frame, coded when the stream ends, reported nothing.

Same 10 MiB of generated prose, 4 vCPU, headless Chromium:

| | Compress | Decompress |
| --- | --- | --- |
| One worker (main before the pool) | 168 KB/s (62.3 s) | 563 KB/s (18.6 s) |
| Worker pool | 369 KB/s (28.4 s) | 561 KB/s (18.7 s) |

Decoding this input stays on one worker, as it does natively: every frame is a
window frame whose model reads the two frames before it. The archive is the
same 5,731 bytes in both rows.

## Measured (4 vCPU cloud VM, headless Chromium, SIMD128 build, ncm v4)

| Measure | Value | Gate |
| --- | --- | --- |
| Cores the page may use without a pool worker | 0 | 0 |
| Longest time the progress bar stands still | 552 ms | ≤ 2000 ms |
| Longest page task while an operation runs | 0 ms | ≤ 200 ms |
| Cancel to ready | 155 ms | ≤ 3000 ms |
| Navigation to ready | 340 ms | ≤ 5000 ms |
| Page archives differing from the library's | 0 | 0 |
| WebAssembly memory, largest worker | 227 MiB | ≤ 256 MiB |
| Controls without an accessible name | 0 | 0 |
| Compress, the acceptance's 10 MiB of prose | 376 KB/s | measured |
| Decompress, the acceptance's 10 MiB of prose | 875 KB/s | measured |

Decode memory is bounded. Measured in Node (same V8, SIMD128 build) on streams
of distinct 8 MiB frames: 226 MiB for 3 frames and 250 MiB for 6 or 12, flat once
the two-frame window is full (241, 257 and 257 MiB before). Two changes, same
bytes: ncm's model reads the window's frames in place instead of a joined copy,
and in WebAssembly, whose memory never shrinks, a model's tables (about 100 MiB
for an 8 MiB frame) are kept for the next frame's model instead of being
allocated again, which had left the allocator's free space in pieces. The 64 MiB
stack the exported reference code needs is part of every figure.

Node (same V8) on 2 MB of repository text: 0.21 MB/s both ways without SIMD128,
0.29 MB/s with it, against 0.53 MB/s for one native core. Random input
compresses at 0.18 MB/s before being stored raw.
