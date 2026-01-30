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

mod fs;
mod memory;
mod noop;
pub(crate) mod utils;

use std::path::Path;

use crate::config::CodegenConfig;

// Re-exports
pub use fs::FsCache;
pub use memory::MemoryCache;
pub use noop::NoCache;

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
