//! Run the core mutation harness's own failure-attribution and anti-vacuity tests.

#[test]
fn varint_validation_harness_is_executed_without_filters() {
    let output = std::process::Command::new("node")
        .args([
            "--test",
            "--test-reporter=tap",
            "scripts/varint-validation.test.mjs",
        ])
        .current_dir(repo_model::repo_root())
        .output()
        .expect("the locked SDK supplies Node");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stdout}\n{stderr}");
    for summary in ["# fail 0", "# skipped 0", "# todo 0"] {
        assert!(stdout.contains(summary), "missing {summary}: {stdout}");
    }
    assert!(!stdout.contains("# tests 0"), "empty harness: {stdout}");
}
