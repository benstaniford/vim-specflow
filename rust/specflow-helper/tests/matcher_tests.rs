use specflow_helper::{parse_bindings, BindingIndex, StepKind};
use std::path::PathBuf;

fn build_index() -> BindingIndex {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/gnarly_bindings.cs");
    let src = std::fs::read_to_string(&path).expect("fixture file should exist");
    BindingIndex::build(parse_bindings(&src, &path))
}

fn assert_match(idx: &BindingIndex, step: &str, kind: Option<StepKind>, expected_pattern: &str) {
    let m = idx
        .find(step, kind)
        .unwrap_or_else(|| panic!("expected match for: {step:?}"));
    assert_eq!(
        m.binding.pattern, expected_pattern,
        "step {step:?} matched wrong binding"
    );
}

fn assert_no_match(idx: &BindingIndex, step: &str, kind: Option<StepKind>) {
    if let Some(m) = idx.find(step, kind) {
        panic!(
            "expected no match for {step:?}, but matched: {:?}",
            m.binding.pattern
        );
    }
}

#[test]
fn no_pattern_compilation_errors_on_real_bindings() {
    let idx = build_index();
    let errs = idx.compile_errors();
    assert!(
        errs.is_empty(),
        "expected all real-world patterns to compile, got: {errs:#?}"
    );
}

#[test]
fn matches_plain_literal() {
    let idx = build_index();
    assert_match(
        &idx,
        "caps lock is enabled",
        Some(StepKind::Given),
        "caps lock is enabled",
    );
}

#[test]
fn quoted_capture_matches_any_content() {
    let idx = build_index();
    assert_match(
        &idx,
        "a reader returns an error 'connection refused'",
        Some(StepKind::Given),
        "a reader returns an error '(.*)'",
    );
}

#[test]
fn numeric_capture_rejects_non_numeric() {
    let idx = build_index();
    assert_match(
        &idx,
        "a time-limited approved JIT App Access request message for 5 days is displayed",
        Some(StepKind::Given),
        r"a time-limited approved JIT App Access request message for (\d+) days is displayed",
    );
    // Crucially, this should NOT match the numeric binding — \d+ rejects "five".
    assert_no_match(
        &idx,
        "a time-limited approved JIT App Access request message for five days is displayed",
        Some(StepKind::Given),
    );
}

#[test]
fn alternation_picks_only_listed_options() {
    let idx = build_index();
    assert_match(
        &idx,
        "a 'approved' JIT App Access request message is displayed",
        Some(StepKind::Given),
        "a '(approved|pending|denied)' JIT App Access request message is displayed",
    );
    // "queued" is not in the alternation — old plugin would have matched (false
    // positive) because it collapses every paren group to PARAM.
    assert_no_match(
        &idx,
        "a 'queued' JIT App Access request message is displayed",
        Some(StepKind::Given),
    );
}

#[test]
fn optional_trailing_group_matches_with_and_without() {
    let idx = build_index();
    let pattern = "I attempt to stop the '(.*)' service((?: as the user|))";
    assert_match(
        &idx,
        "I attempt to stop the 'SampleService' service",
        Some(StepKind::Given),
        pattern,
    );
    assert_match(
        &idx,
        "I attempt to stop the 'SampleService' service as the user",
        Some(StepKind::Given),
        pattern,
    );
}

#[test]
fn verbatim_escaped_quotes_match_double_quoted_step_text() {
    let idx = build_index();
    assert_match(
        &idx,
        r#""bob" has an access level of "admin" for "C:\\Windows""#,
        Some(StepKind::Given),
        r#""(.*)" has an access level of "(.*)" for "(.*)""#,
    );
}

#[test]
fn non_verbatim_form_matches_real_step() {
    let idx = build_index();
    assert_match(
        &idx,
        "groups from 'local' 'are' available for 'admin'",
        Some(StepKind::Given),
        "groups from '(local|remote)' '(are|are not)' available for '(admin|user)'",
    );
}

#[test]
fn named_capture_with_char_classes_matches_url() {
    let idx = build_index();
    assert_match(
        &idx,
        "I build the repository 'https://github.com/example-org/sample-app.git' using Microsoft Build",
        Some(StepKind::Given),
        r"I build the repository '(https://github.com/[\w,\-,_]+/(?<repo>[\w,\-,_]+)\.git)' using Microsoft Build",
    );
}

#[test]
fn and_step_matches_any_kind() {
    let idx = build_index();
    // The Then-binding has the kind "Then", but an And/But step has no kind.
    assert_match(
        &idx,
        "I delete the 'HKLM\\Software\\Foo' key from 'Bar' in the 'HKLM' hive and '64-bit' view",
        None,
        "I ((?:fail to |))delete the '(.*)' key from '(.*)' in the '(.*)' hive and '(.*)' view",
    );
}

#[test]
fn anchors_pattern_to_avoid_partial_matches() {
    let idx = build_index();
    // "caps lock is enabled" is a literal binding; a longer step that merely
    // contains it must NOT match.
    assert_no_match(
        &idx,
        "verify caps lock is enabled and the LED glows",
        Some(StepKind::Given),
    );
}

#[test]
fn step_type_filters_out_wrong_kind() {
    let idx = build_index();
    // The 'caps lock is enabled' binding is Given. A When step with identical
    // text should not match when kind=Some(When).
    assert_no_match(&idx, "caps lock is enabled", Some(StepKind::When));
}
