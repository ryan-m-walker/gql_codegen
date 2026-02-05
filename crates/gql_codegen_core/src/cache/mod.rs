//! Caching infrastructure for incremental code generation
//!
//! Two-phase caching:
//! 1. Metadata check (fast, stat only)
//! 2. Content hash (from already-loaded SourceCache)

mod fs;
mod memory;
mod noop;
pub mod utils;

use std::path::PathBuf;

pub use utils::{CacheData, GlobCache, MetadataCheckResult};

pub use fs::FsCache;
pub use memory::MemoryCache;
pub use noop::NoCache;
pub use utils::{
    compute_hashes_from_cache, create_glob_cache, hash_config_options, is_glob_cache_valid,
    normalize_path, output_matches_existing,
};

/// Cache trait for incremental caching
pub trait Cache {
    /// Check if metadata matches (fast, no file reads)
    /// Takes pre-resolved paths to avoid duplicate glob expansion.
    fn check_metadata(&self, paths: &[PathBuf]) -> MetadataCheckResult;

    /// Check if computed hashes match stored
    fn is_fresh(&self, computed: &CacheData) -> bool;

    /// Store cache data after successful generation
    fn store(&mut self, data: CacheData) -> std::io::Result<()>;

    /// Get stored data for reference
    fn stored(&self) -> Option<&CacheData>;

    /// Clear all cached data. Returns true if cache was cleared, false if already empty.
    fn clear(&mut self) -> std::io::Result<bool>;
}
