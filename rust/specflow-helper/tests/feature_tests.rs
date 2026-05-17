use specflow_helper::{parse_feature_steps, StepKind};

#[test]
fn extracts_basic_steps_with_kinds() {
    let src = "\
Feature: x
  Scenario: one
    Given a precondition
    When I act
    Then I observe
";
    let steps = parse_feature_steps(src);
    assert_eq!(steps.len(), 3);
    assert_eq!(steps[0].kind, Some(StepKind::Given));
    assert_eq!(steps[1].kind, Some(StepKind::When));
    assert_eq!(steps[2].kind, Some(StepKind::Then));
    assert_eq!(steps[0].text, "a precondition");
    assert_eq!(steps[0].keyword, "Given");
    assert_eq!(steps[0].line, 3);
}

#[test]
fn and_inherits_the_preceding_kind() {
    let src = "\
Scenario: x
  Given a thing
  And another thing
  Then result
  And another result
  But not this
";
    let steps = parse_feature_steps(src);
    assert_eq!(steps.len(), 5);
    assert_eq!(steps[1].kind, Some(StepKind::Given)); // And after Given
    assert_eq!(steps[2].kind, Some(StepKind::Then));
    assert_eq!(steps[3].kind, Some(StepKind::Then)); // And after Then
    assert_eq!(steps[4].kind, Some(StepKind::Then)); // But after Then
}

#[test]
fn inheritance_resets_between_scenarios() {
    let src = "\
Scenario: first
  Given a thing
Scenario: second
  And another thing
";
    let steps = parse_feature_steps(src);
    assert_eq!(steps.len(), 2);
    assert_eq!(steps[0].kind, Some(StepKind::Given));
    // The And in the second scenario has nothing to inherit from.
    assert_eq!(steps[1].kind, None);
}

#[test]
fn preserves_trailing_colon_on_data_table_steps() {
    let src = "\
Scenario: x
  When I run 'foo' with the following:
    | a | b |
    | 1 | 2 |
";
    let steps = parse_feature_steps(src);
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].text, "I run 'foo' with the following:");
}

#[test]
fn skips_tags_comments_blanks_and_tables() {
    let src = "\
@tag1 @tag2
Feature: x

  # comment
  @scenarioTag
  Scenario: x
    Given a thing
    | col1 | col2 |
    | 1    | 2    |
";
    let steps = parse_feature_steps(src);
    assert_eq!(steps.len(), 1, "got: {steps:?}");
    assert_eq!(steps[0].text, "a thing");
}

#[test]
fn handles_indented_scenarios_and_background() {
    let src = "\
Feature: x
  Background:
    Given background step
  Scenario: x
    Given scenario step
    And another
";
    let steps = parse_feature_steps(src);
    assert_eq!(steps.len(), 3);
    assert_eq!(steps[0].text, "background step");
    assert_eq!(steps[1].text, "scenario step");
    assert_eq!(steps[2].kind, Some(StepKind::Given)); // And inherits from scenario-step Given
}
