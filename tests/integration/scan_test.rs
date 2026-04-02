use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

#[test]
fn scan_finds_axios_vuln() {
    let mut cmd = Command::cargo_bin("dep-doctor").unwrap();
    cmd.args([
        "scan",
        fixtures_dir().join("npm-axios-vuln").to_str().unwrap(),
    ])
    .assert()
    .success()
    .stdout(contains("axios-csrf-ssrf-CVE-2023-45857"));
}

#[test]
fn scan_fixed_repo_has_no_built_in_axios_vuln() {
    // The fixture uses axios 1.8.4 which is beyond the affected range of the
    // built-in CSRF/SSRF problem. This test verifies the built-in problem is
    // not reported. The feed may still surface other advisories — that's correct
    // behaviour and is intentionally not asserted against here.
    let mut cmd = Command::cargo_bin("dep-doctor").unwrap();
    cmd.args([
        "scan",
        fixtures_dir().join("npm-axios-fixed").to_str().unwrap(),
    ])
    .assert()
    .success()
    .stdout(contains("axios-csrf-ssrf-CVE-2023-45857").not());
}

#[test]
fn scan_deep_finds_source_hits() {
    let mut cmd = Command::cargo_bin("dep-doctor").unwrap();
    cmd.args([
        "scan",
        fixtures_dir().join("npm-axios-vuln").to_str().unwrap(),
        "--deep",
    ])
    .assert()
    .success()
    .stdout(contains("server.js"));
}

#[test]
fn scan_python_requests_vuln() {
    let mut cmd = Command::cargo_bin("dep-doctor").unwrap();
    cmd.args([
        "scan",
        fixtures_dir().join("python-requests").to_str().unwrap(),
    ])
    .assert()
    .success()
    .stdout(contains("pip-requests-redirect-credential-leak"));
}

#[test]
fn problems_list_works() {
    let mut cmd = Command::cargo_bin("dep-doctor").unwrap();
    cmd.args(["problems", "list"])
        .assert()
        .success()
        .stdout(contains("axios"));
}

#[test]
fn problems_show_works() {
    let mut cmd = Command::cargo_bin("dep-doctor").unwrap();
    cmd.args(["problems", "show", "npm-axios-csrf-ssrf-CVE-2023-45857"])
        .assert()
        .success()
        .stdout(contains("withCredentials"));
}

#[test]
fn json_reporter_outputs_valid_json() {
    let mut cmd = Command::cargo_bin("dep-doctor").unwrap();
    let output = cmd
        .args([
            "scan",
            fixtures_dir().join("npm-axios-vuln").to_str().unwrap(),
            "--reporter",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let parsed: serde_json::Value =
        serde_json::from_slice(&output).expect("output should be valid JSON");
    assert!(parsed.is_object());
    assert!(parsed.get("findings").unwrap().is_array());
    assert!(parsed.get("typosquat_warnings").unwrap().is_array());
}
