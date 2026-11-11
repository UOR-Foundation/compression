//! The κ-named model states of the standard basis (`client/gen/gen_state.py`).
//!
//! A model state is a basis root: bytes the ncm coder codes first, emitting
//! nothing for them, so that a frame is coded as their continuation
//! ([`crate::ncm::encode_warm`]). A state frame is a linked archive over the
//! basis `[library, S]`, named by that basis's identity like any one-root
//! window ([`crate::archive::window_identity`]), so the identity binds the
//! state's bytes: a decoder that lacks them cannot read the frame, and one
//! that has them reads exactly what was coded. A state can change the size
//! of an archive, never its meaning. States are appended, never changed, so
//! every archive that names one keeps its meaning.

use sha2::{Digest, Sha256};
use std::sync::OnceLock;

/// State 1: 256 KiB consolidated from this repository's own text, code and
/// configuration at the v0.4.0 tree (`gen_state.py`).
pub static STATE_1: &[u8] = include_bytes!("state-1.bin");

/// Every state of the basis, in order; strategy state `i` is `ALL[i - 1]`.
pub static ALL: [&[u8]; 1] = [STATE_1];

/// SHA-256 of each state's bytes (its κ), in order.
pub const KAPPA: [&str; 1] = ["6ae4057d2dc5800b1a80b345300b2521c65bcd68e4201212b1afc408385b695d"];

/// The state with 1-based index `i`, if the basis has it.
pub fn state(i: u8) -> Option<&'static [u8]> {
    ALL.get(usize::from(i).checked_sub(1)?).copied()
}

/// The identity of the basis `[library, state i]` that a state frame names.
pub fn identity(i: u8) -> Option<[u8; 32]> {
    static IDS: OnceLock<Vec<[u8; 32]>> = OnceLock::new();
    let ids = IDS.get_or_init(|| ALL.iter().map(|s| crate::archive::frame_label(s)).collect());
    ids.get(usize::from(i).checked_sub(1)?).copied()
}

/// The 1-based index of the state whose basis has `identity`.
pub fn named(id: &[u8]) -> Option<u8> {
    (1..=ALL.len() as u8).find(|&i| identity(i).as_ref().map(|x| &x[..]) == Some(id))
}

/// The κ of state `i`: SHA-256 of its bytes.
pub fn kappa_bytes(i: u8) -> Option<[u8; 32]> {
    Some(Sha256::digest(state(i)?).into())
}

/// The κ of state `i` as lowercase hex.
pub fn kappa(i: u8) -> Option<String> {
    Some(kappa_bytes(i)?.iter().map(|b| format!("{b:02x}")).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn states_are_their_kappa() {
        for (i, k) in KAPPA.iter().enumerate() {
            assert_eq!(kappa(i as u8 + 1).as_deref(), Some(*k));
        }
        assert_eq!(state(0), None);
        assert_eq!(state(ALL.len() as u8 + 1), None);
        assert_eq!(named(&identity(1).unwrap()), Some(1));
        assert_eq!(named(&[0; 32]), None);
    }
}
