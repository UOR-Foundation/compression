//! The shipped κ-named model states, end to end through the exported model.
//! The unit tests run the client over short slices of the states (see
//! `states::ALL`), because the exported model replays a state at about
//! 40 µs per bit; this test checks the shipped ones, once per state the
//! portfolio schedules for a small text frame.

fn text(n: usize, seed: u32) -> Vec<u8> {
    let words = [
        &b"archive "[..],
        b"frame ",
        b"state ",
        b"model ",
        b"byte ",
        b"window\n",
    ];
    let mut x = seed;
    let mut out = Vec::with_capacity(n + 8);
    while out.len() < n {
        x = x.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        out.extend_from_slice(words[(x >> 16) as usize % words.len()]);
    }
    out.truncate(n);
    out
}

/// A small text frame: the native archive is the exported model's archive
/// over the shipped states, and both decoders restore it.
#[test]
fn shipped_states_native_is_the_model() {
    let input = text(1500, 5);
    let native = uorc::compress_frame(&input, 0).expect("native");
    let model = uorc::model_compress_frame(&input).expect("model");
    assert_eq!(native, model);
    assert_eq!(uorc::model_decompress_frame(&native).as_ref(), Ok(&input));
    assert_eq!(
        uorc::decompress(&native, uorc::DEFAULT_DECODE_WORK).as_ref(),
        Ok(&input)
    );
}

/// Continuing a shipped state codes a small text frame smaller than the
/// same profile alone: the least state frame beats the FULL frame.
#[test]
fn shipped_states_make_small_text_smaller() {
    use uorc::{archive, ncm, portfolio, states};
    let input = text(3000, 7);
    let lengths = [input.len() as u64];
    let least = portfolio::STRATEGIES
        .iter()
        .filter(|s| s.state > 0 && !s.x86)
        .map(|s| {
            let state = states::state(s.state).expect("shipped");
            let id = states::identity(s.state).expect("named");
            let p = ncm::encode_warm(state, &input, s.profile).expect("codes");
            archive::state_archive(
                false,
                s.profile,
                uorc::ncm::Learning::STANDARD,
                &id,
                &lengths,
                &p,
            )
            .expect("frame")
            .len()
        })
        .min()
        .expect("a state strategy");
    let alone = archive::linked_archive_profile(
        archive::Codec::Ncm,
        uorc::ncm::streams::Word::from(ncm::FULL),
        &lengths,
        &ncm::encode_profile(&[], &input, ncm::FULL).expect("codes"),
    )
    .expect("frame");
    assert!(least < alone.len(), "{least} vs {}", alone.len());
}
