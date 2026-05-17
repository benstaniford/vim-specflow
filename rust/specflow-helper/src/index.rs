use crate::cache::{self, CacheFile, FileEntry, CACHE_VERSION};
use crate::matcher::BindingIndex;
use crate::parser::{parse_bindings, Binding};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Directories the file walker will not descend into.
///
/// These are universally either build output or VCS metadata. If any of these
/// ever turn out to legitimately hold step definitions, callers can pass a
/// narrower root.
const SKIP_DIRS: &[&str] = &[
    ".git",
    "target",
    "bin",
    "obj",
    "node_modules",
    "packages",
    ".vs",
    ".vscode",
    ".idea",
];

/// The persistent step-binding index for one root directory.
///
/// Construct via [`Index::build`]; query via [`Index::into_query`] (or
/// [`Index::bindings`] for read-only access).
pub struct Index {
    pub bindings: Vec<Binding>,
    pub stats: BuildStats,
}

#[derive(Debug, Default, Clone)]
pub struct BuildStats {
    pub files_scanned: usize,
    pub files_reparsed: usize,
    pub files_from_cache: usize,
    pub bindings_total: usize,
}

impl Index {
    /// Build the index for `root`, using `cache_path` for persistence.
    ///
    /// When `cache_path` is `None` the index is built from scratch and not
    /// persisted; useful for one-off scans and tests.
    pub fn build(root: &Path, cache_path: Option<&Path>) -> std::io::Result<Self> {
        let prior: HashMap<PathBuf, FileEntry> = cache_path
            .and_then(cache::load)
            .map(|c| c.entries.into_iter().collect())
            .unwrap_or_default();

        let cs_files = walk_cs_files(root);
        let mut bindings = Vec::new();
        let mut new_entries: Vec<(PathBuf, FileEntry)> = Vec::with_capacity(cs_files.len());
        let mut stats = BuildStats {
            files_scanned: cs_files.len(),
            ..BuildStats::default()
        };

        for file in cs_files {
            let meta = match std::fs::metadata(&file) {
                Ok(m) => m,
                Err(_) => continue,
            };
            let mtime_ns = mtime_ns(meta.modified().ok());
            let size = meta.len();

            let entry = match prior.get(&file) {
                Some(cached) if cached.mtime_ns == mtime_ns && cached.size == size => {
                    stats.files_from_cache += 1;
                    cached.clone()
                }
                _ => {
                    let src = std::fs::read_to_string(&file).unwrap_or_default();
                    let parsed = parse_bindings(&src, &file);
                    stats.files_reparsed += 1;
                    FileEntry {
                        mtime_ns,
                        size,
                        bindings: parsed,
                    }
                }
            };
            bindings.extend(entry.bindings.iter().cloned());
            new_entries.push((file, entry));
        }

        stats.bindings_total = bindings.len();

        if let Some(path) = cache_path {
            let cache = CacheFile {
                version: CACHE_VERSION,
                entries: new_entries,
            };
            // Cache write failures are non-fatal — log via stderr but return Ok.
            if let Err(e) = cache::save(path, &cache) {
                eprintln!("specflow-helper: cache write to {path:?} failed: {e}");
            }
        }

        Ok(Index { bindings, stats })
    }

    pub fn into_query(self) -> BindingIndex {
        BindingIndex::build(self.bindings)
    }

    pub fn bindings(&self) -> &[Binding] {
        &self.bindings
    }
}

fn mtime_ns(t: Option<SystemTime>) -> i128 {
    match t {
        Some(time) => match time.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(d) => d.as_nanos() as i128,
            Err(e) => -(e.duration().as_nanos() as i128),
        },
        None => 0,
    }
}

fn walk_cs_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk(root, &mut out);
    out
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(ft) = entry.file_type() else { continue };
        if ft.is_dir() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if SKIP_DIRS.contains(&name.as_ref()) {
                continue;
            }
            // Skip hidden dirs except `.` and `..` (already filtered by read_dir).
            if name.starts_with('.') && name.as_ref() != "." {
                continue;
            }
            walk(&path, out);
        } else if ft.is_file() && path.extension().and_then(|s| s.to_str()) == Some("cs") {
            out.push(path);
        }
    }
}
