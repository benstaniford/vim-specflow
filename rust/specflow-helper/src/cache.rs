use crate::parser::Binding;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// Per-file cache entry. `mtime_ns` and `size` are the invalidation key —
/// any change to either triggers a re-parse on the next index build.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub mtime_ns: i128,
    pub size: u64,
    pub bindings: Vec<Binding>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheFile {
    /// Schema version — bump when the on-disk format changes incompatibly.
    pub version: u32,
    pub entries: Vec<(PathBuf, FileEntry)>,
}

pub const CACHE_VERSION: u32 = 1;

/// Compute the default cache path for `root`.
///
/// Resolution order:
///   1. `XDG_CACHE_HOME` (Linux/Mac if explicitly set)
///   2. `LOCALAPPDATA` (Windows-standard, e.g. `C:\Users\X\AppData\Local`)
///   3. `HOME/.cache` (Linux/Mac default)
///   4. `USERPROFILE\AppData\Local` (Windows fallback)
///   5. relative `.cache` (last-ditch)
///
/// The filename embeds a hash of the canonicalized root so multiple repos
/// coexist.
pub fn default_cache_path(root: &Path) -> PathBuf {
    let base = resolve_base_dir(
        std::env::var_os("XDG_CACHE_HOME").as_deref().map(Path::new),
        std::env::var_os("LOCALAPPDATA").as_deref().map(Path::new),
        std::env::var_os("HOME").as_deref().map(Path::new),
        std::env::var_os("USERPROFILE").as_deref().map(Path::new),
    );
    let canonical = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let mut hasher = DefaultHasher::new();
    canonical.hash(&mut hasher);
    let key = format!("{:016x}", hasher.finish());
    base.join("vim-specflow").join(format!("{key}.json"))
}

fn resolve_base_dir(
    xdg: Option<&Path>,
    localappdata: Option<&Path>,
    home: Option<&Path>,
    userprofile: Option<&Path>,
) -> PathBuf {
    if let Some(p) = xdg {
        return p.to_path_buf();
    }
    if let Some(p) = localappdata {
        return p.to_path_buf();
    }
    if let Some(p) = home {
        return p.join(".cache");
    }
    if let Some(p) = userprofile {
        return p.join("AppData").join("Local");
    }
    PathBuf::from(".cache")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xdg_wins_when_set() {
        let base = resolve_base_dir(
            Some(Path::new("/tmp/xdg")),
            Some(Path::new("C:\\Local")),
            Some(Path::new("/home/x")),
            Some(Path::new("C:\\Users\\X")),
        );
        assert_eq!(base, Path::new("/tmp/xdg"));
    }

    #[test]
    fn localappdata_used_when_xdg_absent() {
        let base = resolve_base_dir(
            None,
            Some(Path::new("C:\\Users\\X\\AppData\\Local")),
            Some(Path::new("/home/x")),
            None,
        );
        assert_eq!(base, Path::new("C:\\Users\\X\\AppData\\Local"));
    }

    #[test]
    fn home_used_on_unix_with_no_xdg_or_localappdata() {
        let base = resolve_base_dir(None, None, Some(Path::new("/home/x")), None);
        assert_eq!(base, Path::new("/home/x/.cache"));
    }

    #[test]
    fn userprofile_is_last_resort_before_relative() {
        // Compare via PathBuf::join so the assertion is independent of the
        // host's path separator (test runs on Linux too).
        let base = resolve_base_dir(None, None, None, Some(Path::new("C:\\Users\\X")));
        assert_eq!(
            base,
            Path::new("C:\\Users\\X").join("AppData").join("Local")
        );
    }

    #[test]
    fn relative_fallback_when_nothing_set() {
        let base = resolve_base_dir(None, None, None, None);
        assert_eq!(base, Path::new(".cache"));
    }
}

pub fn load(path: &Path) -> Option<CacheFile> {
    let bytes = std::fs::read(path).ok()?;
    let cache: CacheFile = serde_json::from_slice(&bytes).ok()?;
    if cache.version != CACHE_VERSION {
        return None;
    }
    Some(cache)
}

pub fn save(path: &Path, cache: &CacheFile) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_vec(cache).map_err(std::io::Error::other)?;
    // Atomic-ish write: temp file + rename.
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, &json)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}
