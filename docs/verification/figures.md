<!-- uorc:non-authoritative -->
# Numbers in what the release ships (UC-FIG-01)

`Uorc.Performance::implementationFigures` is the authority, and `crates/figures`
is the check. This file records the evidence the gate produces and the planted
defects that show it can fail.

## Gate

`just figures` runs `figures check`. It runs as the factory obligation
`figures-ubuntu-24.04`, keyed on its own inputs, and takes a few seconds. It
reads every numeral of three kinds of source:

- the shipped crates' `src/`, which are the crates the release obligations build;
- the page's JavaScript, under the directory inputs of the obligation that
  builds the site;
- the definitions of the model the client is exported from, which is the
  import closure of `Uorc.Client`, less the modules an imported standard judges
  (`Uorc.Integrity.*`).

It compares them with `model/figures.toml`. The register was seeded on main
`e713d1c` (v0.6.0). It held 8,252 sites:

| Source | Sites |
| --- | --- |
| crates/uorc, crates/uorc-wasm | 1,116 |
| web/*.js | 60 |
| model (Uorc.Client closure) | 7,076 |

## Planted defects

Each defect was planted in the obligation's closure copy
(`node scripts/factory.mjs materialize figures-ubuntu-24.04`), and the copy was
restored afterwards.

| Defect | Command | Diagnostic | Restored |
| --- | --- | --- | --- |
| `pub const LINE: usize = 64;` added to `crates/uorc/src/lib.rs` | `figures check` | `crates/uorc/src/lib.rs: 1 site(s) of 64 where the register admits 0 (in LINE)` | passes |
| `const WORKERS = 4;` added to `web/uorc.js` | `figures check` | `web/uorc.js: 1 site(s) of 4 where the register admits 17 (in WORKERS, …)` | passes |
| a definition `lanes` multiplying by `8` added to `Uorc.Codec.Table` | `figures check` | `client/src/Uorc/Codec/Table.lex.tex: 1 site(s) of 8 where the register admits 2 (in byteOf, lanes)` | passes |
| `figures write` after an added site | `figures write` | refuses: the writer never raises a count | n/a |
| `figures seed` with a register present | `figures seed` | refuses: a register is never reseeded | n/a |

`cargo test -p figures` holds the same cases on fixtures (rejection, a stale
register, never reseeding, never raising a count, tags checked against the
model, the page and model scopes, and the standards exclusion). On the
repository, `crates/figures/tests/repository.rs` plants a figure in each kind of
source.
