//! Reporting conformance. Real runtime probes remain mandatory in `just vv`.

fn run_reporting_tests() {
    let root = repo_model::repo_root();
    let output = std::process::Command::new("node")
        .args([
            "--test",
            "--test-reporter=tap",
            "scripts/capability-probes.test.mjs",
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
fn executable_sdk_capability_reporting_uc_sdk_01() {
    run_reporting_tests();
}

#[test]
fn observed_binding_report_anti_vacuity_uc_sdk_02() {
    run_reporting_tests();
}
