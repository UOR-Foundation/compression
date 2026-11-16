//! The wire model's named quantities and constant tables, generated from its
//! sources (`just figures-native`), so the native code reads each one where the
//! model states it rather than restating it. A generated module is kept as the
//! generator writes it, so `figures check` can compare it with a regeneration.

#[rustfmt::skip]
pub mod archive;
#[rustfmt::skip]
pub mod frame;
#[rustfmt::skip]
pub mod graph;
#[rustfmt::skip]
pub mod framing;
#[rustfmt::skip]
pub mod ncm;
#[rustfmt::skip]
pub mod neuromorphic;
#[rustfmt::skip]
pub mod x86;
