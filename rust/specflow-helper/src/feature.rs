use crate::parser::StepKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureStep {
    /// 1-based line number in the feature file.
    pub line: usize,
    /// Concrete kind (Given/When/Then) — for And/But this is inherited from
    /// the most recent concrete step in the same scenario/background block.
    /// `None` only when an And/But appears with no preceding concrete step
    /// (a malformed feature file).
    pub kind: Option<StepKind>,
    /// The raw keyword the line actually starts with: "Given", "When",
    /// "Then", "And", or "But".
    pub keyword: String,
    /// Step text with the keyword stripped and trailing whitespace trimmed.
    /// The trailing colon for data-table steps is preserved.
    pub text: String,
}

/// Parse every step line out of a .feature file.
///
/// Inheritance: scenario and background openers reset the inherited kind so
/// And/But from a previous scenario don't leak into the next one. Other
/// non-step lines (tags, comments, table rows, blank lines, feature/example
/// keywords) are skipped.
pub fn parse_feature_steps(source: &str) -> Vec<FeatureStep> {
    let mut out = Vec::new();
    let mut inherited: Option<StepKind> = None;
    for (i, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();
        // Reset inheritance at scenario boundaries.
        if is_scenario_boundary(trimmed) {
            inherited = None;
            continue;
        }
        let (keyword, rest, kind, is_inherit) = if let Some(r) = trimmed.strip_prefix("Given ") {
            ("Given", r, Some(StepKind::Given), false)
        } else if let Some(r) = trimmed.strip_prefix("When ") {
            ("When", r, Some(StepKind::When), false)
        } else if let Some(r) = trimmed.strip_prefix("Then ") {
            ("Then", r, Some(StepKind::Then), false)
        } else if let Some(r) = trimmed.strip_prefix("And ") {
            ("And", r, inherited, true)
        } else if let Some(r) = trimmed.strip_prefix("But ") {
            ("But", r, inherited, true)
        } else {
            continue;
        };
        if !is_inherit {
            inherited = kind;
        }
        out.push(FeatureStep {
            line: i + 1,
            kind,
            keyword: keyword.to_string(),
            text: rest.trim_end().to_string(),
        });
    }
    out
}

fn is_scenario_boundary(trimmed: &str) -> bool {
    // Match Gherkin section openers (case-sensitive — matches SpecFlow).
    for prefix in [
        "Scenario:",
        "Scenario Outline:",
        "Scenario Template:",
        "Background:",
        "Examples:",
        "Rule:",
        "Feature:",
    ] {
        if trimmed.starts_with(prefix) {
            return true;
        }
    }
    false
}
