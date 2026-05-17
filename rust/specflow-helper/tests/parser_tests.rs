use specflow_helper::{parse_bindings, StepKind};
use std::path::PathBuf;

fn fixture(name: &str) -> (String, PathBuf) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let source = std::fs::read_to_string(&path).expect("fixture file should exist");
    (source, path)
}

#[test]
fn extracts_every_binding_from_gnarly_fixture() {
    let (src, path) = fixture("gnarly_bindings.cs");
    let bindings = parse_bindings(&src, &path);
    // 13 attributes total (one method carries two [Given(...)] attributes).
    assert_eq!(bindings.len(), 13, "found bindings: {bindings:#?}");
}

#[test]
fn assigns_correct_step_kinds() {
    let (src, path) = fixture("gnarly_bindings.cs");
    let bindings = parse_bindings(&src, &path);
    let givens = bindings.iter().filter(|b| b.kind == StepKind::Given).count();
    let whens = bindings.iter().filter(|b| b.kind == StepKind::When).count();
    let thens = bindings.iter().filter(|b| b.kind == StepKind::Then).count();
    assert_eq!(givens, 11);
    assert_eq!(whens, 1);
    assert_eq!(thens, 1);
}

#[test]
fn unescapes_verbatim_double_quotes() {
    let (src, path) = fixture("gnarly_bindings.cs");
    let bindings = parse_bindings(&src, &path);
    let access = bindings
        .iter()
        .find(|b| b.pattern.contains("has an access level"))
        .expect("access-level binding should be parsed");
    // The C# source has `""(.*)""` which is the verbatim escape for `"(.*)"`.
    assert_eq!(
        access.pattern,
        r#""(.*)" has an access level of "(.*)" for "(.*)""#
    );
}

#[test]
fn parses_non_verbatim_string_form() {
    let (src, path) = fixture("gnarly_bindings.cs");
    let bindings = parse_bindings(&src, &path);
    let groups = bindings
        .iter()
        .find(|b| b.pattern.starts_with("groups from"))
        .expect("non-verbatim binding should be parsed");
    assert_eq!(
        groups.pattern,
        "groups from '(identity|pmc)' '(are|are not)' available for '(admin|user)'"
    );
}

#[test]
fn captures_both_attributes_on_overloaded_method() {
    let (src, path) = fixture("gnarly_bindings.cs");
    let bindings = parse_bindings(&src, &path);
    let jit: Vec<_> = bindings
        .iter()
        .filter(|b| b.pattern.contains("JIT Admin session with reason"))
        .collect();
    assert_eq!(jit.len(), 2);
    // Ensure they're on distinct lines.
    assert_ne!(jit[0].line, jit[1].line);
}

#[test]
fn records_line_numbers_for_jumps() {
    let (src, path) = fixture("gnarly_bindings.cs");
    let bindings = parse_bindings(&src, &path);
    let caps = bindings
        .iter()
        .find(|b| b.pattern == "caps lock is enabled")
        .expect("caps lock binding should be parsed");
    // The fixture places this binding on line 10 (1-based). Update if you
    // shuffle the fixture.
    let actual_line = src
        .lines()
        .position(|l| l.contains(r#"[Given(@"caps lock is enabled")]"#))
        .map(|i| i + 1)
        .expect("binding line should exist");
    assert_eq!(caps.line, actual_line);
}

#[test]
fn ignores_unrelated_attributes() {
    let src = r#"
        [Scope(Tag = "ignored")]
        [TestMethod]
        [GivenButNotReally("nope")]
        public void Foo() { }
    "#;
    let bindings = parse_bindings(src, &PathBuf::from("inline.cs"));
    assert!(bindings.is_empty(), "spurious bindings: {bindings:?}");
}

#[test]
fn handles_multiple_attributes_on_one_line() {
    // Sometimes attributes share a line; both should be picked up.
    let src = r#"        [Given(@"thing one")] [Scope(Tag = "x")] [Given(@"thing two")]"#;
    let bindings = parse_bindings(src, &PathBuf::from("inline.cs"));
    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0].pattern, "thing one");
    assert_eq!(bindings[1].pattern, "thing two");
    assert_eq!(bindings[0].line, 1);
    assert_eq!(bindings[1].line, 1);
}
