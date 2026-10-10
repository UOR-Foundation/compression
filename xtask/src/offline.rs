//! Executed tests for the source-owned reproducibility scenarios.

fn run_validation_tests() {
    let root = repo_model::repo_root();
    let output = std::process::Command::new("node")
        .args([
            "--test",
            "--test-reporter=tap",
            "scripts/offline-validation.test.mjs",
        ])
        .current_dir(root)
        .output()
        .expect("the locked SDK supplies Node");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stdout.contains("# skipped 0"), "skipped test: {stdout}");
    assert!(stdout.contains("# todo 0"), "pending test: {stdout}");
}

#[test]
fn clean_root_artifact_identity_uc_rep_01() {
    run_validation_tests();
}

#[test]
fn dependency_and_host_fallback_negatives_uc_rep_02() {
    run_validation_tests();
}
