//! Internal hashing utilities for cache

use std::collections::HashMap;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::CodegenConfig;
use crate::source_cache::SourceCache;

/// File metadata for fast change detection (no content read needed)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileMeta {
    pub mtime_secs: u64,
    pub size: u64,
}

impl FileMeta {
    /// Get metadata from filesystem (stat only, no read)
    pub fn from_path(path: &Path) -> Option<Self> {
        let metadata = fs::metadata(path).ok()?;
        let mtime = metadata.modified().ok()?;
        let mtime_secs = mtime.duration_since(SystemTime::UNIX_EPOCH).ok()?.as_secs();
        let size = metadata.len();

        Some(Self { mtime_secs, size })
    }

    /// Check if current file metadata matches cached
    pub fn matches_current(&self, path: &Path) -> bool {
        Self::from_path(path)
            .map(|current| current == *self)
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheData {
    /// Combined hash of all input file contents
    pub inputs_hash: u64,
    /// Hash of config options
    pub config_hash: u64,
    /// Per-file metadata for fast change detection
    pub file_meta: HashMap<PathBuf, FileMeta>,
    /// Cached glob results for fast file discovery
    #[serde(default)]
    pub glob_cache: Option<GlobCache>,
}

/// Cached glob expansion results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobCache {
    /// Hash of the glob patterns (to detect config changes)
    pub patterns_hash: u64,
    /// The resolved file paths
    pub files: Vec<PathBuf>,
    /// Directory mtimes at cache time (for invalidation)
    pub dir_mtimes: HashMap<PathBuf, u64>,
}

/// Result of Phase 1 metadata check
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataCheckResult {
    /// All metadata matches - likely cache hit (skip loading)
    AllMatch,
    /// Some metadata changed - need to load and verify
    Changed,
    /// No previous cache data
    NoPrevious,
}

pub fn check_metadata(paths: &[PathBuf], previous: Option<&CacheData>) -> MetadataCheckResult {
    let Some(prev) = previous else {
        return MetadataCheckResult::NoPrevious;
    };

    // Check if file set changed (new files or deleted files)
    let current_set: std::collections::HashSet<_> = paths.iter().collect();
    let cached_set: std::collections::HashSet<_> = prev.file_meta.keys().collect();

    if current_set != cached_set {
        return MetadataCheckResult::Changed;
    }

    let all_match = paths.par_iter().all(|path| {
        prev.file_meta
            .get(path)
            .map(|meta| meta.matches_current(path))
            .unwrap_or(false)
    });

    if all_match {
        MetadataCheckResult::AllMatch
    } else {
        MetadataCheckResult::Changed
    }
}

pub fn compute_hashes_from_cache(
    config: &CodegenConfig,
    source_cache: &SourceCache,
    schema_files: &[(PathBuf, String)],
) -> CacheData {
    let capacity = schema_files.len() + source_cache.len();
    let mut file_meta = HashMap::with_capacity(capacity);
    let mut content_hashes = Vec::with_capacity(capacity);

    for (path, content) in schema_files {
        if let Some(meta) = FileMeta::from_path(path) {
            file_meta.insert(path.clone(), meta);
        }
        content_hashes.push((path.clone(), hash_bytes(content.as_bytes())));
    }

    for (_idx, path, content) in source_cache.iter() {
        if let Some(meta) = FileMeta::from_path(path) {
            file_meta.insert(path.to_path_buf(), meta);
        }
        content_hashes.push((path.to_path_buf(), hash_bytes(content.as_bytes())));
    }

    // Sort for deterministic ordering
    content_hashes.sort_by(|(a, _), (b, _)| a.cmp(b));
    let all_hashes: Vec<u64> = content_hashes.iter().map(|(_, h)| *h).collect();

    let config_hash = hash_config_options(config);
    let inputs_hash = combine_hashes(&all_hashes);

    CacheData {
        inputs_hash,
        config_hash,
        file_meta,
        glob_cache: None, // Set by caller if glob caching is used
    }
}

/// Normalize a path - if absolute, use as-is; if relative, join with base_dir
pub fn normalize_path(path: &str, base_dir: &Path) -> PathBuf {
    let p = Path::new(path);

    if p.is_absolute() {
        p.to_path_buf()
    } else {
        base_dir.join(p)
    }
}

/// Hash config options that affect output (excludes paths)
pub fn hash_config_options(config: &CodegenConfig) -> u64 {
    let mut hasher = DefaultHasher::new();

    let mut generates: Vec<_> = config.generates.iter().collect();
    generates.sort_by_key(|(k, _)| *k);

    for (output_path, output_config) in generates {
        if let Some(filename) = Path::new(output_path).file_name() {
            filename.hash(&mut hasher);
        }

        for plugin in &output_config.plugins {
            plugin.name().hash(&mut hasher);
            if let Some(opts) = plugin.options() {
                hash_plugin_options(opts, &mut hasher);
            }
        }

        if let Some(opts) = &output_config.config {
            hash_plugin_options(opts, &mut hasher);
        }

        output_config.documents_only.hash(&mut hasher);
        output_config.prelude.hash(&mut hasher);
    }

    hasher.finish()
}

fn hash_plugin_options(opts: &crate::config::PluginOptions, hasher: &mut DefaultHasher) {
    // PluginOptions derives Hash with BTreeMap for deterministic ordering
    opts.hash(hasher);
}

fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

fn combine_hashes(hashes: &[u64]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for h in hashes {
        h.hash(&mut hasher);
    }
    hasher.finish()
}

/// Check if output content matches existing file (for skip-write optimization)
pub fn output_matches_existing(path: &Path, new_content: &[u8]) -> bool {
    match fs::read(path) {
        Ok(existing) => existing == new_content,
        Err(_) => false,
    }
}

/// Check if cached glob results are still valid
pub fn is_glob_cache_valid(cache: &GlobCache, patterns: &[&str]) -> bool {
    // Check if patterns changed
    let current_hash = hash_patterns(patterns);
    if cache.patterns_hash != current_hash {
        return false;
    }

    // Check if any directory mtime changed (would indicate new/deleted files)
    for (dir, cached_mtime) in &cache.dir_mtimes {
        let current_mtime = get_dir_mtime(dir);
        if current_mtime != Some(*cached_mtime) {
            return false;
        }
    }

    // Check that all cached files still exist
    cache.files.par_iter().all(|f| f.exists())
}

/// Create a new glob cache from resolved paths
pub fn create_glob_cache(patterns: &[&str], files: Vec<PathBuf>) -> GlobCache {
    let patterns_hash = hash_patterns(patterns);

    // Collect unique parent directories
    let mut dirs: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();
    for file in &files {
        if let Some(parent) = file.parent() {
            // Add the immediate parent and walk up to capture structure changes
            let mut current = parent;
            loop {
                dirs.insert(current.to_path_buf());
                match current.parent() {
                    Some(p) if !p.as_os_str().is_empty() => current = p,
                    _ => break,
                }
            }
        }
    }

    // Get mtimes for all directories
    let dir_mtimes: HashMap<PathBuf, u64> = dirs
        .into_iter()
        .filter_map(|dir| get_dir_mtime(&dir).map(|mtime| (dir, mtime)))
        .collect();

    GlobCache {
        patterns_hash,
        files,
        dir_mtimes,
    }
}

fn hash_patterns(patterns: &[&str]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for p in patterns {
        p.hash(&mut hasher);
    }
    hasher.finish()
}

fn get_dir_mtime(path: &Path) -> Option<u64> {
    fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
}
