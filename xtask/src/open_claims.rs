//! Rows that stay open until the client ships the capability they describe. Each test pins the
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
fn conditional_basis_is_open_uc_bas_01() {
    assert_unmeasured_open("UC-BAS-01");
}
