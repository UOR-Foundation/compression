//! UC-NEU-01: the native ncm coder's plasticity is the one the Prism model
//! `Uorc.Neuromorphic` states. The model's bounds are kernel-checked by
//! `lexlean verify`; this test holds the native constants equal to the
//! model's and the model's theorem inventory complete, so neither can drift.

use std::collections::BTreeMap;
use std::path::PathBuf;
use uorc::ncm::standard;

fn module() -> String {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../src/Uorc/Neuromorphic.lex.tex");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The value of each `field: value` pair of a declaration, in either the
/// one-line or the multi-line canonical layout.
fn field<'a>(decl: &'a str, key: &str) -> Option<&'a str> {
    let at = decl.find(&format!("{key}: "))? + key.len() + 2;
    let rest = &decl[at..];
    let end = rest.find([',', '}', '\n']).unwrap_or(rest.len());
    Some(rest[..end].trim())
}

/// Every parameterless `Nat` definition whose body is a literal.
fn constants(source: &str) -> BTreeMap<String, i64> {
    let mut out = BTreeMap::new();
    for decl in source.split("definition{").skip(1) {
        // a declaration ends at the blank line before the next
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

fn theorems(source: &str) -> Vec<String> {
    source
        .split("theorem{")
        .skip(1)
        .filter_map(|decl| field(decl, "name").map(str::to_string))
        .collect()
}

#[test]
fn native_plasticity_constants_are_the_models_uc_neu_01() {
    let c = constants(&module());
    let model = |name: &str| {
        *c.get(name)
            .unwrap_or_else(|| panic!("model constant {name}"))
    };
    let native: [(&str, i64); 18] = [
        ("mixerCount", standard::MIXERS as i64),
        ("mixerInputs", standard::MIXER_INPUTS as i64),
        ("slotContexts", standard::CONTEXTS as i64),
        ("weightLimit", standard::WEIGHT_LIMIT.into()),
        ("weightSpan", (2 * standard::WEIGHT_LIMIT).into()),
        ("weightInitial", standard::WEIGHT_INITIAL.into()),
        ("inputLimit", standard::INPUT_LIMIT.into()),
        ("errorLimit", standard::ERROR_LIMIT.into()),
        ("plasticityScale", 1 << standard::PLASTICITY_SHIFT),
        ("plasticityFloor", standard::PLASTICITY_FLOOR),
        ("plasticityGain", standard::PLASTICITY_GAIN),
        ("plasticityHalfLife", standard::PLASTICITY_HALF_LIFE),
        ("plasticityCeiling", standard::plasticity(0).into()),
        ("stepLimit", standard::STEP_LIMIT.into()),
        ("countLimit", standard::COUNT_LIMIT.into()),
        ("rateNumerator", standard::RATE_NUMERATOR.into()),
        ("refinementShift", standard::REFINEMENT_SHIFT.into()),
        // the APM target the coder learns toward: 2^16 + 2^shift - 2
        (
            "refinementTarget",
            (1 << 16) + (1 << standard::REFINEMENT_SHIFT) - 2,
        ),
    ];
    for (name, value) in native {
        assert_eq!(model(name), value, "{name}");
    }
    // The slot rates the model bounds are the native table's.
    let rate = |count: i64| i64::from(standard::RATE_NUMERATOR) / (2 * count + 3);
    assert_eq!(model("rateCeiling"), rate(0));
    assert_eq!(model("rateFloor"), rate(standard::COUNT_LIMIT.into()));
}

#[test]
fn native_plasticity_meets_the_models_vectors_and_bounds_uc_neu_01() {
    // the closed vectors the model proves with `decide`
    for (pos, lr) in [
        (0, 128),
        (4096, 76),
        (421_888, 25),
        (421_889, 24),
        (u32::MAX, 24),
    ] {
        assert_eq!(standard::plasticity(pos), lr, "plasticity({pos})");
    }
    // The share above the floor is the quotient `plasticityBoundsHold`
    // bounds: share * (4096 + pos) <= 104 * 4096, so 24 <= lr <= 128.
    let mut pos = 0u32;
    loop {
        let share = i64::from(standard::plasticity(pos)) - standard::PLASTICITY_FLOOR;
        let d = standard::PLASTICITY_HALF_LIFE + i64::from(pos);
        assert!(share * d <= standard::PLASTICITY_GAIN * standard::PLASTICITY_HALF_LIFE);
        assert!((0..=104).contains(&share));
        match pos.checked_add(1 + pos / 64) {
            Some(next) => pos = next,
            None => break,
        }
    }
    // `laneFitHolds`: the largest product and step, in the native lanes.
    let top = i64::from(standard::INPUT_LIMIT)
        * i64::from(standard::ERROR_LIMIT)
        * i64::from(standard::plasticity(0));
    assert_eq!(top, 1_073_217_536);
    assert!(top <= i64::from(i32::MAX));
    assert_eq!(
        (top + (1 << standard::PLASTICITY_SHIFT) - 1) >> standard::PLASTICITY_SHIFT,
        i64::from(standard::STEP_LIMIT)
    );
}

#[test]
fn the_model_proves_every_stated_bound_uc_neu_01() {
    let names = theorems(&module());
    for name in [
        "scaleIsMultiply",
        "hebbianBoundsHold",
        "potentiationNeverLowersHolds",
        "depressionNeverRaisesHolds",
        "plasticityBoundsHold",
        "plasticityAt0",
        "plasticityAt421889",
        "laneFitHolds",
        "slotRatesBoundedHolds",
        "slotRateAtConsolidation",
        "slotCountBoundHolds",
        "slotStepWithinGapHolds",
        "refinementBoundsHold",
    ] {
        assert!(names.iter().any(|n| n == name), "theorem {name}");
    }
}
