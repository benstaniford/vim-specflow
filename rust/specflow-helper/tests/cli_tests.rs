//! End-to-end tests for the `specflow-helper` binary.
//!
//! Each test invokes the actual compiled binary via `Command`. Cargo sets
//! `CARGO_BIN_EXE_specflow-helper` for integration tests, so we don't have
//! to guess the path.

use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_specflow-helper"))
}

fn smoke_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/smoke_corpus")
}

fn run(args: &[&str]) -> (String, String, i32) {
    let output = Command::new(bin())
        .args(args)
        .output()
        .expect("failed to invoke specflow-helper");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let code = output.status.code().unwrap_or(-1);
    (stdout, stderr, code)
}

#[test]
fn version_flag_prints_version() {
    let (stdout, _, code) = run(&["--version"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("specflow-helper"));
}

#[test]
fn help_flag_prints_usage() {
    let (stdout, _, code) = run(&["--help"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("Subcommands"));
    assert!(stdout.contains("resolve"));
    assert!(stdout.contains("scan"));
}

#[test]
fn resolve_hits_existing_binding() {
    let root = smoke_root();
    let (stdout, stderr, code) = run(&[
        "resolve",
        "--root",
        root.to_str().unwrap(),
        "--no-cache",
        "--kind",
        "Then",
        "--step",
        "a local ECS event is created with EventId 116",
    ]);
    assert_eq!(code, 0, "stderr: {stderr}");
    let v: Value = serde_json::from_str(stdout.trim()).expect("parse json");
    assert_eq!(v["kind"], "Then");
    assert!(v["file"].as_str().unwrap().ends_with("SmokeBindings.cs"));
    assert_eq!(
        v["pattern"],
        "a local ECS event (is|is not|isn't) created with EventId (.*)"
    );
    assert!(v["line"].as_u64().unwrap() > 0);
}

#[test]
fn resolve_returns_empty_object_on_miss() {
    let root = smoke_root();
    let (stdout, _, code) = run(&[
        "resolve",
        "--root",
        root.to_str().unwrap(),
        "--no-cache",
        "--kind",
        "Given",
        "--step",
        "this step definitely does not exist anywhere",
    ]);
    assert_eq!(code, 0);
    let v: Value = serde_json::from_str(stdout.trim()).expect("parse json");
    assert!(v.as_object().unwrap().is_empty(), "expected {{}}, got {v}");
}

#[test]
fn resolve_and_kind_falls_back_to_any() {
    // --kind And tells the binary "this is an And/But step, search all kinds."
    let root = smoke_root();
    let (stdout, stderr, code) = run(&[
        "resolve",
        "--root",
        root.to_str().unwrap(),
        "--no-cache",
        "--kind",
        "And",
        "--step",
        "a local ECS event is created with EventId 42",
    ]);
    assert_eq!(code, 0, "stderr: {stderr}");
    let v: Value = serde_json::from_str(stdout.trim()).expect("parse json");
    assert_eq!(v["kind"], "Then", "should fall through to Then binding");
}

#[test]
fn resolve_missing_root_errors() {
    let (_, stderr, code) = run(&["resolve", "--step", "x"]);
    assert_ne!(code, 0);
    assert!(stderr.contains("--root"));
}

#[test]
fn scan_returns_steps_and_resolutions_for_smoke_feature() {
    let root = smoke_root();
    let feature = root.join("Smoke.feature");
    let (stdout, stderr, code) = run(&[
        "scan",
        "--root",
        root.to_str().unwrap(),
        "--no-cache",
        "--feature",
        feature.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "stderr: {stderr}");
    let v: Value = serde_json::from_str(stdout.trim()).expect("parse json");

    let steps = v["steps"].as_array().expect("steps is array");
    assert_eq!(steps.len(), 47, "smoke feature has 47 step lines");

    let unbound: Vec<&Value> = steps.iter().filter(|s| s["resolved"].is_null()).collect();
    assert!(
        unbound.is_empty(),
        "expected zero unbound, got: {unbound:?}"
    );

    // Every step carries a 1-based line number and the keyword it appeared with.
    for s in steps {
        assert!(s["line"].as_u64().unwrap() >= 1);
        let kw = s["keyword"].as_str().unwrap();
        assert!(
            matches!(kw, "Given" | "When" | "Then" | "And" | "But"),
            "bad keyword: {kw}"
        );
    }

    // Stats accompany the result.
    let stats = &v["stats"];
    assert!(stats["bindings"].as_u64().unwrap() > 0);
}

#[test]
fn scan_marks_unknown_steps_resolved_null() {
    // Synthesize a feature file with one bogus step and confirm it shows up
    // as unresolved in the output.
    let tmp = std::env::temp_dir().join(format!(
        "specflow-cli-test-{}.feature",
        std::process::id()
    ));
    std::fs::write(
        &tmp,
        "Feature: bogus\n  Scenario: x\n    Given a step that no binding covers\n",
    )
    .unwrap();
    let root = smoke_root();
    let (stdout, _, code) = run(&[
        "scan",
        "--root",
        root.to_str().unwrap(),
        "--no-cache",
        "--feature",
        tmp.to_str().unwrap(),
    ]);
    let _ = std::fs::remove_file(&tmp);
    assert_eq!(code, 0);
    let v: Value = serde_json::from_str(stdout.trim()).unwrap();
    let steps = v["steps"].as_array().unwrap();
    assert_eq!(steps.len(), 1);
    assert!(steps[0]["resolved"].is_null());
    assert_eq!(steps[0]["text"], "a step that no binding covers");
}

#[test]
fn list_emits_one_line_per_binding() {
    let root = smoke_root();
    let (stdout, stderr, code) = run(&[
        "list",
        "--root",
        root.to_str().unwrap(),
        "--no-cache",
    ]);
    assert_eq!(code, 0, "stderr: {stderr}");
    let lines: Vec<&str> = stdout.lines().collect();
    // SmokeBindings.cs declares 28 bindings (the [Given]/[When]/[Then]
    // attributes — note FileExistence and IRunWithTable each carry two).
    assert!(
        lines.len() >= 25,
        "expected ~28 bindings, got {}",
        lines.len()
    );
    // Format: `[Kind] pattern \t file:line`
    for line in &lines {
        assert!(
            line.starts_with("[Given]") || line.starts_with("[When]") || line.starts_with("[Then]"),
            "unexpected line: {line:?}"
        );
        let parts: Vec<&str> = line.split('\t').collect();
        assert_eq!(parts.len(), 2, "expected one tab separator: {line:?}");
        assert!(
            parts[1].contains(':'),
            "expected file:line in trailing field: {line:?}"
        );
    }
}

#[test]
fn unknown_subcommand_exits_nonzero() {
    let (_, stderr, code) = run(&["wibble"]);
    assert_ne!(code, 0);
    assert!(stderr.contains("unknown subcommand"));
}
