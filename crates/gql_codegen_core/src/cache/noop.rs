//! No-op cache implementation (always regenerates)

use std::path::PathBuf;

use super::utils::{CacheData, MetadataCheckResult};
use super::Cache;

/// No-op cache - always reports as stale (forces regeneration)
pub struct NoCache;

impl Cache for NoCache {
    fn check_metadata(&self, _paths: &[PathBuf]) -> MetadataCheckResult {
        MetadataCheckResult::NoPrevious
    }

    fn is_fresh(&self, _computed: &CacheData) -> bool {
        false
    }

    fn store(&mut self, _data: CacheData) -> std::io::Result<()> {
        Ok(())
    }

    fn stored(&self) -> Option<&CacheData> {
        None
    }

    fn clear(&mut self) -> std::io::Result<bool> {
        Ok(false) // Nothing to clear
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_always_not_fresh() {
        let cache = NoCache;
        let data = CacheData {
            inputs_hash: 123,
            config_hash: 456,
            file_meta: HashMap::new(),
        };
        assert!(!cache.is_fresh(&data));
    }
}
