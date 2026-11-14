//! The codec's coordinates: figures the engine decides by outcome.
//!
//! A coordinate is a figure of the coder that is not structural, such as the
//! mixers' plasticity. Until the engine has decided it, it codes at its
//! standard value (the one [`crate::ncm::standard`] states). The engine
//! measures other values over a κ-named corpus by running this codec with
//! the coordinate set ([`with`]). The value the coder reads is the set one,
//! or the standard one when none is set. Every decision is recorded in the
//! claim register under its κ, so no value is measured twice.
//!
//! A set coordinate changes the bytes of every frame the process codes. It
//! is a measuring instrument, not a format: [`with`] holds a lock and
//! restores the standard values when it returns, and nothing the client
//! writes for a user sets one.

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Mutex;

use crate::ncm::standard;

/// A coordinate of the coder.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Coordinate {
    /// [`standard::PLASTICITY_FLOOR`].
    PlasticityFloor,
    /// [`standard::PLASTICITY_GAIN`].
    PlasticityGain,
    /// [`standard::PLASTICITY_HALF_LIFE`].
    PlasticityHalfLife,
}

impl Coordinate {
    /// Every coordinate, in order.
    pub const ALL: &[Coordinate] = &[
        Coordinate::PlasticityFloor,
        Coordinate::PlasticityGain,
        Coordinate::PlasticityHalfLife,
    ];

    /// Its name in the claim register (the `Uorc.Neuromorphic` field).
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Coordinate::PlasticityFloor => "plasticityFloor",
            Coordinate::PlasticityGain => "plasticityGain",
            Coordinate::PlasticityHalfLife => "plasticityHalfLife",
        }
    }

    /// The coordinate a register name names.
    #[must_use]
    pub fn named(name: &str) -> Option<Coordinate> {
        Coordinate::ALL.iter().copied().find(|c| c.name() == name)
    }

    /// The value the coder codes at when the coordinate is not set.
    #[must_use]
    pub fn standard(self) -> i64 {
        match self {
            Coordinate::PlasticityFloor => standard::PLASTICITY_FLOOR,
            Coordinate::PlasticityGain => standard::PLASTICITY_GAIN,
            Coordinate::PlasticityHalfLife => standard::PLASTICITY_HALF_LIFE,
        }
    }

    /// The least value the coordinate can take: the half-life divides, so
    /// it starts at the multiplicative identity; the others are additive.
    #[must_use]
    pub fn least(self) -> i64 {
        match self {
            Coordinate::PlasticityHalfLife => 1,
            _ => 0,
        }
    }

    /// The value the coder reads now.
    #[must_use]
    pub fn get(self) -> i64 {
        match SET[self as usize].load(Ordering::Relaxed) {
            UNSET => self.standard(),
            v => v,
        }
    }
}

/// No value set: the coder reads the standard value. A value below every
/// coordinate's [`Coordinate::least`].
const UNSET: i64 = i64::MIN;
#[allow(clippy::declare_interior_mutable_const)]
const UNSET_CELL: AtomicI64 = AtomicI64::new(UNSET);
static SET: [AtomicI64; Coordinate::ALL.len()] = [UNSET_CELL; Coordinate::ALL.len()];
static MEASURING: Mutex<()> = Mutex::new(());

/// Run `f` with `values` set, then restore the standard values. Callers are
/// serialized, so two measurements never share a process's coordinates.
///
/// # Errors
/// A value below the coordinate's [`Coordinate::least`].
pub fn with<R>(values: &[(Coordinate, i64)], f: impl FnOnce() -> R) -> Result<R, String> {
    for &(c, v) in values {
        if v < c.least() {
            return Err(format!("{} = {v} is below {}", c.name(), c.least()));
        }
    }
    let _held = MEASURING
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    struct Restore;
    impl Drop for Restore {
        fn drop(&mut self) {
            for cell in &SET {
                cell.store(UNSET, Ordering::Relaxed);
            }
        }
    }
    let _restore = Restore;
    for &(c, v) in values {
        SET[c as usize].store(v, Ordering::Relaxed);
    }
    Ok(f())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_set_coordinate_is_read_inside_and_the_standard_one_after() {
        let c = Coordinate::PlasticityHalfLife;
        assert_eq!(c.get(), standard::PLASTICITY_HALF_LIFE);
        let inside = with(&[(c, 65536)], || c.get()).unwrap();
        assert_eq!(inside, 65536);
        assert_eq!(c.get(), standard::PLASTICITY_HALF_LIFE);
        assert!(with(&[(c, 0)], || ()).is_err());
        for &c in Coordinate::ALL {
            assert_eq!(Coordinate::named(c.name()), Some(c));
        }
    }
}
