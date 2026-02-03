//! In-memory cache implementation (for testing)

use std::path::PathBuf;

use super::utils::{check_metadata, CacheData, MetadataCheckResult};
use super::Cache;

/// In-memory cache - useful for testing
pub struct MemoryCache {
    stored: Option<CacheData>,
}

impl MemoryCache {
    pub fn new() -> Self {
        Self { stored: None }
    }
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

impl Cache for MemoryCache {
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
        self.stored = Some(data);
        Ok(())
    }

    fn stored(&self) -> Option<&CacheData> {
        self.stored.as_ref()
    }

    fn clear(&mut self) -> std::io::Result<bool> {
        let had_data = self.stored.is_some();
        self.stored = None;
        Ok(had_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_no_stored_is_not_fresh() {
        let cache = MemoryCache::new();
        let data = CacheData {
            inputs_hash: 123,
            config_hash: 456,
            file_meta: HashMap::new(),
            glob_cache: None,
        };
        assert!(!cache.is_fresh(&data));
    }

    #[test]
    fn test_matching_hashes_is_fresh() {
        let mut cache = MemoryCache::new();
        let data = CacheData {
            inputs_hash: 123,
            config_hash: 456,
            file_meta: HashMap::new(),
            glob_cache: None,
        };
        cache.store(data.clone()).unwrap();
        assert!(cache.is_fresh(&data));
    }

    #[test]
    fn test_different_hashes_not_fresh() {
        let mut cache = MemoryCache::new();
        let data1 = CacheData {
            inputs_hash: 123,
            config_hash: 456,
            file_meta: HashMap::new(),
            glob_cache: None,
        };
        cache.store(data1).unwrap();

        let data2 = CacheData {
            inputs_hash: 999,
            config_hash: 456,
            file_meta: HashMap::new(),
            glob_cache: None,
        };
        assert!(!cache.is_fresh(&data2));
    }
}
