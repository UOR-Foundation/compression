//! Actual generated-model execution is required; a committed receipt cannot satisfy this test.
#[cfg(test)]
mod tests {
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
}
