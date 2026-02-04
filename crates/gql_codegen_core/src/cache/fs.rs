//! Filesystem-based cache implementation

use std::fs;
use std::path::PathBuf;

use super::Cache;
use super::utils::{CacheData, MetadataCheckResult, check_metadata};

/// Filesystem-based cache - persists to .sgc/cache.json
pub struct FsCache {
    cache_dir: PathBuf,
    stored: Option<CacheData>,
}

impl FsCache {
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        let cache_dir = cache_dir.into();
        let cache_file = cache_dir.join("cache.json");

        let stored = if cache_file.exists() {
            fs::read_to_string(&cache_file)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
        } else {
            None
        };

        Self { cache_dir, stored }
    }
}

impl Cache for FsCache {
    fn check_metadata(&self, paths: &[PathBuf]) -> MetadataCheckResult {
        check_metadata(paths, self.stored.as_ref())
    }

    fn is_fresh(&self, computed: &CacheData) -> bool {
        self.stored
            .as_ref()
            .map(|s| s.inputs_hash == computed.inputs_hash && s.config_hash == computed.config_hash)
            .unwrap_or(false)
    }

    fn store(&mut self, data: CacheData) -> std::io::Result<()> {
        fs::create_dir_all(&self.cache_dir)?;

        let cache_file = self.cache_dir.join("cache.json");
        let json = serde_json::to_string_pretty(&data).map_err(std::io::Error::other)?;

        fs::write(cache_file, json)?;
        self.stored = Some(data);
        Ok(())
    }

    fn stored(&self) -> Option<&CacheData> {
        self.stored.as_ref()
    }

    fn clear(&mut self) -> std::io::Result<bool> {
        self.stored = None;
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
