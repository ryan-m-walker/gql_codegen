//! Caching infrastructure for incremental code generation
//!
//! Provides a trait-based caching system with pluggable implementations.
//!
//! # Example
//! ```ignore
//! let mut cache = FsCache::new(".sgc");
//! if cache.check(&config, &config_content, &base_dir) {
//!     println!("Nothing changed");
//!     return Ok(());
//! }
//! // ... generate files ...
//! cache.commit();
//! cache.flush()?;
//! ```

use std::collections::HashMap;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::CodegenConfig;

/// Cache trait - implement this for custom caching strategies
pub trait Cache {
    /// Check if inputs are fresh (unchanged since last generation).
    /// Computes and stores hashes internally for later commit.
    /// Returns `true` if cache is fresh (no regeneration needed).
    fn check(&mut self, config: &CodegenConfig, config_content: &str, base_dir: &Path) -> bool;

    /// Commit the pending hashes after successful generation.
    /// Call this only after generation succeeds.
    fn commit(&mut self);

    /// Persist cache to storage (if applicable).
    fn flush(&self) -> std::io::Result<()>;
}

// ─────────────────────────────────────────────────────────────────────────────
// Filesystem Cache
// ─────────────────────────────────────────────────────────────────────────────

/// Filesystem-based cache - persists to .sgc/cache.json
pub struct FsCache {
    cache_dir: PathBuf,
    stored: CacheData,
    pending: Option<CacheData>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct CacheData {
    inputs_hash: u64,
    config_hash: u64,
    file_hashes: HashMap<PathBuf, u64>,
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

        // Store pending for later commit
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
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        fs::write(cache_file, json)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Memory Cache
// ─────────────────────────────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────────────────────────
// No-op Cache
// ─────────────────────────────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────────────────────────
// Internal hashing utilities
// ─────────────────────────────────────────────────────────────────────────────

fn compute_hashes(config: &CodegenConfig, config_content: &str, base_dir: &Path) -> CacheData {
    // Collect all paths to hash
    let paths: Vec<PathBuf> = config
        .schema
        .as_vec()
        .iter()
        .map(|p| base_dir.join(p))
        .collect();

    // Parallel hash all files
    let file_hashes: HashMap<PathBuf, u64> = paths
        .par_iter()
        .filter_map(|path| hash_file(path).ok().map(|h| (path.clone(), h)))
        .collect();

    let all_hashes: Vec<u64> = file_hashes.values().copied().collect();
    let config_hash = hash_str(config_content);
    let inputs_hash = combine_hashes(&all_hashes);

    CacheData {
        inputs_hash,
        config_hash,
        file_hashes,
    }
}

fn hash_file(path: &Path) -> std::io::Result<u64> {
    let content = fs::read(path)?;
    Ok(hash_bytes(&content))
}

fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

fn hash_str(s: &str) -> u64 {
    hash_bytes(s.as_bytes())
}

fn combine_hashes(hashes: &[u64]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for h in hashes {
        h.hash(&mut hasher);
    }
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::StringOrArray;
    use std::collections::HashMap as StdHashMap;

    fn test_config() -> CodegenConfig {
        CodegenConfig {
            schema: StringOrArray::Single("test.graphql".into()),
            documents: StringOrArray::Multiple(vec![]),
            generates: StdHashMap::new(),
            base_dir: None,
        }
    }

    #[test]
    fn test_memory_cache_workflow() {
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
    fn test_no_cache_always_stale() {
        let mut cache = NoCache;
        let config = test_config();

        assert!(!cache.check(&config, "{}", Path::new(".")));
        cache.commit();
        assert!(!cache.check(&config, "{}", Path::new("."))); // Still stale
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
