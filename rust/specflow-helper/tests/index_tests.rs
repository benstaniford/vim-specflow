use specflow_helper::{default_cache_path, Index, StepKind};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Minimal scratch directory helper — keeps the dev-dep footprint flat.
struct Scratch {
    path: PathBuf,
}

impl Scratch {
    fn new(label: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let pid = std::process::id();
        let path = std::env::temp_dir().join(format!("specflow-helper-test-{label}-{pid}-{nanos}"));
        fs::create_dir_all(&path).expect("create scratch dir");
        Scratch { path }
    }

    fn write(&self, rel: &str, contents: &str) -> PathBuf {
        let full = self.path.join(rel);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(&full, contents).expect("write file");
        full
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

const SAMPLE_CS: &str = r#"
using TechTalk.SpecFlow;
namespace X {
    [Binding] public class S {
        [Given(@"I do '(.*)'")]
        public void Foo(string x) { }
        [When(@"I press (\d+)")]
        public void Bar(int n) { }
    }
}
"#;

#[test]
fn builds_index_from_directory_tree() {
    let dir = Scratch::new("build");
    dir.write("Tests/Foo.cs", SAMPLE_CS);
    dir.write("Tests/sub/Bar.cs", "[Then(@\"it works\")] public void X() {}");
    let idx = Index::build(&dir.path, None).expect("build index");
    assert_eq!(idx.bindings.len(), 3);
    assert_eq!(idx.stats.files_scanned, 2);
    assert_eq!(idx.stats.files_reparsed, 2);
    assert_eq!(idx.stats.files_from_cache, 0);
}

#[test]
fn cache_round_trip_skips_unchanged_files() {
    let dir = Scratch::new("cache");
    dir.write("A.cs", SAMPLE_CS);
    dir.write("B.cs", "[Then(@\"it works\")] public void X() {}");
    let cache = dir.path.join("cache.json");

    let first = Index::build(&dir.path, Some(&cache)).expect("first build");
    assert_eq!(first.stats.files_reparsed, 2);
    assert_eq!(first.stats.files_from_cache, 0);
    assert!(cache.exists(), "cache file should have been written");

    let second = Index::build(&dir.path, Some(&cache)).expect("second build");
    assert_eq!(second.stats.files_reparsed, 0);
    assert_eq!(second.stats.files_from_cache, 2);
    assert_eq!(second.bindings.len(), first.bindings.len());
}

#[test]
fn cache_reparses_changed_file_only() {
    let dir = Scratch::new("change");
    let a = dir.write("A.cs", SAMPLE_CS);
    dir.write("B.cs", "[Then(@\"it works\")] public void X() {}");
    let cache = dir.path.join("cache.json");

    let _first = Index::build(&dir.path, Some(&cache)).expect("first build");

    // Edit A.cs to add a third binding. Sleep ensures mtime advances on
    // filesystems with second-level granularity; size changes anyway.
    std::thread::sleep(std::time::Duration::from_millis(10));
    let mut updated = SAMPLE_CS.to_string();
    updated.push_str("[Given(@\"a new step\")] public void Z() {}");
    fs::write(&a, &updated).expect("update file");

    let second = Index::build(&dir.path, Some(&cache)).expect("second build");
    assert_eq!(second.stats.files_reparsed, 1, "only A.cs should reparse");
    assert_eq!(second.stats.files_from_cache, 1, "B.cs should be cached");
    assert_eq!(second.bindings.len(), 4);
}

#[test]
fn cache_picks_up_new_file() {
    let dir = Scratch::new("newfile");
    dir.write("A.cs", SAMPLE_CS);
    let cache = dir.path.join("cache.json");
    let first = Index::build(&dir.path, Some(&cache)).expect("first");
    assert_eq!(first.bindings.len(), 2);

    dir.write("C.cs", "[Given(@\"brand new\")] public void Q() {}");
    let second = Index::build(&dir.path, Some(&cache)).expect("second");
    assert_eq!(second.stats.files_reparsed, 1);
    assert_eq!(second.stats.files_from_cache, 1);
    assert_eq!(second.bindings.len(), 3);
}

#[test]
fn cache_drops_deleted_file() {
    let dir = Scratch::new("delete");
    let a = dir.write("A.cs", SAMPLE_CS);
    dir.write("B.cs", "[Then(@\"it works\")] public void X() {}");
    let cache = dir.path.join("cache.json");
    let _first = Index::build(&dir.path, Some(&cache)).expect("first");

    fs::remove_file(&a).expect("delete A.cs");
    let second = Index::build(&dir.path, Some(&cache)).expect("second");
    assert_eq!(second.stats.files_scanned, 1);
    assert_eq!(second.bindings.len(), 1);
}

#[test]
fn skips_build_output_dirs() {
    let dir = Scratch::new("skip");
    dir.write("Tests/Real.cs", SAMPLE_CS);
    // Bindings under bin/obj/.git should be ignored.
    dir.write("Tests/bin/Debug/Generated.cs", SAMPLE_CS);
    dir.write("Tests/obj/Release/Other.cs", SAMPLE_CS);
    dir.write(".git/hooks/Skip.cs", SAMPLE_CS);
    let idx = Index::build(&dir.path, None).expect("build");
    assert_eq!(idx.stats.files_scanned, 1);
    assert_eq!(idx.bindings.len(), 2);
}

#[test]
fn query_layer_filters_by_step_kind() {
    let dir = Scratch::new("query");
    dir.write("S.cs", SAMPLE_CS);
    let idx = Index::build(&dir.path, None).expect("build");
    let q = idx.into_query();
    assert!(q.find("I press 7", Some(StepKind::When)).is_some());
    assert!(q.find("I press 7", Some(StepKind::Given)).is_none());
    assert!(q.find("I press 7", None).is_some());
}

#[test]
fn default_cache_path_includes_root_hash() {
    let a = default_cache_path(Path::new("/some/repo/a"));
    let b = default_cache_path(Path::new("/some/repo/b"));
    assert_ne!(a, b, "different roots must produce different cache files");
    assert!(a.to_string_lossy().ends_with(".json"));
}
