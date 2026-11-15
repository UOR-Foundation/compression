//! The codec's coordinates set away from their standard values. Setting a
//! coordinate changes what every coder in the process reads, so these tests
//! run in their own process, apart from the library's tests.

use uorc::coordinates::{with, Coordinate};

/// A model warmed at one coordinate value is never reused at another: after a
/// decoder kept the model warmed at one value, a frame coded at another still
/// decodes natively (the cache and the stored images are keyed by
/// [`uorc::coordinates::current`]).
#[test]
fn a_warmed_model_is_never_shared_across_coordinates() {
    let data = include_bytes!("../Cargo.toml");
    let half = Coordinate::PlasticityHalfLife;
    let native = |v: i64| {
        with(&[(half, v)], || {
            let s = uorc::compress(data, 0).expect("compresses");
            let frames = uorc::frames(&s).expect("frames");
            frames
                .iter()
                .map(|f| uorc::archive::fast_decode(f).or_else(|| uorc::archive::state_decode(f)))
                .collect::<Option<Vec<_>>>()
                .map(|parts| parts.concat())
        })
        .unwrap()
    };
    for v in [half.standard(), half.standard() * 2] {
        assert_eq!(native(v).as_deref(), Some(&data[..]), "half-life {v}");
    }
}
