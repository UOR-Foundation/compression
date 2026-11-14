//! The coordinate stage: each tick moves the search for the codec's
//! coordinates ([`uorc::coordinates`]) by outcome over the κ-named corpus.
//!
//! For each coordinate the search gallops from its least value (the
//! identity: 0 for an additive figure, 1 for a divisor) by doubling, past
//! the standard value and on while the corpus gets strictly smaller, then
//! bisects the bracket around the best value until no integer is left
//! unmeasured beside it. The standard value is one of the measured points,
//! so a decided value never codes the corpus larger than it. Each value measured
//! is a closed claim in `frontier.toml` under its κ, so a later tick walks
//! the same path free and only measures what is new; at most the model's
//! `hypothesisBudget` new values are measured per tick.
//!
//! A finished search decides the coordinate:
//!
//! - **decided**: every family of the corpus codes smallest at the value the
//!   whole corpus does. That value is the coordinate's decided figure, with
//!   the claims as its evidence.
//! - **per-input**: some family codes smallest at another value. No single
//!   figure is right, so the coordinate must become a frame-word facet,
//!   stepped per frame by outcome. Its distance is the bytes the best
//!   single value loses against each family's own best.
//!
//! The decisions are written to `model/coordinates.toml`
//! (`uorc/coordinates/1`).

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use uorc::coordinates::Coordinate;
use uorc::portfolio;

use crate::decide::FrontierDecider;
use crate::tick::Measure;

/// The decisions file, relative to the repository root.
pub const COORDINATES: &str = "model/coordinates.toml";
/// Its schema.
pub const SCHEMA: &str = "uorc/coordinates/1";

/// One coordinate's search and decision.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Row {
    /// The coordinate's register name.
    pub name: String,
    /// `searching`, `decided` or `per-input`.
    pub status: String,
    /// The value the coder codes at today.
    pub standard: i64,
    /// Corpus bytes at the standard value.
    pub standard_bytes: u64,
    /// The value the whole corpus codes smallest at, among those measured.
    pub best: i64,
    /// Corpus bytes at `best`.
    pub best_bytes: u64,
    /// Each family's best value among those measured (families with members).
    pub family_best: BTreeMap<String, i64>,
    /// `per-input`: bytes `best` loses against each family's own best.
    pub distance: u64,
    /// Values measured, in the order the search measured them.
    pub measured: Vec<i64>,
}

/// The decisions file.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct CoordinatesFile {
    /// [`SCHEMA`].
    pub schema: String,
    /// κ of the corpus the decisions are over.
    pub corpus: String,
    /// One row per coordinate.
    #[serde(default, rename = "coordinate")]
    pub rows: Vec<Row>,
}

impl CoordinatesFile {
    /// Write the file.
    pub fn save(&self, root: &Path) -> Result<(), String> {
        let text = toml::to_string(self).map_err(|e| e.to_string())?;
        let head = "# The codec's coordinates, searched by outcome over a κ-named corpus (written by\n\
                    # `uorc-engine tick`; see crates/engine/src/coordinates.rs). Every value measured is\n\
                    # a closed claim in frontier.toml under its κ.\n\n";
        std::fs::write(root.join(COORDINATES), format!("{head}{text}"))
            .map_err(|e| format!("{COORDINATES}: {e}"))
    }
}

/// A measurement the search asks for: `Ok(None)` when it is new and the
/// tick's budget is spent.
pub trait Probe {
    /// Corpus measure at `values` (empty: the standard coder).
    fn probe(&mut self, values: &BTreeMap<String, i64>) -> Result<Option<Measure>, String>;
}

/// The decider as a probe of the shipped client, with a budget of new runs.
pub struct Budgeted<'a> {
    /// The decider.
    pub decider: &'a mut FrontierDecider,
    /// New runs left this tick.
    pub left: u64,
    /// The repository root the register is written back to after each new
    /// run, so a tick cut short keeps every claim it decided.
    pub root: &'a Path,
}

impl Probe for Budgeted<'_> {
    fn probe(&mut self, values: &BTreeMap<String, i64>) -> Result<Option<Measure>, String> {
        let codec = self.decider.shipped_at(values);
        if self.decider.settled(&codec) {
            return self.decider.measure_codec(&codec).map(Some);
        }
        if self.left == 0 {
            return Ok(None);
        }
        self.left -= 1;
        let m = self.decider.measure_codec(&codec)?;
        self.decider.save(self.root)?;
        Ok(Some(m))
    }
}

fn at(c: Coordinate, v: i64) -> BTreeMap<String, i64> {
    BTreeMap::from([(c.name().to_owned(), v)])
}

/// Search one coordinate as far as the budget reaches.
pub fn search(
    c: Coordinate,
    members: [usize; portfolio::FAMILIES],
    probe: &mut dyn Probe,
) -> Result<Option<Row>, String> {
    let Some(standard) = probe.probe(&BTreeMap::new())? else {
        return Ok(None);
    };
    // the standard value is a measured point of the search like any other
    let mut seen: Vec<(i64, Measure)> = vec![(c.standard(), standard.clone())];
    let mut complete = true;
    let mut measure = |v: i64, seen: &mut Vec<(i64, Measure)>| -> Result<Option<u64>, String> {
        if let Some((_, m)) = seen.iter().find(|(x, _)| *x == v) {
            return Ok(Some(m.bytes));
        }
        Ok(probe.probe(&at(c, v))?.map(|m| {
            let b = m.bytes;
            seen.push((v, m));
            b
        }))
    };
    // gallop from the identity while the corpus gets strictly smaller
    let mut path: Vec<(i64, u64)> = Vec::new();
    let mut v = c.least();
    loop {
        let Some(b) = measure(v, &mut seen)? else {
            complete = false;
            break;
        };
        // past the standard value, the first value that does not improve on
        // the one before it ends the gallop
        let worse = v > c.standard() && path.last().is_some_and(|&(_, p)| b >= p);
        path.push((v, b));
        if worse {
            break;
        }
        // the coordinate's domain ends the gallop at its greatest value
        let greatest = c.greatest(Coordinate::standard);
        if v >= greatest {
            break;
        }
        let Some(next) = (if v == 0 { Some(1) } else { v.checked_mul(2) }) else {
            break;
        };
        v = next.min(greatest);
    }
    if complete {
        // bisect the bracket around the best value
        let i = (0..path.len())
            .min_by_key(|&i| (path[i].1, path[i].0))
            .unwrap_or(0);
        let mut lo = if i > 0 { path[i - 1].0 } else { path[i].0 };
        let mut hi = path.get(i + 1).map_or(path[i].0, |p| p.0);
        let (mut best, mut best_bytes) = path[i];
        loop {
            let left = best - lo;
            let right = hi - best;
            if left <= 1 && right <= 1 {
                break;
            }
            let m = if left >= right {
                lo + left / 2
            } else {
                best + right / 2
            };
            let Some(b) = measure(m, &mut seen)? else {
                complete = false;
                break;
            };
            if b < best_bytes || (b == best_bytes && m < best) {
                if m < best {
                    hi = best;
                } else {
                    lo = best;
                }
                best = m;
                best_bytes = b;
            } else if m < best {
                lo = m;
            } else {
                hi = m;
            }
        }
    }
    // nothing measured yet: the budget ran out before this coordinate
    let Some((best, best_m)) = seen
        .iter()
        .min_by_key(|(v, m)| (m.bytes, *v))
        .map(|(v, m)| (*v, m.clone()))
    else {
        return Ok(None);
    };
    let mut family_best = BTreeMap::new();
    let mut distance = 0;
    for (f, name) in portfolio::FAMILY_NAMES.iter().enumerate() {
        if members[f] == 0 {
            continue;
        }
        let (fv, fm) = seen
            .iter()
            .min_by_key(|(v, m)| (m.family[f].0, *v))
            .map(|(v, m)| (*v, m.family[f].0))
            .ok_or("the search measured nothing")?;
        family_best.insert((*name).to_owned(), fv);
        distance += best_m.family[f].0 - fm;
    }
    let status = if !complete {
        "searching"
    } else if family_best.values().all(|&v| v == best) {
        "decided"
    } else {
        "per-input"
    };
    Ok(Some(Row {
        name: c.name().to_owned(),
        status: status.to_owned(),
        standard: c.standard(),
        standard_bytes: standard.bytes,
        best,
        best_bytes: best_m.bytes,
        family_best,
        distance: if status == "per-input" { distance } else { 0 },
        measured: seen.iter().map(|(v, _)| *v).collect(),
    }))
}

/// The stage: search every coordinate within `budget` new runs and write
/// the decisions.
pub fn stage(
    root: &Path,
    decider: &mut FrontierDecider,
    budget: u64,
) -> Result<CoordinatesFile, String> {
    use crate::tick::Decider;
    let members = decider.members();
    let corpus = decider.corpus();
    let mut probe = Budgeted {
        decider,
        left: budget,
        root,
    };
    let mut file = CoordinatesFile {
        schema: SCHEMA.to_owned(),
        corpus,
        rows: Vec::new(),
    };
    for &c in Coordinate::ALL {
        if let Some(row) = search(c, members, &mut probe)? {
            file.rows.push(row);
        }
    }
    file.save(root)?;
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A corpus whose bytes are `(v - 300)^2` on one family and
    /// `(v - target)^2` on another.
    struct Bowl {
        target: i64,
        runs: u64,
    }

    impl Probe for Bowl {
        fn probe(&mut self, values: &BTreeMap<String, i64>) -> Result<Option<Measure>, String> {
            self.runs += 1;
            let v = values.values().next().copied().unwrap_or(4096);
            let a = ((v - 300) * (v - 300)) as u64;
            let b = ((v - self.target) * (v - self.target)) as u64;
            let mut m = Measure {
                bytes: a + b,
                ..Measure::default()
            };
            m.family[0] = (a, 0);
            m.family[1] = (b, 0);
            Ok(Some(m))
        }
    }

    fn members() -> [usize; portfolio::FAMILIES] {
        let mut m = [0; portfolio::FAMILIES];
        m[0] = 1;
        m[1] = 1;
        m
    }

    #[test]
    fn a_shared_optimum_is_decided_and_found_exactly() {
        let mut p = Bowl {
            target: 300,
            runs: 0,
        };
        let row = search(Coordinate::PlasticityHalfLife, members(), &mut p)
            .unwrap()
            .unwrap();
        assert_eq!(row.status, "decided");
        assert_eq!(row.best, 300);
        assert_eq!(row.best_bytes, 0);
        assert!(p.runs < 40, "{} runs", p.runs);
    }

    #[test]
    fn a_split_optimum_is_per_input_with_its_distance() {
        let mut p = Bowl {
            target: 900,
            runs: 0,
        };
        let row = search(Coordinate::PlasticityHalfLife, members(), &mut p)
            .unwrap()
            .unwrap();
        assert_eq!(row.status, "per-input");
        assert_eq!(row.best, 600);
        assert!(row.distance > 0);
        assert_ne!(row.family_best["tiny"], row.family_best["small"]);
    }
}
