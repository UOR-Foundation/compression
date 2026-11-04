//! Binding checks for the separately executed product component diagnostic.
//! These receipts do not assert pinned-SDK, native, browser or release acceptance.

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::sync::OnceLock;

    use repo_model::{Level, Model};
    use serde_json::Value;
    use sha2::{Digest, Sha256};

    fn root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("xtask has a repository parent")
            .to_path_buf()
    }

    fn digest(path: &Path) -> String {
        format!(
            "{:x}",
            Sha256::digest(std::fs::read(path).expect("bound file exists"))
        )
    }

    fn list_length(term: &Value) -> u64 {
        match term["kind"].as_str() {
            Some("nil") => 0,
            Some("cons") => 1 + list_length(&term["tail"]),
            _ => panic!("the source-owned scenario inventory is not a closed typed list"),
        }
    }

    fn check_receipt() {
        static CHECKED: OnceLock<()> = OnceLock::new();
        CHECKED.get_or_init(|| {
            let root = root();
            let receipt: Value = serde_json::from_slice(
                &std::fs::read(root.join("validation/product/evidence.json"))
                    .expect("execute scripts/product-control-validation.py first"),
            )
            .expect("diagnostic receipt is JSON");
            assert_eq!(receipt["pinned_sdk_acceptance"], false);
            assert_eq!(receipt["restored_byte_for_byte"], true);
            assert_eq!(receipt["restored_control"]["exit"], 0);
            assert_eq!(
                receipt["boundary_probe_sha256"],
                digest(&root.join("tests/product-control/user-notice-forgery.lex.tex"))
            );
            let negatives = receipt["boundary_negatives"]
                .as_array()
                .expect("typed boundary negative");
            assert_eq!(negatives.len(), 1);
            assert!(negatives[0]["exit"].as_i64().is_some_and(|exit| exit != 0));
            assert_eq!(
                receipt["gate_sha256"],
                digest(&root.join("scripts/product-control-validation.py"))
            );
            assert_eq!(
                receipt["host_projection_sha256"],
                digest(&root.join("validation/product/host-contract.json"))
            );
            assert_eq!(
                receipt["host_projection_gate_sha256"],
                digest(&root.join("scripts/product-host-projection.py"))
            );
            let sources = receipt["source_sha256"]
                .as_object()
                .expect("source inventory");
            assert_eq!(sources.len(), 13);
            for (module, expected) in sources {
                assert_eq!(
                    expected,
                    &Value::String(digest(
                        &root.join(format!("src/Uorc/Product/{module}.lex.tex"))
                    ))
                );
            }
            let text = std::fs::read_to_string(root.join("src/Uorc/Product/Scenarios.lex.tex"))
                .expect("source-owned scenarios");
            let data = text
                .split_once("\\semanticdata{")
                .expect("semantic source")
                .1;
            let data = data
                .rsplit_once("}\n\\end{semanticmodule}")
                .expect("closed module")
                .0;
            let module: Value = serde_json::from_str(data).expect("semantic declarations");
            let declarations = module["declarations"].as_array().expect("declarations");
            let inventory: u64 = ["cases", "observations"]
                .into_iter()
                .map(|name| {
                    let declaration = declarations
                        .iter()
                        .find(|d| d["name"] == name)
                        .expect("runtime-owned inventory");
                    list_length(&declaration["body"])
                })
                .sum();
            assert!(inventory > 0);
            assert_eq!(receipt["runtime_total"].as_u64(), Some(inventory));
            assert_eq!(receipt["runtime_passed"].as_u64(), Some(inventory));
            let mutations = receipt["mutations"].as_array().expect("planted defects");
            assert_eq!(mutations.len(), 4);
            assert!(mutations.iter().all(|m| m["verification"]["exit"]
                .as_i64()
                .or_else(|| m["runtime"]["exit"].as_i64())
                .is_some_and(|exit| exit != 0)));
        });
    }

    fn check_open(id: &str) {
        check_receipt();
        let model = Model::load(&root().join("model")).expect("model loads");
        assert_eq!(
            model
                .ledger
                .claim
                .iter()
                .find(|claim| claim.id == id)
                .expect("registered obligation")
                .level,
            Level::Open
        );
    }

    #[test]
    fn component_control_receipt_is_source_bound_uc_ctl_01() {
        check_open("UC-CTL-01");
    }

    #[test]
    fn generated_view_value_receipt_is_source_bound_uc_viw_01() {
        check_open("UC-VIW-01");
    }

    #[test]
    fn public_request_component_receipt_is_source_bound_uc_api_01() {
        check_open("UC-API-01");
    }

    #[test]
    fn lifecycle_component_receipt_is_source_bound_uc_prod_01() {
        check_open("UC-PROD-01");
    }
    #[test]
    fn storage_component_receipt_is_source_bound_uc_stg_01() {
        check_open("UC-STG-01");
    }
}
