//! The uorc engine (`Uorc.Engine`, `src/Uorc/Engine.lex.tex`): the codec is a
//! point in a modelled space (the portfolio's strategies, each a context set
//! with its filter and κ-named state, and their weights per family), and each
//! tick moves it. A tick mints "candidate dominates incumbent on family f" by
//! the rule 30 register and the search weights, decides it in exact bytes and
//! steps over a κ-named corpus, learns from the outcome under the bounded
//! Hebbian rule, and consolidates the winners into the shipped portfolio
//! (`crates/uorc/src/weights.rs`) when the moved portfolio dominates the
//! shipped one. A claim known by its κ is never decided again. No LLM is in
//! the loop: the model states the laws and the decider runs the shipped code.

pub mod decide;
pub mod model;
pub mod render;
pub mod space;
pub mod state;
pub mod tick;
