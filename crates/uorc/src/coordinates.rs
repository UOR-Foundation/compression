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

    /// The greatest value the coordinate can take with the others at the
    /// values `others` gives: the mixers' plasticity may not pass the
    /// ceiling under which `Uorc.Neuromorphic::laneFitHolds` bounds every
    /// weight step ([`plasticity_ceiling`]), so floor and gain share it; the
    /// half-life only spreads the decay and has no bound but its type's.
    #[must_use]
    pub fn greatest(self, others: impl Fn(Coordinate) -> i64) -> i64 {
        match self {
            Coordinate::PlasticityFloor => {
                plasticity_ceiling() - others(Coordinate::PlasticityGain)
            }
            Coordinate::PlasticityGain => {
                plasticity_ceiling() - others(Coordinate::PlasticityFloor)
            }
            Coordinate::PlasticityHalfLife => i64::MAX,
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

/// The largest plasticity whose weight step stays within
/// [`standard::STEP_LIMIT`] for every input and error the mixers admit:
/// `STEP_LIMIT * 2^PLASTICITY_SHIFT / (INPUT_LIMIT * ERROR_LIMIT)`, the
/// model's `plasticityCeiling`.
#[must_use]
pub fn plasticity_ceiling() -> i64 {
    (i64::from(standard::STEP_LIMIT) << standard::PLASTICITY_SHIFT)
        / (i64::from(standard::INPUT_LIMIT) * i64::from(standard::ERROR_LIMIT))
}

/// The value of every coordinate the coder reads now, in
/// [`Coordinate::ALL`] order: what a model warmed in this process depends on
/// beyond the build, so a stored warmed model is keyed by it.
pub fn current() -> impl Iterator<Item = i64> {
    Coordinate::ALL.iter().map(|c| c.get())
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
    let value = |c: Coordinate| {
        values
            .iter()
            .find(|(x, _)| *x == c)
            .map_or(c.standard(), |&(_, v)| v)
    };
    for &(c, v) in values {
        if v < c.least() {
            return Err(format!("{} = {v} is below {}", c.name(), c.least()));
        }
        if v > c.greatest(value) {
            return Err(format!("{} = {v} is above {}", c.name(), c.greatest(value)));
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
        assert_eq!(plasticity_ceiling(), 128);
        let floor = Coordinate::PlasticityFloor;
        assert!(with(&[(floor, 128 - standard::PLASTICITY_GAIN)], || ()).is_ok());
        assert!(with(&[(floor, 129 - standard::PLASTICITY_GAIN)], || ()).is_err());
        for &c in Coordinate::ALL {
            assert_eq!(Coordinate::named(c.name()), Some(c));
        }
    }

    /// A model warmed at one coordinate value is never reused at another:
    /// after a decoder kept the model warmed at one value, a frame coded at
    /// another still decodes natively (the cache and the stored images are
    /// keyed by [`current`]).
    #[test]
    fn a_warmed_model_is_never_shared_across_coordinates() {
        let data = include_bytes!("../Cargo.toml");
        let half = Coordinate::PlasticityHalfLife;
        let native = |v: i64| {
            with(&[(half, v)], || {
                let s = crate::compress(data, 0).expect("compresses");
                let frames = crate::frames(&s).expect("frames");
                frames
                    .iter()
                    .map(|f| {
                        crate::archive::fast_decode(f).or_else(|| crate::archive::state_decode(f))
                    })
                    .collect::<Option<Vec<_>>>()
                    .map(|parts| parts.concat())
            })
            .unwrap()
        };
        for v in [half.standard(), half.standard() * 2] {
            assert_eq!(native(v).as_deref(), Some(&data[..]), "half-life {v}");
        }
    }
}
