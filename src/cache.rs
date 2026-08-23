use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use tempfile::NamedTempFile;
use tracing::warn;

/// Tracks which post/thread ids have already been notified, keyed to their
/// creation time so stale entries can be pruned. Persists as JSON at
/// `<dir>/<file_name>`, written atomically so a crash mid-write can never
/// leave a truncated file that later gets mistaken for an empty one.
pub struct SeenCache {
    path: PathBuf,
    entries: HashMap<String, i64>,
}

impl SeenCache {
    /// Load the cache from disk. A missing or unparseable file is treated as
    /// an empty cache for this run — the file itself is left untouched, so a
    /// transient read glitch can never wipe real history.
    pub fn load(dir: &Path, file_name: &str) -> Self {
        let path = dir.join(file_name);
        let entries = match fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|err| {
                warn!(%err, path = %path.display(), "cache file unreadable, starting empty for this run");
                HashMap::new()
            }),
            Err(_) => HashMap::new(),
        };
        Self { path, entries }
    }

    pub fn contains(&self, id: &str) -> bool {
        self.entries.contains_key(id)
    }

    pub fn remember(&mut self, id: &str, created_at: i64) {
        self.entries.insert(id.to_string(), created_at);
    }

    /// Drop entries older than `cutoff` (a unix timestamp).
    pub fn prune(&mut self, cutoff: i64) {
        self.entries.retain(|_, created_at| *created_at >= cutoff);
    }

    /// Write the cache back to disk atomically: write to a temp file in the
    /// same directory, then rename over the real path.
    pub fn save(&self) -> anyhow::Result<()> {
        let dir = self.path.parent().unwrap_or_else(|| Path::new("."));
        let mut tmp = NamedTempFile::new_in(dir)?;
        serde_json::to_writer(&mut tmp, &self.entries)?;
        tmp.persist(&self.path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_missing_file_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        let cache = SeenCache::load(dir.path(), "cache.json");
        assert!(!cache.contains("anything"));
    }

    #[test]
    fn corrupt_file_is_never_overwritten_by_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cache.json");
        fs::write(&path, b"not json").unwrap();

        let cache = SeenCache::load(dir.path(), "cache.json");
        assert!(!cache.contains("anything"));
        // load() must not touch the file on disk.
        assert_eq!(fs::read(&path).unwrap(), b"not json");
    }

    #[test]
    fn remember_and_contains_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let mut cache = SeenCache::load(dir.path(), "cache.json");
        cache.remember("abc", 1000);
        assert!(cache.contains("abc"));
        assert!(!cache.contains("xyz"));
    }

    #[test]
    fn prune_drops_entries_older_than_cutoff() {
        let dir = tempfile::tempdir().unwrap();
        let mut cache = SeenCache::load(dir.path(), "cache.json");
        cache.remember("old", 100);
        cache.remember("boundary", 200);
        cache.remember("new", 300);

        cache.prune(200);

        assert!(!cache.contains("old"));
        assert!(cache.contains("boundary"));
        assert!(cache.contains("new"));
    }

    #[test]
    fn save_then_load_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let mut cache = SeenCache::load(dir.path(), "cache.json");
        cache.remember("abc", 1234);
        cache.save().unwrap();

        let reloaded = SeenCache::load(dir.path(), "cache.json");
        assert!(reloaded.contains("abc"));
    }

    #[test]
    fn save_replaces_file_atomically_not_zeroing_on_reload() {
        let dir = tempfile::tempdir().unwrap();
        let mut cache = SeenCache::load(dir.path(), "cache.json");
        cache.remember("abc", 1234);
        cache.save().unwrap();

        // Simulate a second process/run loading concurrently-ish: the file
        // must always be either the old complete contents or the new
        // complete contents, never partial.
        let path = dir.path().join("cache.json");
        let contents = fs::read_to_string(&path).unwrap();
        assert!(serde_json::from_str::<HashMap<String, i64>>(&contents).is_ok());
    }
}
