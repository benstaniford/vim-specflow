//! Scan every .feature file under the corpus, resolve every step, print a
//! by-file unbound summary. Run:
//!   cargo run --release --example corpus_unbound -- /path/to/Tests/Automation

use specflow_helper::{parse_feature_steps, Index};
use std::path::{Path, PathBuf};

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

fn main() {
    let root = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/home/ben/Code/epm-windows/Tests/Automation".to_string());
    let root = PathBuf::from(root);
    let idx = Index::build(&root, None).expect("build index");
    let query = idx.into_query();

    let mut features = Vec::new();
    walk_features(&root, &mut features);

    let mut total = 0usize;
    let mut unbound_total = 0usize;
    let mut per_file: Vec<(PathBuf, usize, usize)> = Vec::new();
    for f in &features {
        let src = std::fs::read_to_string(f).unwrap_or_default();
        let steps = parse_feature_steps(&src);
        let mut unbound = 0;
        for s in &steps {
            if query.find(&s.text, s.kind).is_none() {
                unbound += 1;
            }
        }
        total += steps.len();
        unbound_total += unbound;
        per_file.push((f.clone(), steps.len(), unbound));
    }

    per_file.sort_by_key(|(_, _, u)| std::cmp::Reverse(*u));
    println!("feature files: {}", features.len());
    println!("total steps: {total}");
    println!(
        "unbound: {unbound_total} ({:.2}%)",
        100.0 * unbound_total as f64 / total.max(1) as f64
    );
    println!("\nTop 15 files by unbound count:");
    for (f, n, u) in per_file.iter().take(15) {
        if *u == 0 {
            break;
        }
        println!(
            "  {:>3}/{:<3} unbound  {}",
            u,
            n,
            f.strip_prefix(&root).unwrap_or(f).display()
        );
    }
}
