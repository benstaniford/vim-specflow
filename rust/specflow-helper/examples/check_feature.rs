//! Ad-hoc helper: report which steps in a .feature file have no binding.
//!
//! Run: cargo run --release --example check_feature -- <root> <feature>

use specflow_helper::{Index, StepKind};
use std::path::PathBuf;

fn main() {
    let mut args = std::env::args().skip(1);
    let root = PathBuf::from(args.next().expect("usage: check_feature <root> <feature>"));
    let feature = PathBuf::from(args.next().expect("usage: check_feature <root> <feature>"));

    let idx = Index::build(&root, None).expect("build index");
    println!(
        "index: {} bindings from {} files\n",
        idx.bindings.len(),
        idx.stats.files_scanned
    );
    let query = idx.into_query();

    let src = std::fs::read_to_string(&feature).expect("read feature");
    let mut last_kind: Option<StepKind> = None;
    for (i, line) in src.lines().enumerate() {
        let trimmed = line.trim_start();
        let (kind, rest) = if let Some(r) = trimmed.strip_prefix("Given ") {
            (Some(StepKind::Given), r)
        } else if let Some(r) = trimmed.strip_prefix("When ") {
            (Some(StepKind::When), r)
        } else if let Some(r) = trimmed.strip_prefix("Then ") {
            (Some(StepKind::Then), r)
        } else if let Some(r) = trimmed.strip_prefix("And ") {
            (last_kind, r)
        } else if let Some(r) = trimmed.strip_prefix("But ") {
            (last_kind, r)
        } else {
            continue;
        };
        if let Some(k) = kind {
            last_kind = Some(k);
        }
        // Important: keep the trailing colon — SpecFlow binding patterns for
        // data-table steps include it (e.g. `I run '(.*)' with the following:`).
        let step = rest.trim_end();
        match query.find(step, kind) {
            Some(m) => println!(
                "  ok  L{:>3} [{:?}] -> {}:{}",
                i + 1,
                kind,
                m.binding.file.display(),
                m.binding.line
            ),
            None => println!("  ??  L{:>3} [{:?}] {}", i + 1, kind, step),
        }
    }
}
