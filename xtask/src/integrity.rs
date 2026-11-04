//! Integrity oracle importer and failure-attribution harness tests.

#[test]
fn conformance_uc_sha_01() {
    let output = std::process::Command::new("node")
        .args([
            "--test",
            "--test-reporter=tap",
            "scripts/integrity-vectors.test.mjs",
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

#[test]
fn conformance_auth_nist_sha256_short() {
    let graph = repo_model::AuthorityGraph::load(&repo_model::repo_root()).unwrap();
    let model = graph.model();
    model.check().unwrap();
    assert!(model
        .authorities
        .authority
        .iter()
        .any(|a| a.id == "NIST-SHA256-SHORT"));
}

#[test]
fn conformance_auth_nist_sha256_long() {
    let graph = repo_model::AuthorityGraph::load(&repo_model::repo_root()).unwrap();
    let model = graph.model();
    model.check().unwrap();
    assert!(model
        .authorities
        .authority
        .iter()
        .any(|a| a.id == "NIST-SHA256-LONG"));
}

#[test]
fn conformance_uc_sha_02() {
    let output = std::process::Command::new("node")
        .args([
            "--test",
            "--test-reporter=tap",
            "scripts/integrity-stream.test.mjs",
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
    assert!(
        !stdout.contains("# tests 0"),
        "empty stream harness: {stdout}"
    );
}
