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
  walks the three journeys in fresh pages, runs the 17 acceptance cases, records
  every state change and checks every control against `enabledIn` at each one,
  and holds the gated measures.

## Planted defects in the browser gate

| Plant | Command | Diagnostic |
| --- | --- | --- |
| Cancel enabled in `done` (built `app.js` edited) | `node scripts/page-acceptance.mjs <site>` | `done: cancel is enabled` |
| The v0.3 wasm (threaded decode) beside the new page | same | `decompress folder: The browser stopped the operation.` and `example text: The browser stopped the operation.` |

Restored: the unmodified site passes, `17 cases, 11 transitions, 8 measures`.

Planting the v0.3 wasm first showed a page defect: an async trap in the worker
raised no `error` event, so the page stayed in `running`. The worker now reports
any non-uorc failure as `stopped` and closes, and the page starts a new one.

## Measured (4 vCPU cloud VM, headless Chromium, SIMD128 build)

| Measure | Value | Gate |
| --- | --- | --- |
| Longest page task while an operation runs | 0 ms (4 long tasks seen outside operations) | ≤ 200 ms |
| Cancel to ready | 112 ms | ≤ 3000 ms |
| Navigation to ready | 225 ms | ≤ 5000 ms |
| Page archives differing from the library's | 0 | 0 |
| WebAssembly memory after a 3-frame (24 MiB) stream | 154 MiB | ≤ 256 MiB |
| Controls without an accessible name | 0 | 0 |
| Compress, generated prose | 409 KB/s | measured |
| Decompress, generated prose | 439 KB/s | measured |

Node (same V8) on 2 MB of repository text: 0.21 MB/s both ways without SIMD128,
0.29 MB/s with it, against 0.53 MB/s for one native core. Random input
compresses at 0.18 MB/s before being stored raw.
