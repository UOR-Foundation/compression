//! The engine's decider: the claim register `frontier.toml` and its decider
//! (`crates/frontier`), which run the shipped codec over a κ-named corpus.
//! Every closed, dominance and exclusion claim the engine decides is put in
//! the register under its κ, so no question is decided twice.

use crate::space::Point;
use crate::state::HypothesisRow;
use crate::tick::{Decider, Measure};
use frontier::{CodecPoint, Corpus, Plan, Register, Status};
use std::path::Path;
use uorc::portfolio::{self, Strategy};

/// The register file, relative to the repository root.
pub const REGISTER: &str = "frontier.toml";

/// The register, one of its corpora with the members' bytes, the export the
/// points are coded by, and who decides (`local`, or the factory's receipt).
pub struct FrontierDecider {
    /// The claim register.
    pub register: Register,
    corpus: Corpus,
    files: Vec<Vec<u8>>,
    export: String,
    by: String,
    members: [usize; portfolio::FAMILIES],
}

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

impl FrontierDecider {
    /// Open the register at `root` and materialize corpus `name`.
    pub fn open(root: &Path, name: &str, by: &str) -> Result<FrontierDecider, String> {
        let register = Register::load(&root.join(REGISTER)).map_err(err)?;
        let corpus = register.corpus_named(name).map_err(err)?.clone();
        let files = corpus.materialize(None, root).map_err(err)?;
        let export = frontier::export_identity(root).map_err(err)?;
        let mut members = [0; portfolio::FAMILIES];
        for f in &files {
            members[portfolio::family(&f[..f.len().min(uorc::FRAME)])] += 1;
        }
        Ok(FrontierDecider {
            register,
            corpus,
            files,
            export,
            by: by.to_owned(),
            members,
        })
    }

    /// Write the register back.
    pub fn save(&self, root: &Path) -> Result<(), String> {
        self.register.save(&root.join(REGISTER)).map_err(err)
    }

    fn codec(&self, point: &Point) -> CodecPoint {
        let plan = match point {
            Point::Fixed(e) => {
                let word = crate::space::word(e);
                Plan::Fixed(Strategy {
                    x86: word & portfolio::X86_BIT != 0,
                    profile: word & !portfolio::X86_BIT,
                })
            }
            Point::Portfolio {
                strategies,
                weights,
            } => Plan::Portfolio {
                strategies: strategies.clone(),
                weights: weights.clone(),
            },
        };
        CodecPoint {
            export: self.export.clone(),
            plan,
            state: None,
            lanes: 1,
        }
    }

    fn family(family: Option<usize>) -> Option<&'static str> {
        family.map(|f| portfolio::FAMILY_NAMES[f])
    }
}

impl Decider for FrontierDecider {
    fn corpus(&self) -> String {
        self.corpus.kappa.clone()
    }

    fn members(&self) -> [usize; portfolio::FAMILIES] {
        self.members
    }

    fn point(&self, point: &Point) -> String {
        self.codec(point).kappa()
    }

    fn measure(&mut self, point: &Point) -> Result<Measure, String> {
        let codec = self.codec(point);
        let closed = self
            .register
            .decide(&codec, &self.corpus, &self.files, &self.by)
            .map_err(err)?;
        let mut m = Measure {
            bytes: closed.bytes,
            steps: closed.steps,
            members: closed.members.iter().map(|r| r.bytes).collect(),
            ..Measure::default()
        };
        for (f, name) in portfolio::FAMILY_NAMES.iter().enumerate() {
            m.family[f] = closed.family(Some(name));
        }
        Ok(m)
    }

    fn claim(&self, candidate: &Point, incumbent: &Point, family: Option<usize>) -> String {
        let mut q = Register::closed_question(&self.codec(candidate), &self.corpus);
        q.form = frontier::Form::Dominance;
        q.codec = None;
        q.candidate = Some(self.point(candidate));
        q.incumbent = Some(self.point(incumbent));
        q.family = Self::family(family).map(str::to_owned);
        frontier::claim_kappa(&q)
    }

    fn known(&self, kappa: &str) -> bool {
        self.register
            .known(kappa)
            .is_some_and(|c| c.status != Status::Minted)
    }

    fn record(
        &mut self,
        hypothesis: &HypothesisRow,
        candidate: &Point,
        incumbent: &Point,
        family: Option<usize>,
    ) -> Result<(), String> {
        let (c, i) = (self.codec(candidate), self.codec(incumbent));
        let claim = self
            .register
            .decide_dominance(&c, &i, Self::family(family), &self.corpus, &self.by)
            .ok_or("a dominance claim without both closed claims")?;
        // the register decides dominance from the same closed claims the
        // tick read (a consolidation also requires that no member grows,
        // which the tick checks beside it)
        let dominance = crate::model::Laws::dominates(
            (hypothesis.bytes[0], hypothesis.steps[0]),
            (hypothesis.bytes[1], hypothesis.steps[1]),
        );
        if claim.id != hypothesis.kappa || (claim.status == Status::Proved) != dominance {
            return Err(format!(
                "the register decided {} {:?}, the tick read {} {dominance}",
                claim.id, claim.status, hypothesis.kappa
            ));
        }
        Ok(())
    }

    fn exclude(&mut self, point: &Point) -> Result<String, String> {
        let codec = self.codec(point);
        Ok(self.register.exclude(&codec, &self.corpus, &self.by).id)
    }
}
