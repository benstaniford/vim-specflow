//! Diagnostic + benchmark against a real SpecFlow tree.
//!
//! Run: cargo run --release --example corpus_smoke -- <root>
//!
//! - Times a full cold build (no cache) and a warm rebuild (cache populated).
//! - Reports compile errors so we can spot patterns the regex crate rejects.
//! - Reports a small step->binding lookup benchmark to estimate per-query cost.

use specflow_helper::{Index, StepKind};
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let root = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: corpus_smoke <root>");
        std::process::exit(2);
    });
    let root = PathBuf::from(root);

    let cache = std::env::temp_dir().join("specflow-helper-bench-cache.json");
    let _ = std::fs::remove_file(&cache);

    println!("root: {}", root.display());

    let t = Instant::now();
    let cold = Index::build(&root, Some(&cache)).expect("cold build");
    let cold_ms = t.elapsed().as_secs_f64() * 1000.0;
    println!(
        "cold:  {:6.1}ms  files={} reparsed={} cached={} bindings={}",
        cold_ms,
        cold.stats.files_scanned,
        cold.stats.files_reparsed,
        cold.stats.files_from_cache,
        cold.bindings.len(),
    );

    let t = Instant::now();
    let warm = Index::build(&root, Some(&cache)).expect("warm build");
    let warm_ms = t.elapsed().as_secs_f64() * 1000.0;
    println!(
        "warm:  {:6.1}ms  files={} reparsed={} cached={} bindings={}",
        warm_ms,
        warm.stats.files_scanned,
        warm.stats.files_reparsed,
        warm.stats.files_from_cache,
        warm.bindings.len(),
    );

    let t = Instant::now();
    let query = warm.into_query();
    let compile_ms = t.elapsed().as_secs_f64() * 1000.0;
    println!(
        "regex compile: {:6.1}ms  patterns={} errors={}",
        compile_ms,
        query.len(),
        query.compile_errors().len(),
    );

    let steps: Vec<String> = query
        .bindings()
        .take(200)
        .map(|b| {
            b.pattern
                .split(|c: char| !c.is_alphanumeric() && c != ' ')
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .collect();
    let t = Instant::now();
    let mut hits = 0usize;
    for _ in 0..5 {
        for s in &steps {
            if query.find(s, Some(StepKind::Given)).is_some()
                || query.find(s, Some(StepKind::When)).is_some()
                || query.find(s, Some(StepKind::Then)).is_some()
            {
                hits += 1;
            }
        }
    }
    let lookup_ms = t.elapsed().as_secs_f64() * 1000.0;
    let total_lookups = steps.len() * 5 * 3;
    println!(
        "lookup bench:  {:6.1}ms total  {:.3}ms/lookup  hits={}/{}",
        lookup_ms,
        lookup_ms / total_lookups as f64,
        hits,
        steps.len() * 5,
    );

    let _ = std::fs::remove_file(&cache);
}
