//! The performance vocabulary of the Prism model (`Uorc.Performance`), as the
//! native client computes it: the step measure and the codec point.
//!
//! A **step** is one model input evaluated for one coded bit. It is exact and
//! deterministic, and it is read from a frame's header alone: a raw frame
//! costs one step per output byte (the copy), a reference frame costs none
//! (its bytes were produced once already), and an ncm frame costs eight bits
//! times the inputs its profile word keeps, per output byte, plus one step per
//! byte when the x86 filter runs. The constants are the model's; the test
//! in `tests/performance.rs` reads them from the module text.

use crate::frame::{Info, Representation};
use crate::ncm;
use sha2::{Digest, Sha256};

/// Coded bits per output byte.
pub const BITS_PER_BYTE: u64 = 8;
/// Inputs every ncm profile evaluates beside its contexts: the match model's
/// two and the bias.
pub const FIXED_INPUTS: u32 = 3;
/// Inputs each optional model (short match, repeat distance) adds.
pub const FEATURE_INPUTS: u32 = 2;
/// Context bits of a profile word.
pub const CONTEXT_BITS: u32 = 17;
/// The most inputs any profile evaluates: 17 contexts, the fixed 3, two features.
pub const MAX_INPUTS: u32 = CONTEXT_BITS + FIXED_INPUTS + 2 * FEATURE_INPUTS;
/// The bound of `Uorc.Performance.decodeStepsBound`: no frame costs more
/// steps per output byte than this (the full model over the x86 filter).
pub const MAX_STEPS_PER_BYTE: u64 = BITS_PER_BYTE * MAX_INPUTS as u64 + 1;
/// Domain tag of a codec point's identity.
pub const CODEC_POINT_TAG: &[u8] = b"uorc/codec-point/1";

/// Model inputs an ncm frame coded under `profile` evaluates per bit.
pub fn inputs(profile: u32) -> u32 {
    (profile & ((1 << CONTEXT_BITS) - 1)).count_ones()
        + FIXED_INPUTS
        + FEATURE_INPUTS * u32::from(profile & ncm::F_SHORT != 0)
        + FEATURE_INPUTS * u32::from(profile & ncm::F_REP != 0)
}

/// Steps to decode `n` output bytes of a frame of this representation and
/// profile word; `None` for a representation the model states no measure for
/// (the ulz decoders kept for earlier archives, and searched programs).
pub fn steps(representation: Representation, profile: Option<u32>, n: u64) -> Option<u64> {
    match representation {
        Representation::Raw => Some(n),
        Representation::Reference => Some(0),
        Representation::Ncm => {
            Some(BITS_PER_BYTE * u64::from(inputs(profile.unwrap_or(ncm::V03))) * n)
        }
        Representation::NcmX86 => {
            Some((BITS_PER_BYTE * u64::from(inputs(profile.unwrap_or(ncm::V03))) + 1) * n)
        }
        Representation::Ulz | Representation::UlzKernel | Representation::Program => None,
    }
}

/// [`steps`] of the frame a header describes.
pub fn frame_steps(info: &Info) -> Option<u64> {
    steps(info.representation, info.ncm_profile, info.output_len)
}

/// A point of the codec space: which decoder basis, profile word, consolidated
/// model state and lane count code a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodecPoint {
    /// The basis identity the frame names (the export's library), zeros when
    /// the frame is self-contained.
    pub basis: [u8; 32],
    /// The ncm profile word, 0 when the frame applies no ncm decoder.
    pub profile: u32,
    /// κ of the consolidated model state the frame starts from, zeros for none.
    pub state: [u8; 32],
    /// Lanes the frame is coded in (1 for a single stream).
    pub lanes: u32,
}

impl CodecPoint {
    /// The point of the frame a header describes (no state, one lane).
    pub fn of(info: &Info) -> CodecPoint {
        CodecPoint {
            basis: info.basis.unwrap_or([0; 32]),
            profile: info.ncm_profile.unwrap_or(0),
            state: [0; 32],
            lanes: 1,
        }
    }

    /// κ of the point: SHA-256 over the tag, the basis, the profile word
    /// (little-endian), the state and the lane count (little-endian).
    pub fn kappa(&self) -> [u8; 32] {
        let mut h = Sha256::new();
        h.update(CODEC_POINT_TAG);
        h.update(self.basis);
        h.update(self.profile.to_le_bytes());
        h.update(self.state);
        h.update(self.lanes.to_le_bytes());
        h.finalize().into()
    }
}

/// κ of a corpus: SHA-256 over `uorc/corpus/1` and, for each object in order,
/// its name length (u32 LE), name, SHA-256 and length (u64 LE).
pub fn corpus_kappa<'a>(objects: impl IntoIterator<Item = (&'a str, [u8; 32], u64)>) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"uorc/corpus/1");
    for (name, digest, len) in objects {
        h.update((name.len() as u32).to_le_bytes());
        h.update(name.as_bytes());
        h.update(digest);
        h.update(len.to_le_bytes());
    }
    h.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_follow_the_profile_word() {
        assert_eq!(inputs(ncm::V03), 20);
        assert_eq!(inputs(ncm::FULL), MAX_INPUTS);
        assert_eq!(steps(Representation::Raw, None, 10), Some(10));
        assert_eq!(steps(Representation::Reference, None, 10), Some(0));
        assert_eq!(steps(Representation::Ncm, Some(ncm::FULL), 10), Some(1920));
        assert_eq!(
            steps(Representation::NcmX86, Some(ncm::FULL), 10),
            Some(10 * MAX_STEPS_PER_BYTE)
        );
        for profile in [ncm::V03, ncm::FULL, 1, 0x7ffff] {
            assert!(inputs(profile) <= MAX_INPUTS);
        }
    }

    #[test]
    fn a_codec_point_is_named_by_all_its_coordinates() {
        let p = CodecPoint {
            basis: [1; 32],
            profile: ncm::FULL,
            state: [0; 32],
            lanes: 1,
        };
        assert_ne!(p.kappa(), CodecPoint { lanes: 2, ..p }.kappa());
        assert_ne!(
            p.kappa(),
            CodecPoint {
                state: [2; 32],
                ..p
            }
            .kappa()
        );
        assert_ne!(
            p.kappa(),
            CodecPoint {
                profile: ncm::V03,
                ..p
            }
            .kappa()
        );
        assert_eq!(p.kappa(), p.kappa());
    }
}
