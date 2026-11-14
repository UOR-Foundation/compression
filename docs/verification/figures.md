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

## The ratchet

The register only falls, and it falls as figures are derived, never as they are
renamed. A literal that moves into a named constant is still the same fixed
figure. The ratchet works the way the rule 30 engine and verdant-resonance's
shrink-only ratchet do:

- **Shrink-only.** The check rejects any site the register does not list, and
  `figures write` never raises a count (verdant-resonance's `--ratchet-down`).
- **Release.** `Uorc.Performance::parametricRelease` names the first release
  that ships only a parametric implementation (0.7.0). From that workspace
  version on, the check rejects the tree while any site is listed, so the
  version bump that tags the release cannot land before the distance is 0.
- **Derivations.** Each class of figure is retired by the abstraction that
  derives it, applied by the writing aid for the whole class at once:

| Class | Derived from | Status | Sites |
| --- | --- | --- | --- |
| global addresses of the NCM models' state (`getAt w glob a`) | the layout: a global's address is the one before it plus that global's width | derived (`client/gen/irlex.py`) | 8,252 → 6,166 |
| global widths | the field's representation; one site per global | open | 64 per model |
| table bases and element widths in `mem` | the table extents | open | about 100 |
| byte-to-`Bytes` comparison trees (`singletonSequence`, `byteSlice`, `natByteSlice`) | the identity map of the byte ring | open, kernel modules | 763 |
| codec arithmetic (shifts, scales, masks, rates, limits) | coordinates decided by outcome (below) | open | most of the rest |

**Coordinates by outcome.** A codec figure that is not structural is a
coordinate. The engine decides it over the κ-named corpus, tick by tick, by
galloping search from the identity (1, 2, 4, …, then bisection), bounded by
the declared host. Each decision is recorded by its κ and never rerun.
There are two outcomes:

- If the best value is the same on every family, that value is a decided
  figure, admitted with its evidence row.
- If the best value differs by family, a single value is refused. The
  coordinate becomes a frame-word facet stepped per frame (as the
  consolidation and delta facets are). Until then it stays open, with its
  measured distance: the bytes the best single value loses against the
  per-family best.

Frames of earlier eras keep the values their archives were written with.
Those values are format identities of the era, stated in the wire model.

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
