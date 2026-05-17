use crate::parser::{Binding, StepKind};
use regex::Regex;

/// A compiled set of bindings ready for step lookup.
///
/// Patterns that fail to compile are retained but skipped during matching;
/// the public [`compile_errors`](Self::compile_errors) accessor lets callers
/// surface them.
pub struct BindingIndex {
    entries: Vec<Entry>,
    errors: Vec<CompileError>,
}

struct Entry {
    binding: Binding,
    compiled: Option<Regex>,
}

#[derive(Debug, Clone)]
pub struct CompileError {
    pub pattern: String,
    pub kind: StepKind,
    pub file: std::path::PathBuf,
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Clone, Copy)]
pub struct MatchResult<'a> {
    pub binding: &'a Binding,
}

impl BindingIndex {
    pub fn build(bindings: Vec<Binding>) -> Self {
        let mut entries = Vec::with_capacity(bindings.len());
        let mut errors = Vec::new();
        for binding in bindings {
            let anchored = anchor(&binding.pattern);
            match Regex::new(&anchored) {
                Ok(re) => entries.push(Entry {
                    binding,
                    compiled: Some(re),
                }),
                Err(e) => {
                    errors.push(CompileError {
                        pattern: binding.pattern.clone(),
                        kind: binding.kind,
                        file: binding.file.clone(),
                        line: binding.line,
                        message: e.to_string(),
                    });
                    entries.push(Entry {
                        binding,
                        compiled: None,
                    });
                }
            }
        }
        BindingIndex { entries, errors }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn compile_errors(&self) -> &[CompileError] {
        &self.errors
    }

    /// Find the first binding whose pattern matches `step`. When `kind` is
    /// `None` (And/But, or unknown), any binding kind is eligible.
    pub fn find(&self, step: &str, kind: Option<StepKind>) -> Option<MatchResult<'_>> {
        for entry in &self.entries {
            if let Some(want) = kind {
                if entry.binding.kind != want {
                    continue;
                }
            }
            if let Some(re) = &entry.compiled {
                if re.is_match(step) {
                    return Some(MatchResult {
                        binding: &entry.binding,
                    });
                }
            }
        }
        None
    }

    pub fn bindings(&self) -> impl Iterator<Item = &Binding> {
        self.entries.iter().map(|e| &e.binding)
    }
}

fn anchor(pattern: &str) -> String {
    let has_start = pattern.starts_with('^');
    let has_end = pattern.ends_with('$') && !pattern.ends_with("\\$");
    match (has_start, has_end) {
        (true, true) => pattern.to_string(),
        (true, false) => format!("{pattern}$"),
        (false, true) => format!("^{pattern}"),
        (false, false) => format!("^{pattern}$"),
    }
}
