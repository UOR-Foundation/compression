//! UC-PERF-02..06: the native client keeps the rules the Prism model
//! `Uorc.Performance` proves. The theorems are kernel-checked by
//! `lexlean verify`; this test holds the native constants equal to the
//! model's, the theorem inventory complete, and the native frames within the
//! bounds the theorems state for the model. One test per registered row.

use std::collections::BTreeMap;
use std::path::PathBuf;
use uorc::archive::{self, Codec};
use uorc::{ncm, performance, portfolio, x86, Representation};

fn module() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../src/Uorc/Performance.lex.tex");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn field<'a>(decl: &'a str, key: &str) -> Option<&'a str> {
    let at = decl.find(&format!("{key}: "))? + key.len() + 2;
    let rest = &decl[at..];
    let end = rest.find([',', '}', '\n']).unwrap_or(rest.len());
    Some(rest[..end].trim())
}

/// Every parameterless `Nat` definition whose body is a literal.
fn constants(source: &str) -> BTreeMap<String, u64> {
    let mut out = BTreeMap::new();
    for decl in source.split("definition{").skip(1) {
        let decl = decl.split("\n\n").next().unwrap_or(decl);
        let (Some(name), Some(result), Some(body)) = (
            field(decl, "name"),
            field(decl, "result"),
            field(decl, "body"),
        ) else {
            continue;
        };
        if decl.contains("parameters: [],") && result == "Nat" {
            if let Ok(v) = body.parse() {
                out.insert(name.to_string(), v);
            }
        }
    }
    out
}

/// Deterministic bytes no model predicts (xorshift64).
fn noise(n: usize, mut s: u64) -> Vec<u8> {
    (0..n)
        .map(|_| {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            (s >> 24) as u8
        })
        .collect()
}

fn text(n: usize) -> Vec<u8> {
    module().into_bytes().into_iter().cycle().take(n).collect()
}

/// Frames of several lengths and kinds: empty, tiny, text, noise, mixed.
fn samples() -> Vec<Vec<u8>> {
    let mut mixed = text(6000);
    mixed.extend(noise(6000, 7));
    vec![
        Vec::new(),
        vec![0],
        noise(127, 1),
        noise(128, 2),
        noise(5000, 3),
        noise(20_000, 4),
        text(300),
        text(20_000),
        vec![0xe8; 9000],
        mixed,
    ]
}

/// The frame a scheduled strategy writes for `frame` on its own.
fn strategy_frame(frame: &[u8], strategy: portfolio::Strategy) -> Option<Vec<u8>> {
    let data = if strategy.x86 {
        x86::filter(frame, false, 0).0
    } else {
        frame.to_vec()
    };
    let codec = if strategy.x86 {
        Codec::NcmX86
    } else {
        Codec::Ncm
    };
    let payload = ncm::encode_profile(&[], &data, strategy.profile)?;
    archive::linked_archive_profile(codec, strategy.profile, &[frame.len() as u64], &payload)
}

fn proved(names: &[&str]) {
    let source = module();
    let have: Vec<&str> = source
        .split("theorem{")
        .skip(1)
        .filter_map(|decl| field(decl, "name"))
        .collect();
    for name in names {
        assert!(have.contains(name), "theorem {name}");
    }
}

#[test]
fn no_native_frame_exceeds_its_input_plus_the_models_overhead_uc_perf_02() {
    proved(&[
        "ulebLengthBound",
        "rawLengthBound",
        "settleNoLargerThanRaw",
        "settleNoLargerThanKernel",
        "archiveBound",
    ]);
    let c = constants(&module());
    let native = [
        ("ulebLimit", archive::uleb_len(u64::MAX) as u64),
        ("rawFixed", archive::raw_length(&[0]) - 2),
        (
            "rawOverhead",
            archive::raw_length(&[0]) - 2 + 2 * archive::uleb_len(u64::MAX) as u64,
        ),
    ];
    for (name, value) in native {
        assert_eq!(c.get(name), Some(&value), "{name}");
    }
    for frame in samples() {
        let n = frame.len() as u64;
        // rawLength n = 2 * ulebLength n + n + rawFixed
        let raw = archive::raw_archive(&frame);
        assert_eq!(
            raw.len() as u64,
            2 * archive::uleb_len(n) as u64 + n + c["rawFixed"]
        );
        assert_eq!(raw.len() as u64, archive::raw_length(&[n]));
        // archiveBound: settled against the raw frame, within n + rawOverhead
        let kept = uorc::compress_frame(&frame, 0).unwrap();
        assert!(kept.len() <= raw.len(), "{n}: larger than raw");
        assert!(kept.len() as u64 <= n + c["rawOverhead"], "{n}");
    }
}

#[test]
fn the_native_portfolio_frame_is_no_larger_than_any_scheduled_strategys_uc_perf_03() {
    proved(&[
        "keepStaysNoLarger",
        "keepNoLargerThanHead",
        "keepNoLargerThanKept",
        "bestNoLargerThanEvery",
    ]);
    for frame in samples() {
        let kept = uorc::compress_frame(&frame, 0).unwrap();
        for strategy in portfolio::plan(&frame) {
            if let Some(alone) = strategy_frame(&frame, strategy) {
                assert!(kept.len() <= alone.len(), "{}: {strategy:?}", frame.len());
            }
        }
    }
}

#[test]
fn native_decode_steps_are_the_models_and_within_its_bound_uc_perf_04() {
    proved(&[
        "countBitsBound",
        "contextsBound",
        "inputsBound",
        "inputsFloor",
        "frameStepsBound",
        "decodeStepsBound",
        "fullModelSteps",
        "releasedModelSteps",
    ]);
    let c = constants(&module());
    let native = [
        ("bitsPerByte", performance::BITS_PER_BYTE),
        ("contextBits", performance::CONTEXT_BITS.into()),
        ("fixedInputs", performance::FIXED_INPUTS.into()),
        ("featureInputs", performance::FEATURE_INPUTS.into()),
        ("maxInputs", performance::MAX_INPUTS.into()),
        ("maxStepsPerByte", performance::MAX_STEPS_PER_BYTE),
    ];
    for (name, value) in native {
        assert_eq!(c.get(name), Some(&value), "{name}");
    }
    // the closed vectors the model proves: fullModelSteps, releasedModelSteps
    let steps = |profile| performance::steps(Representation::Ncm, Some(profile), 10);
    assert_eq!(steps(ncm::FULL), Some(1920));
    assert_eq!(steps(ncm::V03), Some(1600));
    // inputsFloor, inputsBound for every strategy and single-bit profile
    let profiles = portfolio::STRATEGIES
        .iter()
        .map(|s| s.profile)
        .chain((0..20).map(|b| 1 << b))
        .chain([0]);
    for profile in profiles {
        let inputs = performance::inputs(profile);
        assert!((performance::FIXED_INPUTS..=performance::MAX_INPUTS).contains(&inputs));
    }
    // decodeStepsBound on the frames the client writes
    for frame in samples() {
        let kept = uorc::compress_frame(&frame, 0).unwrap();
        let info = uorc::inspect(&kept).unwrap();
        let steps = performance::frame_steps(&info).expect("a measured representation");
        assert!(steps <= c["maxStepsPerByte"] * frame.len() as u64);
    }
}

#[test]
fn a_frame_coded_at_any_point_decodes_to_its_input_uc_perf_05() {
    proved(&[
        "admittedIsExact",
        "keepPreservesExact",
        "bestOfAdmittedIsExact",
    ]);
    for frame in samples() {
        // every strategy of the portfolio, scheduled for this frame or not
        for strategy in portfolio::STRATEGIES {
            if let Some(alone) = strategy_frame(&frame, strategy) {
                assert_eq!(uorc::decompress_frame(&alone, u64::MAX).unwrap(), frame);
                // the point is read from the header alone
                let info = uorc::inspect(&alone).unwrap();
                let point = performance::CodecPoint::of(&info);
                assert_eq!(point.profile & performance::POINT_X86 != 0, strategy.x86);
            }
        }
        let kept = uorc::compress_frame(&frame, 0).unwrap();
        assert_eq!(uorc::decompress_frame(&kept, u64::MAX).unwrap(), frame);
    }
}

#[test]
fn the_model_proves_dominance_is_a_strict_order_uc_perf_06() {
    proved(&[
        "dominanceLowersCost",
        "dominanceIsIrreflexive",
        "dominanceIsTransitive",
        "dominanceNeverGrows",
    ]);
}

/// Every pair of small keys, and `None` for a failed trial.
fn small_trials() -> Vec<Option<performance::Trial>> {
    let mut out = vec![None];
    for bytes in 0..5 {
        for steps in 0..5 {
            out.push(Some(performance::Trial { bytes, steps }));
        }
    }
    out
}

#[test]
fn a_bounded_trial_is_the_unbounded_one_or_cannot_win_uc_perf_07() {
    proved(&[
        "boundedSettlesTheSame",
        "settleOfKeep",
        "boundedPortfolioSettlesTheSame",
        "skippedTrialKeepsBytes",
        "skippedTrialKeepsSteps",
    ]);
    for frame in samples() {
        let raw = archive::raw_length(&[frame.len() as u64]);
        for strategy in portfolio::plan(&frame) {
            let Some(full) = ncm::encode_profile(&[], &frame, strategy.profile) else {
                continue;
            };
            // abandoned at any bound: the same payload, or none once the
            // payload reaches the bound
            for stop in [0, 1, full.len() / 2, full.len(), full.len() + 1, usize::MAX] {
                match ncm::encode_below(&[], &frame, strategy.profile, stop) {
                    Some(payload) => assert_eq!(payload, full),
                    None => assert!(full.len() >= stop, "{}: {stop}", frame.len()),
                }
            }
        }
        // settled against the raw frame, the bounded portfolio writes what an
        // unbounded one settles to: no scheduled frame below raw is lost
        let kept = uorc::compress_frame(&frame, 0).unwrap();
        let least = portfolio::plan(&frame)
            .into_iter()
            .filter_map(|s| strategy_frame(&frame, s))
            .map(|a| a.len() as u64)
            .fold(raw, u64::min);
        assert!(kept.len() as u64 <= least);
    }
}

#[test]
fn the_chosen_trial_is_least_and_undominated_uc_perf_08() {
    proved(&[
        "pickIsLeastOfKept",
        "pickIsLeastOfHead",
        "pickBytesAreSettled",
        "pickStaysLeast",
        "chosenIsLeast",
        "pickStaysUnderRaw",
        "chosenIsNoWorseThanRaw",
        "chosenKeepsTheSize",
        "leastIsUndominated",
        "excludedTrialKeepsBytes",
        "excludedTrialKeepsSteps",
    ]);
    let trials = small_trials();
    let raw = performance::Trial { bytes: 3, steps: 3 };
    // every list of three trials over the small keys
    for a in &trials {
        for b in &trials {
            for c in &trials {
                let list = [*a, *b, *c];
                let kept = performance::chosen(raw, &list);
                assert_eq!(performance::lex_gap(kept, raw), 0);
                // chosenKeepsTheSize: the bytes the size rule alone settles to
                let least = list
                    .iter()
                    .flatten()
                    .map(|t| t.bytes)
                    .fold(raw.bytes, u64::min);
                assert_eq!(kept.bytes, least);
                for t in list.iter().flatten().chain([&raw]) {
                    assert_eq!(performance::lex_gap(kept, *t), 0, "{list:?}");
                    assert_eq!(performance::gain(*t, kept), 0, "{list:?}");
                    // an excluded trial (no earlier than the kept one) changes nothing
                    assert_eq!(performance::pick(Some(*t), kept), kept);
                }
            }
        }
    }
}

#[test]
fn a_consolidation_spends_exactly_its_gain_uc_perf_09() {
    proved(&[
        "consolidationSpendsItsGain",
        "consolidationStaysUnder",
        "potentialNeverRises",
    ]);
    let trials: Vec<_> = small_trials().into_iter().flatten().collect();
    for &incumbent in &trials {
        for &candidate in &trials {
            let next = performance::consolidate(candidate, incumbent);
            let gain = performance::gain(candidate, incumbent);
            assert_eq!(next.cost() + gain, incumbent.cost());
            assert!(next.bytes <= incumbent.bytes && next.steps <= incumbent.steps);
            assert_eq!(gain > 0, next != incumbent);
        }
        // any sequence of candidates: the potential never rises
        let mut shipped = incumbent;
        for &candidate in trials.iter().rev() {
            let before = shipped.cost();
            shipped = performance::consolidate(candidate, shipped);
            assert!(shipped.cost() <= before);
        }
        assert!(shipped.cost() <= incumbent.cost());
    }
}

#[test]
fn the_encoder_stops_exactly_at_its_ledger_cap_uc_perf_10() {
    proved(&[
        "saturateStep",
        "ledgerSaturatesAtTheCap",
        "stopKeepsEveryPayloadBelowTheCap",
        "stopAbandonsExactlyAtTheCap",
        "aOneNarrowsTheWidth",
        "aOneSpendsAFraction",
        "aZeroNarrowsTheWidth",
        "aZeroSpendsAFraction",
    ]);
    assert!(module().contains("name: exactRawDecision,"));
    let mid = portfolio::MID;
    for data in [text(20_000), noise(20_000, 7)] {
        for profile in [ncm::FULL, mid, ncm::V03] {
            let full = ncm::encode_below(&[], &data, profile, usize::MAX).expect("payload");
            // payload = 4 header bytes + emitted + 1 final byte, so the cap
            // (stop - 5) is reached exactly when stop <= payload length
            let l = full.len();
            for stop in l - 3..=l + 3 {
                let bounded = ncm::encode_below(&[], &data, profile, stop);
                if stop > l {
                    assert_eq!(bounded.as_deref(), Some(&full[..]), "{profile:#x} {stop}");
                } else {
                    assert_eq!(bounded, None, "{profile:#x} {stop}");
                }
            }
        }
    }
    // the model cannot shrink random bytes: every shipped profile codes them
    // to more than raw, so the raw frame is decided by the ledger alone
    let random = noise(1 << 16, 11);
    for profile in [ncm::FULL, mid, ncm::V03] {
        let payload = ncm::encode_below(&[], &random, profile, usize::MAX).expect("payload");
        assert!(payload.len() > random.len(), "{profile:#x}");
    }
}

#[test]
fn a_shared_pass_writes_each_twins_own_payload_uc_perf_11() {
    let mid = portfolio::MID;
    assert_eq!(ncm::FULL ^ mid, ncm::F_SEL);
    for data in [text(30_000), noise(30_000, 3)] {
        let full = ncm::encode_learn_below(&[], &data, ncm::FULL, usize::MAX);
        let alone = ncm::encode_below(&[], &data, mid, usize::MAX);
        let (a, b) = ncm::encode_pair_below(&[], &data, ncm::FULL, true, [usize::MAX; 2]);
        assert_eq!(a, full);
        assert_eq!(b, alone);
        // the other direction, without learning
        let (b2, a2) = ncm::encode_pair_below(&[], &data, mid, false, [usize::MAX; 2]);
        assert_eq!(b2.map(|p| p.0), alone);
        assert_eq!(a2, ncm::encode_below(&[], &data, ncm::FULL, usize::MAX));
        // either side abandoned, the other is unchanged
        let la = full.as_ref().map_or(0, |p| p.0.len());
        let lb = alone.as_ref().map_or(0, |p| p.len());
        assert_eq!(
            ncm::encode_pair_below(&[], &data, ncm::FULL, true, [la / 2, usize::MAX]),
            (None, alone.clone())
        );
        assert_eq!(
            ncm::encode_pair_below(&[], &data, ncm::FULL, true, [usize::MAX, lb / 2]),
            (full.clone(), None)
        );
    }
    // the client codes twins in one pass when no core is spare: the archives
    // are the same at every thread count
    let input: Vec<u8> = text(300_000).into_iter().chain(noise(200_000, 5)).collect();
    let mut one = Vec::new();
    let mut four = Vec::new();
    uorc::compress_stream(&input[..], &mut one, 0, 1).expect("compress");
    uorc::compress_stream(&input[..], &mut four, 0, 4).expect("compress");
    assert_eq!(one, four);
    assert_eq!(uorc::decompress(&one, u64::MAX).expect("decode"), input);
}
