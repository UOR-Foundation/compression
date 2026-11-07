//! Actual generated-model execution is required; a committed receipt cannot satisfy this test.
#[cfg(test)]
mod tests {
    #[test]
    fn generated_owned_machine_flow_uc_exe_01() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("repository parent");
        let result = std::process::Command::new("node")
            .arg(root.join("scripts/product-machine-runtime.mjs"))
            .current_dir(root)
            .status()
            .expect(
                "execute actual source linking, kernel compilation and generated-model controls",
            );
        assert!(
            result.success(),
            "the generated product-machine gate failed"
        );
    }

    #[test]
    fn generated_search_and_core_controls_execute() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("repository parent");
        let result = std::process::Command::new("node")
            .arg(root.join("scripts/executed-controls-runtime.mjs"))
            .current_dir(root)
            .status()
            .expect("execute concrete archive, replay, certification and step-machine controls");
        assert!(result.success(), "an executed control vector failed");
    }

    #[test]
    fn generated_payload_image_flow_uc_stg_01() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("repository parent");
        let result = std::process::Command::new("node")
            .arg(root.join("scripts/product-image-runtime.mjs"))
            .current_dir(root)
            .status()
            .expect("execute source payload admission, kernel compilation and actual lifecycle controls");
        assert!(result.success(), "the generated payload-image gate failed");
    }
    #[test]
    fn generated_staged_output_image_flow_uc_stg_01() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("repository parent");
        let result = std::process::Command::new("node")
            .arg(root.join("scripts/product-write-image-runtime.mjs"))
            .current_dir(root)
            .status()
            .expect(
                "execute actual bounded quote, staging, copy and publication transport controls",
            );
        assert!(result.success(), "the generated output-image gate failed");
    }

    #[test]
    fn generated_application_flow_uc_viw_01() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("repository parent");
        let result = std::process::Command::new("node")
            .arg(root.join("scripts/product-application-runtime.mjs"))
            .current_dir(root)
            .status()
            .expect(
                "execute actual generated application state, effects, callbacks and View controls",
            );
        assert!(result.success(), "the generated application gate failed");
    }
    #[test]
    fn generated_inspection_report_flow_uc_exe_01() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("repository parent");
        let result = std::process::Command::new("node")
            .arg(root.join("scripts/product-inspection-runtime.mjs"))
            .current_dir(root)
            .status()
            .expect("execute actual framing-to-inspection owner and negative controls");
        assert!(
            result.success(),
            "the generated inspection-report gate failed"
        );
    }
    #[test]
    fn generated_read_controller_flow_uc_stg_01() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("repository parent");
        let result = std::process::Command::new("node")
            .arg(root.join("scripts/product-read-controller-runtime.mjs"))
            .current_dir(root)
            .status()
            .expect("execute bounded read/image attachment and actual custody controls");
        assert!(
            result.success(),
            "the generated read-controller gate failed"
        );
    }

    #[test]
    fn generated_output_delivery_flow_uc_stg_01() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("repository parent");
        let result = std::process::Command::new("node")
            .arg(root.join("scripts/product-output-delivery-runtime.mjs"))
            .current_dir(root)
            .status()
            .expect("execute partial-write accounting and actual publication dispositions");
        assert!(
            result.success(),
            "the generated output-delivery gate failed"
        );
    }

    #[test]
    fn generated_charged_target_flow_uc_exe_01() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("repository parent");
        let result = std::process::Command::new("node")
            .arg(root.join("scripts/product-target-descriptor-runtime.mjs"))
            .current_dir(root)
            .status()
            .expect("execute charged target construction, ordered lengths and ownership controls");
        assert!(
            result.success(),
            "the generated target-descriptor gate failed"
        );
    }
}
