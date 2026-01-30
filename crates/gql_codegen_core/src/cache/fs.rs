//! Filesystem-based cache implementation

use std::fs;
use std::path::{Path, PathBuf};

use super::utils::{compute_hashes, CacheData};
use super::Cache;
use crate::config::CodegenConfig;

/// Filesystem-based cache - persists to .sgc/cache.json
pub struct FsCache {
    cache_dir: PathBuf,
    stored: CacheData,
    pending: Option<CacheData>,
}

impl FsCache {
    /// Create or load cache from the given directory
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        let cache_dir = cache_dir.into();
        let cache_file = cache_dir.join("cache.json");

        let stored = if cache_file.exists() {
            fs::read_to_string(&cache_file)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            CacheData::default()
        };

        Self {
            cache_dir,
            stored,
            pending: None,
        }
    }
}

impl Cache for FsCache {
    fn check(&mut self, config: &CodegenConfig, config_content: &str, base_dir: &Path) -> bool {
        let computed = compute_hashes(config, config_content, base_dir);
        let is_fresh = self.stored.inputs_hash == computed.inputs_hash
            && self.stored.config_hash == computed.config_hash;

        self.pending = Some(computed);
        is_fresh
    }

    fn commit(&mut self) {
        if let Some(pending) = self.pending.take() {
            self.stored = pending;
        }
    }

    fn flush(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.cache_dir)?;

        let cache_file = self.cache_dir.join("cache.json");
        let json = serde_json::to_string_pretty(&self.stored)
            .map_err(std::io::Error::other)?;

        fs::write(cache_file, json)
    }
}
