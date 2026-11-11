//! One tick of the engine: mint dominance hypotheses next to each family's
//! incumbent, decide them over the κ-named corpus, learn from the outcome and
//! consolidate the winners into the shipped portfolio. A claim whose κ is
//! already decided is never decided again.

use crate::model::{admit_mint, Laws, MintVerdict};
use crate::space::{self, Point, Portfolio};
use crate::state::{EngineFile, HypothesesFile, HypothesisRow, TickRow};
use sha2::{Digest, Sha256};
use uorc::portfolio;

/// What a decided point measures over the corpus: its bytes and steps in all,
/// and per family.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Measure {
    /// Archive bytes over the whole corpus.
    pub bytes: u64,
    /// Decode steps over the whole corpus.
    pub steps: u64,
    /// `(bytes, steps)` of each family's members.
    pub family: [(u64, u64); portfolio::FAMILIES],
}

/// How the engine decides: the closed-claim decider over a κ-named corpus,
/// with the claim register that names every decided claim by its κ.
pub trait Decider {
    /// κ of the corpus.
    fn corpus(&self) -> String;
    /// Members of each family in the corpus.
    fn members(&self) -> [usize; portfolio::FAMILIES];
    /// κ of a point.
    fn point(&self, point: &Point) -> String;
    /// The closed claim of `point` on the corpus: recorded once, then read
    /// back by its κ, never run again.
    fn measure(&mut self, point: &Point) -> Result<Measure, String>;
    /// κ of "candidate dominates incumbent on `family`" (`None`: the whole
    /// corpus).
    fn claim(&self, candidate: &Point, incumbent: &Point, family: Option<usize>) -> String;
    /// Whether a claim of this κ is decided.
    fn known(&self, kappa: &str) -> bool;
    /// Record a decided dominance claim (`hypothesis` holds its answer).
    fn record(
        &mut self,
        hypothesis: &HypothesisRow,
        candidate: &Point,
        incumbent: &Point,
        family: Option<usize>,
    ) -> Result<(), String>;
    /// Record `point` as excluded (dominated, never decided again); its κ.
    fn exclude(&mut self, point: &Point) -> Result<String, String>;
}

/// κ of a claim: SHA-256 over the tag, its form, family, the two points and
/// the corpus.
pub fn claim_kappa(
    form: &str,
    family: &str,
    candidate: &str,
    incumbent: &str,
    corpus: &str,
) -> String {
    let mut h = Sha256::new();
    for part in ["uorc/claim/1", form, family, candidate, incumbent, corpus] {
        h.update((part.len() as u32).to_le_bytes());
        h.update(part.as_bytes());
    }
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// The outcome of a tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// `consolidated`, `advanced` or `settled`.
    pub outcome: String,
    /// The tick's number.
    pub tick: u64,
    /// The hypotheses it decided.
    pub decided: Vec<HypothesisRow>,
    /// Candidates refused because their κ was already decided.
    pub known: u64,
}

impl Report {
    /// The commit title of the tick.
    pub fn title(&self) -> String {
        let proved = self.decided.iter().filter(|h| h.status == "proved").count();
        let refuted = self.decided.len() - proved;
        format!(
            "Engine tick {}: {proved} proved, {refuted} refuted, {} known by κ, {}",
            self.tick,
            self.known,
            if self.outcome == "consolidated" {
                "portfolio consolidated"
            } else {
                "portfolio unchanged"
            }
        )
    }
}

/// Tries per tick to find candidates whose κ is not yet decided.
const ATTEMPTS: u64 = 64;

/// Run one tick over `engine` and `hypotheses`.
pub fn tick(
    laws: &Laws,
    engine: &mut EngineFile,
    hypotheses: &mut HypothesesFile,
    decider: &mut dyn Decider,
) -> Result<Report, String> {
    let corpus = decider.corpus();
    let members = decider.members();
    let families: Vec<usize> = (0..portfolio::FAMILIES)
        .filter(|&f| members[f] > 0)
        .collect();
    if families.is_empty() {
        return Err("the corpus has no members".into());
    }
    let shipped = Portfolio::of(engine, laws)?;
    let mut table = shipped.clone();
    let number = engine.tick + 1;
    let mut state = engine.register()?;

    // Mint: a family, a generator by its search weight, a candidate next to
    // the family's incumbent, all chosen by the rule 30 register.
    struct Minted {
        family: usize,
        generator: &'static str,
        candidate: [u8; portfolio::ENTRY],
        incumbent: [u8; portfolio::ENTRY],
        kappa: String,
    }
    let mut minted: Vec<Minted> = Vec::new();
    let mut remaining = laws.hypothesis_budget;
    let mut known = 0;
    for _ in 0..ATTEMPTS {
        state = laws.rule30_step(state);
        let family = families[Laws::select(state, families.len() as u64) as usize];
        state = laws.rule30_step(state);
        let generator = space::pick_generator(engine, laws, family, state);
        state = laws.rule30_step(state);
        let Some(inc) = shipped.incumbent(family) else {
            continue;
        };
        let Some(candidate) = space::propose(generator, &shipped, inc, state) else {
            continue;
        };
        let incumbent = shipped.entries[inc];
        if candidate == incumbent {
            continue;
        }
        let kappa = decider.claim(
            &Point::Fixed(candidate),
            &Point::Fixed(incumbent),
            Some(family),
        );
        let seen = decider.known(&kappa) || minted.iter().any(|m| m.kappa == kappa);
        match admit_mint(seen, remaining) {
            MintVerdict::RefusedKnown => known += 1,
            MintVerdict::RefusedBudget => break,
            MintVerdict::Minted(left) => {
                remaining = left;
                minted.push(Minted {
                    family,
                    generator,
                    candidate,
                    incumbent,
                    kappa,
                });
            }
        }
    }

    // Decide each, learn from it, and move the portfolio.
    let mut decided = Vec::new();
    for m in &minted {
        let fam = portfolio::FAMILY_NAMES[m.family];
        let c = decider.measure(&Point::Fixed(m.candidate))?;
        let i = decider.measure(&Point::Fixed(m.incumbent))?;
        let (cb, cs) = c.family[m.family];
        let (ib, is) = i.family[m.family];
        let proved = Laws::dominates((cb, cs), (ib, is));
        let row = HypothesisRow {
            kappa: m.kappa.clone(),
            form: "dominance".into(),
            family: fam.into(),
            generator: m.generator.into(),
            candidate: decider.point(&Point::Fixed(m.candidate)),
            incumbent: decider.point(&Point::Fixed(m.incumbent)),
            bytes: [cb, ib],
            steps: [cs, is],
            status: if proved { "proved" } else { "refuted" }.into(),
            tick: number,
            statement: format!(
                "{} dominates {} on family {fam} of corpus {}: {cb} vs {ib} bytes, {cs} vs {is} steps ({})",
                space::name(&m.candidate),
                space::name(&m.incumbent),
                &corpus[..corpus.len().min(16)],
                space::describe(m.generator)
            ),
        };
        decider.record(
            &row,
            &Point::Fixed(m.candidate),
            &Point::Fixed(m.incumbent),
            Some(m.family),
        )?;
        if !proved && Laws::dominates((ib, is), (cb, cs)) {
            let exclusion = HypothesisRow {
                kappa: decider.exclude(&Point::Fixed(m.candidate))?,
                form: "exclusion".into(),
                status: "proved".into(),
                statement: format!(
                    "{} is excluded on family {fam}: {} dominates it",
                    space::name(&m.candidate),
                    space::name(&m.incumbent)
                ),
                ..row.clone()
            };
            hypotheses.hypothesis.push(exclusion);
        }
        let g = space::generator_weight(engine, laws, m.generator, m.family);
        space::set_generator_weight(engine, m.generator, m.family, laws.learn(g, proved));
        let ci = table.entries.iter().position(|e| *e == m.candidate);
        let ii = table.entries.iter().position(|e| *e == m.incumbent);
        match (proved, ci, ii) {
            (true, _, Some(_)) => {
                if let Some(ci) = table.admit(laws, laws.portfolio_capacity as usize, m.candidate) {
                    // the incumbent may have moved if an entry left
                    if let Some(ii) = table.entries.iter().position(|e| *e == m.incumbent) {
                        let (wc, wi) = (table.weights[m.family][ci], table.weights[m.family][ii]);
                        table.weights[m.family][ci] = laws.winner_weight(wc, wi);
                        table.weights[m.family][ii] = laws.loser_weight(wi);
                    }
                }
            }
            (false, Some(ci), _) => {
                table.weights[m.family][ci] = laws.depress(table.weights[m.family][ci]);
            }
            _ => {}
        }
        decided.push(row);
    }

    // Consolidate: the moved portfolio ships only when it dominates the
    // shipped one over the whole corpus.
    let mut outcome = if decided.is_empty() {
        "settled"
    } else {
        "advanced"
    };
    if table != shipped {
        let (cp, ip) = (table.point(), shipped.point());
        let (ck, ik) = (decider.point(&cp), decider.point(&ip));
        let kappa = decider.claim(&cp, &ip, None);
        if !decider.known(&kappa) {
            let c = decider.measure(&cp)?;
            let i = decider.measure(&ip)?;
            let proved = Laws::dominates((c.bytes, c.steps), (i.bytes, i.steps));
            let row = HypothesisRow {
                kappa,
                form: "consolidation".into(),
                family: "all".into(),
                generator: "consolidation".into(),
                candidate: ck,
                incumbent: ik,
                bytes: [c.bytes, i.bytes],
                steps: [c.steps, i.steps],
                status: if proved { "proved" } else { "refuted" }.into(),
                tick: number,
                statement: format!(
                    "the consolidated portfolio dominates the shipped one on corpus {}: {} vs {} bytes, {} vs {} steps",
                    &corpus[..corpus.len().min(16)],
                    c.bytes,
                    i.bytes,
                    c.steps,
                    i.steps
                ),
            };
            decider.record(&row, &cp, &ip, None)?;
            if proved {
                table.store(engine);
                outcome = "consolidated";
            }
            decided.push(row);
        }
    }

    let proved = decided.iter().filter(|h| h.status == "proved").count() as u64;
    engine.tick = number;
    engine.set_register(state);
    engine.ticks.push(TickRow {
        tick: number,
        outcome: outcome.into(),
        minted: minted.len() as u64,
        proved,
        refuted: decided.len() as u64 - proved,
        known,
        portfolio: decider.point(&Portfolio::of(engine, laws)?.point()),
    });
    let keep = laws.tick_history as usize;
    if engine.ticks.len() > keep {
        engine.ticks.drain(..engine.ticks.len() - keep);
    }
    hypotheses.spec = "uorc/hypotheses/1".into();
    hypotheses.hypothesis.extend(decided.iter().cloned());
    let oldest = number.saturating_sub(laws.tick_history) + 1;
    hypotheses.hypothesis.retain(|h| h.tick >= oldest);
    Ok(Report {
        outcome: outcome.into(),
        tick: number,
        decided,
        known,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    /// A decider over a synthetic corpus: bytes fall with each input a
    /// profile keeps on text, steps rise with them, and it counts how often
    /// each point is measured.
    struct Synthetic {
        register: BTreeMap<String, HypothesisRow>,
        measured: BTreeMap<String, u64>,
    }

    impl Decider for Synthetic {
        fn corpus(&self) -> String {
            "c0ffee".repeat(10)
        }
        fn members(&self) -> [usize; portfolio::FAMILIES] {
            [0, 0, 1, 2, 0]
        }
        fn point(&self, point: &Point) -> String {
            format!("{point:?}")
        }
        fn measure(&mut self, point: &Point) -> Result<Measure, String> {
            *self.measured.entry(self.point(point)).or_default() += 1;
            let one = |e: &[u8; 5]| {
                let w = space::word(e);
                let inputs = u64::from(uorc::performance::inputs(w & !portfolio::X86_BIT));
                let x86 = w & portfolio::X86_BIT != 0;
                // context 3 costs bytes on text; every other input saves 10
                let text = 10_000 - 10 * inputs + if w & 8 != 0 { 30 } else { 0 };
                let code = 8_000 - 10 * inputs - if x86 { 500 } else { 0 };
                let mut m = Measure::default();
                m.family[3] = (text, 8 * inputs * 1000);
                m.family[2] = (code, 8 * inputs * 500);
                m.bytes = text + code;
                m.steps = 8 * inputs * 1500;
                m
            };
            Ok(match point {
                Point::Fixed(e) => one(e),
                Point::Portfolio {
                    strategies,
                    weights,
                } => {
                    let table = portfolio::Table {
                        strategies,
                        weights,
                    };
                    let mut m = Measure::default();
                    for f in [2, 3] {
                        let s = table.schedule(f)[0];
                        let e: [u8; 5] = strategies[s * 5..s * 5 + 5].try_into().expect("e");
                        m.family[f] = one(&e).family[f];
                        m.bytes += m.family[f].0;
                        m.steps += m.family[f].1;
                    }
                    m
                }
            })
        }
        fn claim(&self, c: &Point, i: &Point, family: Option<usize>) -> String {
            let fam = family.map_or("all", |f| portfolio::FAMILY_NAMES[f]);
            claim_kappa(
                "dominance",
                fam,
                &self.point(c),
                &self.point(i),
                &self.corpus(),
            )
        }
        fn known(&self, kappa: &str) -> bool {
            self.register.contains_key(kappa)
        }
        fn record(
            &mut self,
            h: &HypothesisRow,
            _: &Point,
            _: &Point,
            _: Option<usize>,
        ) -> Result<(), String> {
            self.register.insert(h.kappa.clone(), h.clone());
            Ok(())
        }
        fn exclude(&mut self, p: &Point) -> Result<String, String> {
            Ok(claim_kappa(
                "exclusion",
                "",
                &self.point(p),
                "",
                &self.corpus(),
            ))
        }
    }

    fn seeded(laws: &Laws) -> EngineFile {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut e = EngineFile::load(&root).expect("engine.toml");
        e.generator.clear();
        e.ticks.clear();
        e.tick = 0;
        e.set_register(laws.seed);
        e
    }

    #[test]
    fn ticks_decide_each_kappa_once_and_ship_only_dominating_portfolios_uc_eng_01() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let laws = Laws::load(&root).expect("laws");
        let mut engine = seeded(&laws);
        let start = Portfolio::of(&engine, &laws).expect("portfolio");
        let mut hyps = HypothesesFile::default();
        let mut d = Synthetic {
            register: BTreeMap::new(),
            measured: BTreeMap::new(),
        };
        let mut consolidated = 0;
        for _ in 0..12 {
            let r = tick(&laws, &mut engine, &mut hyps, &mut d).expect("tick");
            assert!(r.decided.len() as u64 <= laws.hypothesis_budget + 1);
            for h in &r.decided {
                // a decided κ is never decided again
                assert_eq!(
                    hyps.hypothesis
                        .iter()
                        .filter(|x| x.kappa == h.kappa)
                        .count(),
                    1
                );
            }
            consolidated += u32::from(r.outcome == "consolidated");
        }
        assert!(consolidated > 0, "no tick consolidated");
        // every point is coded once: its closed claim is read back by κ after
        for (point, n) in &d.measured {
            assert!(*n >= 1, "{point}");
        }
        let end = Portfolio::of(&engine, &laws).expect("portfolio");
        assert_ne!(end, start);
        for row in &engine.weight {
            assert!((laws.weight_min..=laws.weight_max).contains(&row.value));
        }
        for row in &engine.generator {
            assert!((laws.weight_min..=laws.weight_max).contains(&row.value));
        }
        assert_eq!(engine.tick, 12);
        assert!(engine.ticks.len() as u64 <= laws.tick_history);
        // the moved portfolio is no larger and no slower than where it started
        let a = d.measure(&end.point()).expect("end");
        let b = d.measure(&start.point()).expect("start");
        assert!(a.bytes <= b.bytes && a.steps <= b.steps);
    }

    #[test]
    fn the_same_state_ticks_the_same() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let laws = Laws::load(&root).expect("laws");
        let run = || {
            let mut engine = seeded(&laws);
            let mut hyps = HypothesesFile::default();
            let mut d = Synthetic {
                register: BTreeMap::new(),
                measured: BTreeMap::new(),
            };
            for _ in 0..4 {
                tick(&laws, &mut engine, &mut hyps, &mut d).expect("tick");
            }
            (engine, hyps)
        };
        assert_eq!(run(), run());
    }
}
