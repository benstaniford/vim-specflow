//! Real-corpus regression tests built from `BeyondTrust.Automation.Windows.Smoke/Smoke.feature`.
//!
//! Each test in this file targets a pattern shape that the previous VimScript
//! plugin got wrong (and therefore highlighted red in the editor). The
//! fixture is hermetic — bindings are copied verbatim from the live corpus
//! into `tests/fixtures/smoke_corpus/SmokeBindings.cs`, so this suite needs
//! no external paths.

use specflow_helper::{Index, StepKind};
use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/smoke_corpus")
}

fn feature_path() -> PathBuf {
    fixture_root().join("Smoke.feature")
}

fn build_index() -> specflow_helper::BindingIndex {
    Index::build(&fixture_root(), None)
        .expect("build smoke index")
        .into_query()
}

/// Walk a feature file the same way the production scanner will: track the
/// most recent concrete step kind so And/But inherit it, preserve trailing
/// colons on data-table steps.
fn iter_steps(src: &str) -> Vec<(usize, Option<StepKind>, String)> {
    let mut out = Vec::new();
    let mut last_kind: Option<StepKind> = None;
    for (i, line) in src.lines().enumerate() {
        let trimmed = line.trim_start();
        let (kind, rest) = if let Some(r) = trimmed.strip_prefix("Given ") {
            last_kind = Some(StepKind::Given);
            (Some(StepKind::Given), r)
        } else if let Some(r) = trimmed.strip_prefix("When ") {
            last_kind = Some(StepKind::When);
            (Some(StepKind::When), r)
        } else if let Some(r) = trimmed.strip_prefix("Then ") {
            last_kind = Some(StepKind::Then);
            (Some(StepKind::Then), r)
        } else if let Some(r) = trimmed.strip_prefix("And ") {
            (last_kind, r)
        } else if let Some(r) = trimmed.strip_prefix("But ") {
            (last_kind, r)
        } else {
            continue;
        };
        out.push((i + 1, kind, rest.trim_end().to_string()));
    }
    out
}

fn resolve<'a>(
    idx: &'a specflow_helper::BindingIndex,
    step: &str,
    kind: Option<StepKind>,
) -> &'a str {
    idx.find(step, kind)
        .unwrap_or_else(|| panic!("step did not resolve: {step:?}"))
        .binding
        .pattern
        .as_str()
}

#[test]
fn every_step_in_smoke_feature_resolves() {
    let idx = build_index();
    let src = std::fs::read_to_string(feature_path()).expect("read feature");
    let mut unbound = Vec::new();
    for (line, kind, step) in iter_steps(&src) {
        if idx.find(&step, kind).is_none() {
            unbound.push(format!("L{line} [{kind:?}] {step}"));
        }
    }
    assert!(
        unbound.is_empty(),
        "expected zero unbound steps, but found {}:\n  {}",
        unbound.len(),
        unbound.join("\n  ")
    );
}

// ---- Pattern regressions the old plugin gets wrong --------------------

#[test]
fn unquoted_dotstar_capture_matches_event_id() {
    // The old plugin can't match a binding with a non-quoted `(.*)` because
    // its normalization replaces the capture with PARAM but leaves the digit
    // in the step alone.
    let idx = build_index();
    let pat = resolve(
        &idx,
        "a local ECS event is created with EventId 116",
        Some(StepKind::Then),
    );
    assert_eq!(
        pat,
        "a local ECS event (is|is not|isn't) created with EventId (.*)"
    );
}

#[test]
fn unquoted_digit_capture_matches_count() {
    // `(\d+) local ECS event has been generated` -- old plugin collapses it
    // to "PARAM local ECS event has been generated" but compares against the
    // raw step "1 local ECS event has been generated", so it never matches.
    let idx = build_index();
    let pat = resolve(
        &idx,
        "1 local ECS event has been generated",
        Some(StepKind::Then),
    );
    assert_eq!(pat, r"(\d+) local ECS event has been generated");
}

#[test]
fn optional_trailing_group_as_the_user() {
    let idx = build_index();
    let pat_with = resolve(
        &idx,
        "I start the 'Avecto QA Test Windows Service' service as the user",
        Some(StepKind::When),
    );
    let pat_without = resolve(
        &idx,
        "I start the 'Avecto QA Test Windows Service' service",
        Some(StepKind::When),
    );
    assert_eq!(pat_with, "I start the '(.*)' service((?: as the user|))");
    assert_eq!(pat_with, pat_without, "both forms must hit the same binding");
}

#[test]
fn optional_trailing_group_from_the_original_folder() {
    let idx = build_index();
    let pat = resolve(
        &idx,
        "I run 'DummyVbsScript.vbs' within 'Resources' using 'Run with Defendpoint'",
        Some(StepKind::Given),
    );
    assert_eq!(
        pat,
        "I run '(.*)' within '(.*)' using '(.*)'((?: from the original folder|))"
    );
}

#[test]
fn verbatim_double_quote_captures_match_double_quoted_step() {
    let idx = build_index();
    let pat = resolve(
        &idx,
        r#"I take ownership of "\Resources\Testapplication.exe" as "BUILTIN\Administrators""#,
        Some(StepKind::Given),
    );
    assert_eq!(pat, r#"I take ownership of "([^"]*)" as "(.*)""#);
}

#[test]
fn data_table_step_with_trailing_colon() {
    // The step text in the .feature ends with `:` because a data table
    // follows. The binding pattern includes the colon too. The old plugin's
    // step extractor drops it, breaking the match.
    let idx = build_index();
    let pat = resolve(
        &idx,
        "I run 'regedit.exe' with the following:",
        Some(StepKind::When),
    );
    assert_eq!(pat, "I run '([^']*)' with the following:");
}

#[test]
fn quoted_alternation_with_compound_negation() {
    // 'is'/'is not'/'isn't' alternation inside quotes. The old plugin
    // normalizes BOTH the step and the binding to PARAM so this happens to
    // match; the regression here is that the binding has a SECOND alternation
    // with apostrophes (isn't) which the normalizer treats inconsistently.
    let idx = build_index();
    let pat = resolve(
        &idx,
        "the 'Avecto QA Test Windows Service' service status 'is' 'Running'",
        Some(StepKind::Then),
    );
    assert_eq!(pat, "the '(.*)' service status '(is|is not|isn't)' '(.*)'");
}

#[test]
fn and_step_inherits_preceding_then_kind() {
    // Line 14: `And a local ECS event is created with EventId 116` follows a
    // `Then`. SpecFlow inherits the previous concrete kind; this test confirms
    // that the matcher with `kind=Some(Then)` finds the binding.
    let idx = build_index();
    assert!(idx
        .find(
            "a local ECS event is created with EventId 116",
            Some(StepKind::Then),
        )
        .is_some());
}

#[test]
fn when_and_then_bindings_with_same_pattern_both_match() {
    // FileBindings.cs declares the same pattern as both [When] and [Then]
    // on consecutive lines. Make sure both kinds resolve.
    let idx = build_index();
    assert!(idx
        .find(
            "the file 'C:\\Program Files\\Avecto\\Privilege Guard Client\\DefendpointService.exe' 'does' exist",
            Some(StepKind::When),
        )
        .is_some());
    assert!(idx
        .find(
            "the file 'C:\\Program Files\\Avecto\\Privilege Guard Client\\PGSystemTray.exe' 'does' exist",
            Some(StepKind::Then),
        )
        .is_some());
}

#[test]
fn anchoring_rejects_substring_step_for_bare_run() {
    // `I run '([^']*)'` is anchored; a longer step like
    // `I run '...' with the following:` must NOT match the bare-run binding.
    let idx = build_index();
    let pat = resolve(
        &idx,
        "I run 'regedit.exe' with the following:",
        Some(StepKind::When),
    );
    assert_ne!(pat, "I run '([^']*)'", "bare-run pattern should not absorb data-table step");
}

#[test]
fn process_state_matches_via_unquoted_capture() {
    // `the process 'X' with window 'Y' (.*) running` — the state ("is"/"is not"/etc.)
    // is captured unquoted by `(.*)`. Old plugin fails because PARAM collapse
    // doesn't apply to unquoted slots.
    let idx = build_index();
    let pat = resolve(
        &idx,
        "the process 'calc' with window 'Calculator' is not running",
        Some(StepKind::Then),
    );
    assert_eq!(pat, "the process '([^']*)' with window '([^']*)' (.*) running");
}

#[test]
fn negation_alternation_distinguishes_running_from_starting() {
    // Two near-identical bindings:
    //   the process '...' (is|is not|isn't) running    -> ProcessRunning
    //   the process (has started|does not start|...)   -> ProcessStart
    // The first matches "the process 'regedit' is running", the second
    // matches "the process does not start" -- without a quoted process name.
    let idx = build_index();
    let running = resolve(&idx, "the process 'regedit' is running", Some(StepKind::Then));
    let starting = resolve(&idx, "the process does not start", Some(StepKind::Then));
    assert_eq!(running, "the process '([^']*)' (is|is not|isn't) running");
    assert_eq!(
        starting,
        "the process (has started|does not start|is still waiting for our messagehost)"
    );
}
