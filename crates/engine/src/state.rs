//! The engine's state files: `model/engine.toml` (the consolidated portfolio,
//! the search weights and the rule 30 register) and `model/hypotheses.toml`
//! (what recent ticks minted and how each was decided).

use serde::{Deserialize, Serialize};
use std::path::Path;

/// `model/engine.toml`.
pub const ENGINE: &str = "model/engine.toml";
/// `model/hypotheses.toml`.
pub const HYPOTHESES: &str = "model/hypotheses.toml";
/// The generated shipped table.
pub const WEIGHTS_RS: &str = "crates/uorc/src/weights.rs";

/// One strategy entry of the portfolio.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyRow {
    /// The u32 word, `0x` hex: bits 0-19 the ncm profile, bit 31 the x86 filter.
    pub word: String,
    /// The κ-named model state's index in the basis (0 for none).
    pub state: u8,
}

/// The weight of one (strategy, family) pair of the portfolio.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeightRow {
    /// The strategy's index in `[[strategy]]`.
    pub strategy: usize,
    /// The family's name.
    pub family: String,
    /// The weight, in `weightMin..=weightMax`.
    pub value: u64,
}

/// The weight of one (generator, family) pair: how the engine searches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratorRow {
    /// The generator's name.
    pub generator: String,
    /// The family's name.
    pub family: String,
    /// The weight, in `weightMin..=weightMax`.
    pub value: u64,
}

/// What one tick did.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TickRow {
    /// The tick's number.
    pub tick: u64,
    /// `advanced`, `consolidated` or `settled`.
    pub outcome: String,
    /// Hypotheses minted.
    pub minted: u64,
    /// Dominance claims proved.
    pub proved: u64,
    /// Dominance claims refuted.
    pub refuted: u64,
    /// Candidates refused because their κ was already decided.
    pub known: u64,
    /// κ of the shipped portfolio after the tick.
    pub portfolio: String,
}

/// `model/engine.toml` (`uorc/engine/1`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineFile {
    /// `uorc/engine/1`.
    pub spec: String,
    /// Ticks run so far.
    pub tick: u64,
    /// The rule 30 register, `0x` hex.
    pub rule30_state: String,
    /// The shipped portfolio's strategies, in portfolio order.
    #[serde(default)]
    pub strategy: Vec<StrategyRow>,
    /// The shipped portfolio's weights.
    #[serde(default)]
    pub weight: Vec<WeightRow>,
    /// The search weights.
    #[serde(default)]
    pub generator: Vec<GeneratorRow>,
    /// The most recent ticks (`tickHistory` of them).
    #[serde(default, rename = "tick_record")]
    pub ticks: Vec<TickRow>,
}

/// One minted hypothesis and its decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HypothesisRow {
    /// κ of the claim.
    pub kappa: String,
    /// `dominance` or `consolidation`.
    pub form: String,
    /// The family the claim is about (`all` for the whole corpus).
    pub family: String,
    /// The generator that proposed the candidate.
    pub generator: String,
    /// κ of the candidate point.
    pub candidate: String,
    /// κ of the incumbent point.
    pub incumbent: String,
    /// The candidate's and the incumbent's bytes on the family.
    pub bytes: [u64; 2],
    /// The candidate's and the incumbent's steps on the family.
    pub steps: [u64; 2],
    /// `proved` or `refuted`.
    pub status: String,
    /// The tick that minted it.
    pub tick: u64,
    /// The claim in words.
    pub statement: String,
}

/// `model/hypotheses.toml` (`uorc/hypotheses/1`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HypothesesFile {
    /// `uorc/hypotheses/1`.
    pub spec: String,
    /// The hypotheses of the most recent ticks, oldest first.
    #[serde(default)]
    pub hypothesis: Vec<HypothesisRow>,
}

fn read<T: for<'de> Deserialize<'de>>(root: &Path, rel: &str) -> Result<T, String> {
    let path = root.join(rel);
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

fn write<T: Serialize>(root: &Path, rel: &str, head: &str, value: &T) -> Result<(), String> {
    let body = toml::to_string(value).map_err(|e| format!("{rel}: {e}"))?;
    let path = root.join(rel);
    std::fs::write(&path, format!("{head}{body}")).map_err(|e| format!("{}: {e}", path.display()))
}

impl EngineFile {
    /// Read `model/engine.toml`.
    pub fn load(root: &Path) -> Result<EngineFile, String> {
        let file: EngineFile = read(root, ENGINE)?;
        if file.spec != "uorc/engine/1" {
            return Err(format!("{ENGINE}: spec {} is not uorc/engine/1", file.spec));
        }
        Ok(file)
    }

    /// Write `model/engine.toml`.
    pub fn save(&self, root: &Path) -> Result<(), String> {
        write(
            root,
            ENGINE,
            "# The uorc engine's consolidated state (`uorc/engine/1`), written by\n\
             # `uorc-engine tick` under the laws of src/Uorc/Engine.lex.tex.\n\
             # [[strategy]] and [[weight]] are the shipped portfolio (crates/uorc/src/weights.rs\n\
             # is generated from them); [[generator]] weights order the engine's search.\n\n",
            self,
        )
    }

    /// The rule 30 register.
    pub fn register(&self) -> Result<u64, String> {
        u64::from_str_radix(self.rule30_state.trim_start_matches("0x"), 16)
            .map_err(|e| format!("{ENGINE}: rule30_state: {e}"))
    }

    /// Set the rule 30 register.
    pub fn set_register(&mut self, state: u64) {
        self.rule30_state = format!("0x{state:016x}");
    }
}

impl HypothesesFile {
    /// Read `model/hypotheses.toml` (empty when absent).
    pub fn load(root: &Path) -> Result<HypothesesFile, String> {
        if !root.join(HYPOTHESES).exists() {
            return Ok(HypothesesFile {
                spec: "uorc/hypotheses/1".into(),
                hypothesis: Vec::new(),
            });
        }
        read(root, HYPOTHESES)
    }

    /// Write `model/hypotheses.toml`.
    pub fn save(&self, root: &Path) -> Result<(), String> {
        write(
            root,
            HYPOTHESES,
            "# Hypotheses the uorc engine minted (`uorc/hypotheses/1`) in its most recent\n\
             # ticks, with each decision; every decided claim is also in frontier.toml by κ.\n\n",
            self,
        )
    }
}
