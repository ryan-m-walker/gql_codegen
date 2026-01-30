//! No-op cache implementation (caching disabled)

use std::path::Path;

use super::Cache;
use crate::config::CodegenConfig;

/// No-op cache - caching disabled, always regenerates
pub struct NoCache;

impl Cache for NoCache {
    fn check(&mut self, _config: &CodegenConfig, _config_content: &str, _base_dir: &Path) -> bool {
        false // Always stale
    }

    fn commit(&mut self) {
        // No-op
    }

    fn flush(&self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::StringOrArray;
    use std::collections::HashMap;

    #[test]
    fn test_always_stale() {
        let mut cache = NoCache;
        let config = CodegenConfig {
            schema: StringOrArray::Single("test.graphql".into()),
            documents: StringOrArray::Multiple(vec![]),
            generates: HashMap::new(),
            base_dir: None,
        };

        assert!(!cache.check(&config, "{}", Path::new(".")));
        cache.commit();
        assert!(!cache.check(&config, "{}", Path::new("."))); // Still stale
    }
}
