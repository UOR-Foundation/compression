<!-- uorc:non-authoritative -->
# Pages client against its product model (UC-WEB-01, UC-WEB-02)

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

## Measured (4 vCPU cloud VM, headless Chromium, SIMD128 build)

| Measure | Value | Gate |
| --- | --- | --- |
| Longest page task while an operation runs | 0 ms (4 long tasks seen outside operations) | ≤ 200 ms |
| Cancel to ready | 112 ms | ≤ 3000 ms |
| Navigation to ready | 225 ms | ≤ 5000 ms |
| Page archives differing from the library's | 0 | 0 |
| WebAssembly memory after a 3-frame (24 MiB) stream | 179 MiB | ≤ 256 MiB |
| Controls without an accessible name | 0 | 0 |
| Compress, generated prose | 409 KB/s | measured |
| Decompress, generated prose | 439 KB/s | measured |

Node (same V8) on 2 MB of repository text: 0.21 MB/s both ways without SIMD128,
0.29 MB/s with it, against 0.53 MB/s for one native core. Random input
compresses at 0.18 MB/s before being stored raw.

## v0.5: the codec point, the claims and the engine (UC-WEB-02)

The v0.5 Prism model states the codec as a point of a modelled space and
records claims about it (`Uorc.Performance`, `frontier.toml`,
`model/engine.toml`). The product now shows that model wherever it shows an
archive:

- `uorc::Info::to_json` (and so `uorc inspect --json`, the wasm `inspect` and
  the page) gives each frame's `state`, `lanes`, `steps` and `codec_point`
  from `uorc::performance`: the step count is exact and read from the header
  alone (raw: one per byte, reference: none, ncm: eight bits times the inputs
  the profile word keeps, per byte), and the codec point κ is SHA-256 over
  `uorc/codec-point/1`, the basis, the profile word, the state κ and the lane
  count. `uorc inspect` prints the same lines in text.
- The page's inspect table adds the profile word, model state, lanes, steps,
  codec point κ and the recorded claims that name that point.
- `scripts/site.mjs` publishes `model.json` beside the closure (as it does the
  bench page): the ledger's `UC-PERF-*` rows, the claims of `frontier.toml`
  and the engine state of `model/engine.toml`. Its reader refuses any TOML it
  cannot read faithfully, so a register the page would misreport fails the
  build. The page's model view (`readModel`) shows the three tables and the
  engine line; the release attaches `model.json` and puts its summary in the
  release notes.
- The factory obligation `pages` keys on the three model files, so a new
  claim or engine tick re-runs the browser acceptance and nothing else.

Gates:

- `inspect_names_each_frames_codec_point_uc_web_02` (crates/uorc/tests/cli.rs):
  for an ncm and a raw frame, the JSON and text inspect equal the library's
  `CodecPoint::of(..).kappa()` and `frame_steps`.
- `the_model_view_holds_every_model_table_uc_web_02` (crates/model): the page
  holds each model table with the model's columns and the inspect columns.
- `model.json projects the theorems, claims and engine state`
  (scripts/site.test.mjs): the projection of fixture files, integers past
  2^53 kept exact as strings, unreadable TOML refused.
- Browser: `inspect-codec-point` compares every frame of a three-frame stream
  with the library's inspect in the same page, against a register that claims
  the first frame's point (served under `/claimed/`); `model-theorems`,
  `model-claims` and `model-engine` compare the model view with the published
  `model.json`. Gated measures `differingCodecPoints` and
  `differingModelValues`, both 0.

Local run (4 vCPU, headless Chromium): `22 cases, 11 transitions, 10 measures`,
both new measures 0.

| Plant (built `app.js` edited) | Diagnostic |
| --- | --- |
| The inspect table reports no claims for any frame | `judgeFormat: inspect rows [...] differ from the library [...]` |
| The model view's empty-table note dropped | `judgeFormat: claims rows [[""]] differ from model.json [["None recorded in this release."]]` |

Restored: the unmodified site passes. Measured in that run: an 8 MiB ncm frame
under profile word 0x7e1ff costs 1,342,177,280 steps (160 per byte), and each
reference frame of the stream costs 0.
