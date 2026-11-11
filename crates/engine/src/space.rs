//! The codec space the engine moves through: the portfolio (strategy entries
//! and their weights per family) and the generators that propose a candidate
//! point next to a family's incumbent.

use crate::model::Laws;
use crate::state::{EngineFile, GeneratorRow, StrategyRow, WeightRow};
use uorc::{ncm, performance, portfolio};

/// The generators, in order.
pub const GENERATORS: [&str; 4] = ["drop-context", "add-context", "x86", "promote"];

/// What a generator does, in words.
pub fn describe(generator: &str) -> &'static str {
    match generator {
        "drop-context" => "drops one input of the incumbent's profile",
        "add-context" => "adds one input to the incumbent's profile",
        "x86" => "toggles the x86 filter of the incumbent",
        "promote" => "schedules another portfolio strategy first",
        _ => "proposes a point",
    }
}

/// The portfolio in the engine's hands: entries and weights.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Portfolio {
    /// The entries' 5-byte forms, in portfolio order.
    pub entries: Vec<[u8; portfolio::ENTRY]>,
    /// `weights[family][entry]`.
    pub weights: Vec<Vec<u64>>,
}

/// A point the engine decides: one strategy coding every frame, or a whole
/// portfolio.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Point {
    /// Every frame coded by this entry.
    Fixed([u8; portfolio::ENTRY]),
    /// The portfolio of these byte tables.
    Portfolio {
        /// The strategy entries.
        strategies: Vec<u8>,
        /// The weights.
        weights: Vec<u8>,
    },
}

fn hex_word(word: u32) -> String {
    format!("0x{word:08x}")
}

fn parse_word(s: &str) -> Result<u32, String> {
    u32::from_str_radix(s.trim_start_matches("0x"), 16).map_err(|e| format!("word {s}: {e}"))
}

/// The word of an entry.
pub fn word(entry: &[u8; portfolio::ENTRY]) -> u32 {
    u32::from_le_bytes([entry[0], entry[1], entry[2], entry[3]])
}

/// An entry of a word and a state.
pub fn entry(word: u32, state: u8) -> [u8; portfolio::ENTRY] {
    let w = word.to_le_bytes();
    [w[0], w[1], w[2], w[3], state]
}

/// The entry in words: its profile, filter and state.
pub fn name(e: &[u8; portfolio::ENTRY]) -> String {
    let w = word(e);
    let profile = w & !portfolio::X86_BIT;
    let base = match profile {
        ncm::FULL => "FULL".to_string(),
        portfolio::MID => "MID".to_string(),
        ncm::V03 => "V03".to_string(),
        p => format!("profile 0x{p:05x}"),
    };
    let x86 = if w & portfolio::X86_BIT != 0 {
        " over x86"
    } else {
        ""
    };
    let state = if e[4] != 0 {
        format!(" from state {}", e[4])
    } else {
        String::new()
    };
    format!("{base}{x86}{state}")
}

impl Portfolio {
    /// The portfolio `model/engine.toml` holds.
    pub fn of(file: &EngineFile, laws: &Laws) -> Result<Portfolio, String> {
        let entries = file
            .strategy
            .iter()
            .map(|r| Ok(entry(parse_word(&r.word)?, r.state)))
            .collect::<Result<Vec<_>, String>>()?;
        let mut weights = vec![vec![laws.weight_min; entries.len()]; portfolio::FAMILIES];
        for row in &file.weight {
            let f = family_index(&row.family)?;
            if row.strategy >= entries.len() {
                return Err(format!("weight row names strategy {}", row.strategy));
            }
            if !(laws.weight_min..=laws.weight_max).contains(&row.value) {
                return Err(format!(
                    "weight {} of strategy {} on {} is out of bounds",
                    row.value, row.strategy, row.family
                ));
            }
            weights[f][row.strategy] = row.value;
        }
        Ok(Portfolio { entries, weights })
    }

    /// Write the portfolio into `file`'s rows.
    pub fn store(&self, file: &mut EngineFile) {
        file.strategy = self
            .entries
            .iter()
            .map(|e| StrategyRow {
                word: hex_word(word(e)),
                state: e[4],
            })
            .collect();
        file.weight = (0..portfolio::FAMILIES)
            .flat_map(|f| (0..self.entries.len()).map(move |s| (f, s)))
            .map(|(f, s)| WeightRow {
                strategy: s,
                family: portfolio::FAMILY_NAMES[f].to_string(),
                value: self.weights[f][s],
            })
            .collect();
    }

    /// The strategy bytes the client reads.
    pub fn strategy_bytes(&self) -> Vec<u8> {
        self.entries.iter().flatten().copied().collect()
    }

    /// The weight bytes the client reads.
    pub fn weight_bytes(&self) -> Vec<u8> {
        self.weights
            .iter()
            .flatten()
            .map(|&w| u8::try_from(w).unwrap_or(u8::MAX))
            .collect()
    }

    /// This portfolio as a point.
    pub fn point(&self) -> Point {
        Point::Portfolio {
            strategies: self.strategy_bytes(),
            weights: self.weight_bytes(),
        }
    }

    /// The incumbent of `family`: the first entry its schedule codes that
    /// the coder can apply.
    pub fn incumbent(&self, family: usize) -> Option<usize> {
        let (s, w) = (self.strategy_bytes(), self.weight_bytes());
        let table = portfolio::Table {
            strategies: &s,
            weights: &w,
        };
        table
            .schedule(family)
            .into_iter()
            .find(|&i| table.strategy(i).is_some())
    }

    /// Add `e` (a candidate proved on `family`) with the weight of a winner
    /// over `incumbent`, `weightMin` elsewhere; when the portfolio is full,
    /// the entry consolidated nowhere with the least total weight leaves
    /// first. `None` when every entry is consolidated somewhere.
    pub fn admit(
        &mut self,
        laws: &Laws,
        capacity: usize,
        e: [u8; portfolio::ENTRY],
    ) -> Option<usize> {
        if let Some(i) = self.entries.iter().position(|x| *x == e) {
            return Some(i);
        }
        if self.entries.len() >= capacity {
            let incumbents: Vec<usize> = (0..portfolio::FAMILIES)
                .filter_map(|f| self.incumbent(f))
                .collect();
            let evict = (0..self.entries.len())
                .filter(|&i| !incumbents.contains(&i))
                .filter(|&i| {
                    (0..portfolio::FAMILIES).all(|f| !laws.consolidated(self.weights[f][i]))
                })
                .min_by_key(|&i| {
                    let total: u64 = (0..portfolio::FAMILIES).map(|f| self.weights[f][i]).sum();
                    (total, std::cmp::Reverse(i))
                })?;
            self.entries.remove(evict);
            for row in &mut self.weights {
                row.remove(evict);
            }
        }
        self.entries.push(e);
        for row in &mut self.weights {
            row.push(laws.weight_min);
        }
        Some(self.entries.len() - 1)
    }
}

/// The index of a family name.
pub fn family_index(name: &str) -> Result<usize, String> {
    portfolio::FAMILY_NAMES
        .iter()
        .position(|n| *n == name)
        .ok_or_else(|| format!("no family {name}"))
}

/// The search weight of `generator` on `family`.
pub fn generator_weight(file: &EngineFile, laws: &Laws, generator: &str, family: usize) -> u64 {
    file.generator
        .iter()
        .find(|r| r.generator == generator && r.family == portfolio::FAMILY_NAMES[family])
        .map_or(laws.weight_initial, |r| r.value)
}

/// Set the search weight of `generator` on `family`.
pub fn set_generator_weight(file: &mut EngineFile, generator: &str, family: usize, value: u64) {
    let fam = portfolio::FAMILY_NAMES[family];
    match file
        .generator
        .iter_mut()
        .find(|r| r.generator == generator && r.family == fam)
    {
        Some(r) => r.value = value,
        None => file.generator.push(GeneratorRow {
            generator: generator.to_string(),
            family: fam.to_string(),
            value,
        }),
    }
}

/// The generator a ring state picks for `family`: in proportion to the
/// search weights (each at least `weightMin`, so none is unreachable).
pub fn pick_generator(file: &EngineFile, laws: &Laws, family: usize, state: u64) -> &'static str {
    let weights: Vec<u64> = GENERATORS
        .iter()
        .map(|g| generator_weight(file, laws, g, family))
        .collect();
    let mut r = Laws::select(state, weights.iter().sum());
    for (g, w) in GENERATORS.iter().zip(&weights) {
        if r < *w {
            return g;
        }
        r -= w;
    }
    GENERATORS[0]
}

/// Bits of a profile word an input lives in: the contexts, then the short
/// match, the repeat model and the match-state selection.
const PROFILE_BITS: u32 = 20;

/// The candidate `generator` proposes next to entry `incumbent`, the ring
/// state choosing among its options; `None` when it has none.
pub fn propose(
    generator: &str,
    portfolio: &Portfolio,
    incumbent: usize,
    state: u64,
) -> Option<[u8; portfolio::ENTRY]> {
    let inc = portfolio.entries[incumbent];
    let w = word(&inc);
    let profile = w & !portfolio::X86_BIT;
    let pick = |options: Vec<u32>| -> Option<u32> {
        (!options.is_empty()).then(|| options[Laws::select(state, options.len() as u64) as usize])
    };
    match generator {
        "drop-context" => {
            let set: Vec<u32> = (0..performance::CONTEXT_BITS)
                .filter(|b| profile & (1 << b) != 0)
                .collect();
            // a profile keeps one context at least
            if set.len() < 2 {
                return None;
            }
            pick(set).map(|b| entry(w & !(1 << b), inc[4]))
        }
        "add-context" => {
            let clear: Vec<u32> = (0..PROFILE_BITS)
                .filter(|b| profile & (1 << b) == 0)
                .collect();
            pick(clear).map(|b| entry(w | (1 << b), inc[4]))
        }
        "x86" => Some(entry(w ^ portfolio::X86_BIT, inc[4])),
        "promote" => {
            let others: Vec<u32> = (0..portfolio.entries.len() as u32)
                .filter(|&i| i as usize != incumbent)
                .collect();
            pick(others).map(|i| portfolio.entries[i as usize])
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn laws() -> Laws {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        Laws::load(&root).expect("laws")
    }

    fn shipped() -> Portfolio {
        let entries = (0..portfolio::SHIPPED.count())
            .map(|i| {
                portfolio::SHIPPED.strategies[i * 5..i * 5 + 5]
                    .try_into()
                    .expect("entry")
            })
            .collect::<Vec<[u8; 5]>>();
        let n = entries.len();
        Portfolio {
            entries,
            weights: (0..portfolio::FAMILIES)
                .map(|f| {
                    (0..n)
                        .map(|s| u64::from(portfolio::SHIPPED.weight(f, s)))
                        .collect()
                })
                .collect(),
        }
    }

    #[test]
    fn every_proposal_is_a_valid_entry() {
        let p = shipped();
        for g in GENERATORS {
            for inc in 0..p.entries.len() {
                for state in 0..64u64 {
                    if let Some(e) = propose(g, &p, inc, state) {
                        assert!(ncm::valid_profile(word(&e) & !portfolio::X86_BIT), "{g}");
                        if g != "promote" {
                            assert_ne!(e, p.entries[inc]);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn a_full_portfolio_keeps_its_consolidated_entries() {
        let l = laws();
        let mut p = shipped();
        let n = p.entries.len();
        let new = entry(ncm::FULL & !1, 0);
        assert_eq!(p.admit(&l, n, new), Some(n - 1));
        assert_eq!(p.entries.len(), n);
        assert!(p.entries.contains(&new));
        for f in 0..portfolio::FAMILIES {
            assert!(p.incumbent(f).is_some());
        }
        // every entry of the shipped portfolio consolidated somewhere stays
        for (i, e) in shipped().entries.iter().enumerate() {
            if (0..portfolio::FAMILIES).any(|f| l.consolidated(shipped().weights[f][i])) {
                assert!(p.entries.contains(e), "{}", name(e));
            }
        }
    }

    #[test]
    fn generators_are_picked_in_proportion_to_their_weights() {
        let l = laws();
        let mut file = EngineFile {
            spec: "uorc/engine/1".into(),
            tick: 0,
            rule30_state: "0x1".into(),
            strategy: vec![],
            weight: vec![],
            generator: vec![],
            ticks: vec![],
        };
        for g in GENERATORS {
            set_generator_weight(&mut file, g, 3, l.weight_min);
        }
        set_generator_weight(&mut file, "x86", 3, l.weight_max);
        let total = 3 * l.weight_min + l.weight_max;
        let hits = (0..total)
            .filter(|&s| pick_generator(&file, &l, 3, s) == "x86")
            .count() as u64;
        assert_eq!(hits, l.weight_max);
    }
}
