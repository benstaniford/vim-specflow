use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StepKind {
    Given,
    When,
    Then,
}

impl StepKind {
    pub fn from_attribute(name: &str) -> Option<StepKind> {
        match name {
            "Given" => Some(StepKind::Given),
            "When" => Some(StepKind::When),
            "Then" => Some(StepKind::Then),
            _ => None,
        }
    }

    pub fn from_keyword(word: &str) -> Option<StepKind> {
        match word {
            "Given" => Some(StepKind::Given),
            "When" => Some(StepKind::When),
            "Then" => Some(StepKind::Then),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            StepKind::Given => "Given",
            StepKind::When => "When",
            StepKind::Then => "Then",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Binding {
    pub kind: StepKind,
    /// The runtime regex pattern (after string-escape unescaping).
    pub pattern: String,
    pub file: PathBuf,
    /// 1-based line number where the attribute starts.
    pub line: usize,
}

/// Extract every Given/When/Then binding declared in a C# source file.
///
/// Tolerates both verbatim (`@"..."`) and regular (`"..."`) string forms,
/// `""`-escaped quotes inside verbatim strings, `\"` and `\\` inside regular
/// strings, and multiple attributes on the same line.
pub fn parse_bindings(source: &str, file: &Path) -> Vec<Binding> {
    let mut out = Vec::new();
    for (line_idx, line) in source.lines().enumerate() {
        scan_line(line, line_idx + 1, file, &mut out);
    }
    out
}

fn scan_line(line: &str, line_number: usize, file: &Path, out: &mut Vec<Binding>) {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'[' {
            i += 1;
            continue;
        }
        let attr_start = i + 1;
        let Some((kind, after_name)) = read_step_attribute_name(bytes, attr_start) else {
            i += 1;
            continue;
        };
        // Expect '(' immediately after the attribute name (possibly whitespace).
        let paren = skip_ws(bytes, after_name);
        if paren >= bytes.len() || bytes[paren] != b'(' {
            i += 1;
            continue;
        }
        let after_paren = skip_ws(bytes, paren + 1);
        let Some((pattern, after_string)) = read_string_literal(bytes, after_paren) else {
            i += 1;
            continue;
        };
        // Optionally tolerate a trailing `)]`; we don't require it because some
        // attributes have additional arguments we don't care about.
        out.push(Binding {
            kind,
            pattern,
            file: file.to_path_buf(),
            line: line_number,
        });
        i = after_string;
    }
}

fn skip_ws(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
        i += 1;
    }
    i
}

/// If `bytes[start..]` begins with `Given`/`When`/`Then` as a full word, return
/// the matched kind and the index just past it. Otherwise return None.
fn read_step_attribute_name(bytes: &[u8], start: usize) -> Option<(StepKind, usize)> {
    for (name, kind) in [
        ("Given", StepKind::Given),
        ("When", StepKind::When),
        ("Then", StepKind::Then),
    ] {
        let end = start + name.len();
        if end <= bytes.len() && &bytes[start..end] == name.as_bytes() {
            // The next char must not continue the identifier.
            let next = bytes.get(end).copied();
            let continues = matches!(next, Some(c) if c.is_ascii_alphanumeric() || c == b'_');
            if !continues {
                return Some((kind, end));
            }
        }
    }
    None
}

/// Read a C# string literal starting at `start`. Handles verbatim (`@"..."`)
/// and regular (`"..."`) forms. Returns the unescaped contents and the index
/// just past the closing quote.
fn read_string_literal(bytes: &[u8], start: usize) -> Option<(String, usize)> {
    if start >= bytes.len() {
        return None;
    }
    let (verbatim, mut i) = if bytes[start] == b'@' && bytes.get(start + 1) == Some(&b'"') {
        (true, start + 2)
    } else if bytes[start] == b'"' {
        (false, start + 1)
    } else {
        return None;
    };

    let mut s = String::new();
    while i < bytes.len() {
        let c = bytes[i];
        if verbatim {
            if c == b'"' {
                if bytes.get(i + 1) == Some(&b'"') {
                    s.push('"');
                    i += 2;
                    continue;
                }
                return Some((s, i + 1));
            }
            s.push(c as char);
            i += 1;
        } else {
            if c == b'\\' {
                match bytes.get(i + 1) {
                    Some(b'"') => {
                        s.push('"');
                        i += 2;
                    }
                    Some(b'\\') => {
                        s.push('\\');
                        i += 2;
                    }
                    Some(b'n') => {
                        s.push('\n');
                        i += 2;
                    }
                    Some(b't') => {
                        s.push('\t');
                        i += 2;
                    }
                    Some(&other) => {
                        s.push('\\');
                        s.push(other as char);
                        i += 2;
                    }
                    None => return None,
                }
                continue;
            }
            if c == b'"' {
                // Regular strings also allow `""` as a literal quote in some
                // dialects; the C# spec doesn't, but we accept it defensively.
                if bytes.get(i + 1) == Some(&b'"') {
                    s.push('"');
                    i += 2;
                    continue;
                }
                return Some((s, i + 1));
            }
            s.push(c as char);
            i += 1;
        }
    }
    None
}
