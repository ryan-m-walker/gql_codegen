//! In-memory cache implementation

use std::path::Path;

use super::utils::{compute_hashes, CacheData};
use super::Cache;
use crate::config::CodegenConfig;

/// In-memory cache - for testing or single-run scenarios
#[derive(Default)]
pub struct MemoryCache {
    stored: CacheData,
    pending: Option<CacheData>,
}

impl MemoryCache {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Cache for MemoryCache {
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
        Ok(()) // No-op for memory cache
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::StringOrArray;
    use std::collections::HashMap;

    fn test_config() -> CodegenConfig {
        CodegenConfig {
            schema: StringOrArray::Single("test.graphql".into()),
            documents: StringOrArray::Multiple(vec![]),
            generates: HashMap::new(),
            base_dir: None,
        }
    }

    #[test]
    fn test_workflow() {
        let mut cache = MemoryCache::new();
        let config = test_config();

        // First check - always stale (no previous data)
        assert!(!cache.check(&config, "{}", Path::new(".")));

        // Commit the hashes
        cache.commit();

        // Second check with same inputs - should be fresh
        assert!(cache.check(&config, "{}", Path::new(".")));

        // Check with different config content - should be stale
        assert!(!cache.check(&config, "{\"changed\": true}", Path::new(".")));
    }

    #[test]
    fn test_commit_only_after_success() {
        let mut cache = MemoryCache::new();
        let config = test_config();

        // Check but don't commit (simulating failed generation)
        assert!(!cache.check(&config, "{}", Path::new(".")));
        // Don't call commit()

        // Next check should still be stale
        assert!(!cache.check(&config, "{}", Path::new(".")));

        // Now commit
        cache.commit();

        // Should be fresh
        assert!(cache.check(&config, "{}", Path::new(".")));
    }
}
