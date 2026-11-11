//! The engine's laws, read from the Prism model `Uorc.Engine`
//! (`src/Uorc/Engine.lex.tex`). The model is the single source of every
//! constant here and of the transitions: the functions below compute exactly
//! the model's definitions, and the tests hold them to the model's
//! kernel-checked closed vectors.

use std::collections::BTreeMap;
use std::path::Path;

/// The model module, relative to the repository root.
pub const MODULE: &str = "src/Uorc/Engine.lex.tex";

/// The constants of `Uorc.Engine`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Laws {
    /// The least weight (`weightMin`).
    pub weight_min: u64,
    /// The greatest weight (`weightMax`).
    pub weight_max: u64,
    /// The weight of a pair nothing has been learned about (`weightInitial`).
    pub weight_initial: u64,
    /// What a success adds (`potentiation`).
    pub potentiation: u64,
    /// What a failure takes (`depression`).
    pub depression: u64,
    /// The weight at which a pair is consolidated (`consolidation`).
    pub consolidation: u64,
    /// Hypotheses one tick may mint (`hypothesisBudget`).
    pub hypothesis_budget: u64,
    /// Strategies the portfolio holds at most (`portfolioCapacity`).
    pub portfolio_capacity: u64,
    /// Ticks the engine keeps a record of (`tickHistory`).
    pub tick_history: u64,
    /// Cells of the rule 30 ring (`cells`).
    pub cells: u64,
    /// The ring's state after a zero state (`seed`).
    pub seed: u64,
    /// The first minute of the hour the workflow ticks at (`cronFirst`).
    pub cron_first: u64,
    /// The second minute (`cronSecond`).
    pub cron_second: u64,
    /// Minutes the workflow allows its setup (`setupMinutes`).
    pub setup_minutes: u64,
    /// Minutes one tick may decide for (`tickMinutes`).
    pub tick_minutes: u64,
    /// The workflow job's timeout (`timeoutMinutes`).
    pub timeout_minutes: u64,
}

/// The value of each `field: value` pair of a declaration, in either the
/// one-line or the multi-line canonical layout.
fn field<'a>(decl: &'a str, key: &str) -> Option<&'a str> {
    let at = decl.find(&format!("{key}: "))? + key.len() + 2;
    let rest = &decl[at..];
    let end = rest.find([',', '}', '\n']).unwrap_or(rest.len());
    Some(rest[..end].trim())
}

/// Every parameterless `Nat` definition whose body is a literal.
pub fn constants(source: &str) -> BTreeMap<String, u64> {
    let mut out = BTreeMap::new();
    for decl in source.split("definition{").skip(1) {
        let decl = decl.split("\n\n").next().unwrap_or(decl);
        let (Some(name), Some(result), Some(body)) = (
            field(decl, "name"),
            field(decl, "result"),
            field(decl, "body"),
        ) else {
            continue;
        };
        if decl.contains("parameters: [],") && result == "Nat" {
            if let Ok(v) = body.parse() {
                out.insert(name.to_string(), v);
            }
        }
    }
    out
}

/// The names of the module's theorems.
pub fn theorems(source: &str) -> Vec<String> {
    source
        .split("theorem{")
        .skip(1)
        .filter_map(|decl| field(decl, "name").map(str::to_string))
        .collect()
}

/// The closed vectors `rule30Step a = b` the module proves.
pub fn rule30_vectors(source: &str) -> Vec<(u64, u64)> {
    source
        .split("theorem{")
        .skip(1)
        .filter_map(|decl| {
            let statement = field(decl, "statement")?;
            let inner = statement.strip_prefix("((rule30Step ")?;
            let (arg, rest) = inner.split_once(") = ")?;
            let arg = arg.trim_matches(|c| c == '(' || c == ')');
            let arg = if arg == "seed" {
                None
            } else {
                arg.parse().ok()
            };
            let value = rest.trim_end_matches(')').parse().ok()?;
            Some((arg.unwrap_or(u64::MAX), value))
        })
        .collect()
}

impl Laws {
    /// The laws stated by the module text `source`.
    pub fn parse(source: &str) -> Result<Laws, String> {
        let c = constants(source);
        let get = |name: &str| {
            c.get(name)
                .copied()
                .ok_or_else(|| format!("{MODULE}: no constant {name}"))
        };
        Ok(Laws {
            weight_min: get("weightMin")?,
            weight_max: get("weightMax")?,
            weight_initial: get("weightInitial")?,
            potentiation: get("potentiation")?,
            depression: get("depression")?,
            consolidation: get("consolidation")?,
            hypothesis_budget: get("hypothesisBudget")?,
            portfolio_capacity: get("portfolioCapacity")?,
            tick_history: get("tickHistory")?,
            cells: get("cells")?,
            seed: get("seed")?,
            cron_first: get("cronFirst")?,
            cron_second: get("cronSecond")?,
            setup_minutes: get("setupMinutes")?,
            tick_minutes: get("tickMinutes")?,
            timeout_minutes: get("timeoutMinutes")?,
        })
    }

    /// The laws of the repository at `root`.
    pub fn load(root: &Path) -> Result<Laws, String> {
        let path = root.join(MODULE);
        let source =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        Laws::parse(&source)
    }

    /// `rule30Step`: one step of rule 30 on the 64-cell ring, a cell's next
    /// value `left xor (centre or right)` with left the next higher cell; a
    /// zero ring restarts from the seed.
    pub fn rule30_step(&self, state: u64) -> u64 {
        let s = if state == 0 { self.seed } else { state };
        s.rotate_right(1) ^ (s | s.rotate_left(1))
    }

    /// `select`: the choice of one of `n` options a ring state makes.
    pub fn select(state: u64, n: u64) -> u64 {
        if n == 0 {
            0
        } else {
            state % n
        }
    }

    /// `potentiate`: a success adds `potentiation` up to `weightMax`.
    pub fn potentiate(&self, w: u64) -> u64 {
        (w + self.potentiation).min(self.weight_max)
    }

    /// `depress`: a failure takes `depression` down to `weightMin`.
    pub fn depress(&self, w: u64) -> u64 {
        if w <= self.weight_min + self.depression {
            self.weight_min
        } else {
            w - self.depression
        }
    }

    /// `learn`.
    pub fn learn(&self, w: u64, success: bool) -> u64 {
        if success {
            self.potentiate(w)
        } else {
            self.depress(w)
        }
    }

    /// `winnerWeight`: a proved winner takes the larger of its weight and
    /// the incumbent's, potentiated.
    pub fn winner_weight(&self, candidate: u64, incumbent: u64) -> u64 {
        self.potentiate(candidate.max(incumbent))
    }

    /// `loserWeight`: the incumbent it beat is depressed.
    pub fn loser_weight(&self, incumbent: u64) -> u64 {
        self.depress(incumbent)
    }

    /// `consolidated`.
    pub fn consolidated(&self, w: u64) -> bool {
        self.consolidation <= w
    }

    /// `dominates`: no more bytes and no more steps, and fewer of one.
    pub fn dominates(candidate: (u64, u64), incumbent: (u64, u64)) -> bool {
        candidate.0 <= incumbent.0
            && candidate.1 <= incumbent.1
            && (candidate.0 < incumbent.0 || candidate.1 < incumbent.1)
    }

    /// `validWorkflow` of the shipped schedule and budgets.
    pub fn valid_workflow(&self) -> bool {
        0 < self.cron_first
            && self.cron_first < 30
            && self.cron_second == self.cron_first + 30
            && self.setup_minutes + self.tick_minutes <= self.timeout_minutes
    }
}

/// `admitMint`: a hypothesis whose κ is known is never minted again, and a
/// tick mints at most its budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MintVerdict {
    /// Minted; the budget left.
    Minted(u64),
    /// Its κ is already decided.
    RefusedKnown,
    /// The tick's budget is spent.
    RefusedBudget,
}

/// `admitMint`.
pub fn admit_mint(known: bool, remaining: u64) -> MintVerdict {
    if known {
        MintVerdict::RefusedKnown
    } else if remaining == 0 {
        MintVerdict::RefusedBudget
    } else {
        MintVerdict::Minted(remaining - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> String {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        std::fs::read_to_string(root.join(MODULE)).expect("model")
    }

    #[test]
    fn the_native_rule30_is_the_models() {
        let s = source();
        let laws = Laws::parse(&s).expect("laws");
        let vectors = rule30_vectors(&s);
        assert!(vectors.len() >= 3, "{vectors:?}");
        for (arg, value) in vectors {
            let arg = if arg == u64::MAX { laws.seed } else { arg };
            assert_eq!(laws.rule30_step(arg), value, "rule30Step {arg}");
        }
        assert_eq!(laws.cells, 64);
    }

    #[test]
    fn the_native_transitions_keep_the_models_laws() {
        let laws = Laws::parse(&source()).expect("laws");
        let (lo, hi) = (laws.weight_min, laws.weight_max);
        for w in lo..=hi {
            for b in [false, true] {
                assert!((lo..=hi).contains(&laws.learn(w, b)));
            }
            assert!(w <= laws.potentiate(w) && laws.depress(w) <= w);
            for c in 0..=hi {
                // winnerIsScheduledFirst
                assert!(laws.loser_weight(w) < laws.winner_weight(c, w));
            }
            if laws.consolidated(w) {
                assert!(laws.consolidated(laws.winner_weight(0, w)));
            }
        }
        assert!(!Laws::dominates((5, 5), (5, 5)));
        assert!(Laws::dominates((4, 5), (5, 5)) && !Laws::dominates((5, 5), (4, 5)));
        assert!(!Laws::dominates((4, 6), (5, 5)));
        assert_eq!(admit_mint(true, 3), MintVerdict::RefusedKnown);
        assert_eq!(admit_mint(false, 0), MintVerdict::RefusedBudget);
        assert_eq!(admit_mint(false, 3), MintVerdict::Minted(2));
        assert!(laws.valid_workflow());
    }

    #[test]
    fn every_law_the_engine_relies_on_is_a_model_theorem() {
        let have = theorems(&source());
        for t in [
            "stepFromBelowPow2",
            "ringIsPow2Cells",
            "potentiationStaysBounded",
            "depressionStaysBounded",
            "potentiationNeverLowers",
            "depressionNeverRaises",
            "winnerIsScheduledFirst",
            "winnerAtCeilingIsConsolidated",
            "dominanceIsIrreflexive",
            "dominanceIsAsymmetric",
            "dominanceIsTransitive",
            "dominanceNeverCostsBytes",
            "knownResultIsNeverRerun",
            "shippedWorkflowIsValid",
        ] {
            assert!(have.iter().any(|h| h == t), "missing theorem {t}");
        }
    }
}
