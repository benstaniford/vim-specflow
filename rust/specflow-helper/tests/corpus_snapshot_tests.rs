//! Optional regression test against a live SpecFlow corpus.
//!
//! Skipped by default because it depends on a path outside the repo. To run:
//!
//!   SPECFLOW_CORPUS_PATH=/path/to/your/tests \
//!     cargo test --release --test corpus_snapshot_tests -- --ignored --nocapture
//!
//! The test counts how many real `.feature` steps fail to resolve and asserts
//! the unbound rate stays under [`MAX_UNBOUND_RATIO`]. When it fails, the top
//! offending files are printed so a real regression is easy to spot.

use specflow_helper::{parse_feature_steps, Index};
use std::path::{Path, PathBuf};

/// Acceptable unbound rate. Adjust upward only with deliberate sign-off — a
/// jump here means the parser or matcher started missing something it used
/// to handle.
const MAX_UNBOUND_RATIO: f64 = 0.01;

fn walk_features(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for e in entries.flatten() {
        let p = e.path();
        let Ok(ft) = e.file_type() else { continue };
        if ft.is_dir() {
            let name = e.file_name();
            let name = name.to_string_lossy();
            if matches!(
                name.as_ref(),
                ".git" | "target" | "bin" | "obj" | "node_modules"
            ) || (name.starts_with('.') && name.as_ref() != ".")
            {
                continue;
            }
            walk_features(&p, out);
        } else if ft.is_file() && p.extension().and_then(|s| s.to_str()) == Some("feature") {
            out.push(p);
        }
    }
}

#[test]
#[ignore = "requires SPECFLOW_CORPUS_PATH to a live SpecFlow tree"]
fn live_corpus_unbound_rate_within_budget() {
    let Some(root) = std::env::var_os("SPECFLOW_CORPUS_PATH").map(PathBuf::from) else {
        eprintln!("SPECFLOW_CORPUS_PATH not set -- skipping");
        return;
    };
    assert!(
        root.is_dir(),
        "SPECFLOW_CORPUS_PATH does not point at a directory: {root:?}"
    );

    let idx = Index::build(&root, None).expect("build index");
    let query = idx.into_query();

    let mut features = Vec::new();
    walk_features(&root, &mut features);
    assert!(!features.is_empty(), "no .feature files found under {root:?}");

    let mut total_steps = 0usize;
    let mut unbound_total = 0usize;
    let mut per_file: Vec<(PathBuf, usize, usize)> = Vec::new();
    for f in &features {
        let src = std::fs::read_to_string(f).unwrap_or_default();
        let steps = parse_feature_steps(&src);
        let unbound = steps
            .iter()
            .filter(|s| query.find(&s.text, s.kind).is_none())
            .count();
        total_steps += steps.len();
        unbound_total += unbound;
        per_file.push((f.clone(), steps.len(), unbound));
    }

    let ratio = unbound_total as f64 / total_steps.max(1) as f64;
    eprintln!(
        "corpus: {features} files, {total_steps} steps, {unbound_total} unbound ({:.2}%)",
        100.0 * ratio,
        features = features.len(),
    );

    if ratio > MAX_UNBOUND_RATIO {
        per_file.sort_by_key(|(_, _, u)| std::cmp::Reverse(*u));
        eprintln!("\nTop unbound files:");
        for (f, n, u) in per_file.iter().take(15) {
            if *u == 0 {
                break;
            }
            eprintln!(
                "  {:>3}/{:<3} unbound  {}",
                u,
                n,
                f.strip_prefix(&root).unwrap_or(f).display()
            );
        }
        panic!(
            "unbound rate {:.4} exceeds budget {:.4}",
            ratio, MAX_UNBOUND_RATIO
        );
    }
}
