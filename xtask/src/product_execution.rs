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
}
