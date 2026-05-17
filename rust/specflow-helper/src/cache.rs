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
/// Honours `XDG_CACHE_HOME`, falling back to `$HOME/.cache`. The filename
/// embeds a hash of the canonicalized root so multiple repos coexist.
pub fn default_cache_path(root: &Path) -> PathBuf {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
        .unwrap_or_else(|| PathBuf::from(".cache"));
    let canonical = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let mut hasher = DefaultHasher::new();
    canonical.hash(&mut hasher);
    let key = format!("{:016x}", hasher.finish());
    base.join("vim-specflow").join(format!("{key}.json"))
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
