//! Open v0.1 obligations without their own measurement yet. Each test pins the
//! claim at `open` with no evidence class and no research disposition, so it is
//! reported and never asserted (R2) until a real measurement replaces this row.

use repo_model::{Level, Model};

fn assert_unmeasured_open(id: &str) {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let claim = model
        .ledger
        .claim
        .iter()
        .find(|claim| claim.id == id)
        .unwrap_or_else(|| panic!("{id} is registered"));
    assert_eq!(claim.level, Level::Open, "{id} must stay open");
    assert_eq!(claim.evidence_kind, "none", "{id} has no evidence class");
    assert_eq!(
        claim.research_disposition, "not_measured",
        "{id} is not measured"
    );
    let feature = claim.feature.as_deref().expect("open claim has a scenario");
    let text = std::fs::read_to_string(root.join(feature)).expect("feature file");
    assert!(
        text.contains(&format!("@{id} @open")),
        "{id} scenario is tagged open in {feature}"
    );
}

#[test]
fn machine_semantics_is_open_uc_mach_01() {
    assert_unmeasured_open("UC-MACH-01");
}

#[test]
fn logical_owned_bytes_is_open_uc_mach_02() {
    assert_unmeasured_open("UC-MACH-02");
}

#[test]
fn wire_profile_is_open_uc_wire_01() {
    assert_unmeasured_open("UC-WIRE-01");
}

#[test]
fn object_boundaries_is_open_uc_obj_01() {
    assert_unmeasured_open("UC-OBJ-01");
}

#[test]
fn streaming_integrity_is_open_uc_int_01() {
    assert_unmeasured_open("UC-INT-01");
}

#[test]
fn consumer_bridge_is_open_uc_int_06() {
    assert_unmeasured_open("UC-INT-06");
}

#[test]
fn conditional_basis_is_open_uc_bas_01() {
    assert_unmeasured_open("UC-BAS-01");
}

#[test]
fn fair_synthesis_is_open_uc_syn_01() {
    assert_unmeasured_open("UC-SYN-01");
}

#[test]
fn sealed_optimum_is_open_uc_opt_01() {
    assert_unmeasured_open("UC-OPT-01");
}

#[test]
fn checkpoint_replay_is_open_uc_chk_01() {
    assert_unmeasured_open("UC-CHK-01");
}

#[test]
fn real_archive_is_open_uc_real_01() {
    assert_unmeasured_open("UC-REAL-01");
}
