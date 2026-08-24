use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn version_supports_machine_readable_output() {
    let mut command = Command::cargo_bin("clutter").unwrap();
    command
        .args(["version", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"schema\":\"clutter.version/v1\"",
        ));
}

#[test]
fn trace_rejects_unknown_schema() {
    let mut command = Command::cargo_bin("clutter").unwrap();
    command
        .args(["trace", "/dev/null"])
        .assert()
        .code(5)
        .stderr(predicate::str::contains("analysis failed"));
}

#[test]
fn trace_snapshot_hash_mismatch_fails_closed() {
    // A trace from a different snapshot must not refine this subject.
    let trace = r#"{
        "schema": "clutter.runtime-trace/v1",
        "abi": "arm64-v8a",
        "snapshot_hash": "deadbeefdeadbeefdeadbeefdeadbeef",
        "executed_pcs": []
    }"#;
    let temp = std::env::temp_dir().join("clutter-lab-trace.json");
    std::fs::write(&temp, trace).unwrap();
    let mut command = Command::cargo_bin("clutter").unwrap();
    command
        .args([
            "trace",
            temp.to_str().unwrap(),
            "--snapshot-hash",
            "00000000000000000000000000000000",
        ])
        .assert()
        .code(5)
        .stderr(predicate::str::contains("does not match expected"));
}

#[test]
fn invalid_scope_is_a_usage_error() {
    let mut command = Command::cargo_bin("clutter").unwrap();
    command
        .args([
            "decompile",
            "missing.apk",
            "--out",
            "unused",
            "--scope",
            "invalid",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("unknown scope"));
}

#[test]
fn missing_input_has_a_stable_io_exit_code() {
    let mut command = Command::cargo_bin("clutter").unwrap();
    command
        .args(["inspect", "definitely-not-present.apk"])
        .assert()
        .code(6)
        .stderr(predicate::str::contains("I/O error"));
}

#[test]
fn decompile_help_documents_split_debug_support() {
    let mut command = Command::cargo_bin("clutter").unwrap();
    command
        .args(["decompile", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--symbols <PATH>"))
        .stdout(predicate::str::contains("--obfuscation-map <JSON>"))
        .stdout(predicate::str::contains("--vm-oracle <JSON>"))
        .stdout(predicate::str::contains("--cross-abi"))
        .stdout(predicate::str::contains("split-debug-info"));
}

#[test]
fn vm_oracle_help_documents_analyzer_and_android_execution() {
    let mut command = Command::cargo_bin("clutter").unwrap();
    command
        .args(["vm-oracle", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--analyzer <PATH>"))
        .stdout(predicate::str::contains("--adb <SERIAL>"))
        .stdout(predicate::str::contains("--abi <ABI>"));
}
