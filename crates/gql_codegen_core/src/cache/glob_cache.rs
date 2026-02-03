//! Incremental glob cache - only re-walks directories that changed
//!
//! Strategy:
//! - Cache file lists per-directory with directory mtime
//! - On check: stat directories, only re-walk those with changed mtime
//! - Merge changed dirs with unchanged cached dirs
//!
//! This is faster than full re-glob when only a few directories change.

use std::collections::{HashMap, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// Incremental glob cache with per-directory tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IncrementalGlobCache {
    /// Hash of glob patterns (detect config changes)
    pub patterns_hash: u64,
    /// Per-directory cache: dir path -> (mtime, files in that dir)
    pub dirs: HashMap<PathBuf, DirEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirEntry {
    /// Directory mtime (seconds since epoch)
    pub mtime: u64,
    /// Files in this directory that matched the pattern
    pub files: Vec<PathBuf>,
}

/// Result of incremental glob check
pub struct IncrementalGlobResult {
    /// All matching files (merged from cache + fresh walks)
    pub files: Vec<PathBuf>,
    /// Updated cache to store
    pub cache: IncrementalGlobCache,
    /// Stats for debugging
    pub stats: GlobStats,
}

#[derive(Debug, Default)]
pub struct GlobStats {
    pub dirs_cached: usize,
    pub dirs_walked: usize,
    pub files_from_cache: usize,
    pub files_from_walk: usize,
}

impl IncrementalGlobCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if patterns changed (requires full re-glob)
    pub fn patterns_match(&self, patterns: &[&str]) -> bool {
        self.patterns_hash == hash_patterns(patterns)
    }

    /// Perform incremental glob - only walk directories that changed
    pub fn incremental_glob(
        &self,
        patterns: &[&str],
        base_dir: &Path,
    ) -> std::io::Result<IncrementalGlobResult> {
        let current_hash = hash_patterns(patterns);

        // If patterns changed, do full glob
        if self.patterns_hash != current_hash {
            return self.full_glob(patterns, base_dir, current_hash);
        }

        // Get all directories we need to check
        let dirs_to_check: HashSet<PathBuf> = self.dirs.keys().cloned().collect();

        // Check which directories changed (parallel stat)
        let dir_status: Vec<(PathBuf, DirStatus)> = dirs_to_check
            .par_iter()
            .map(|dir| {
                let status = match get_dir_mtime(dir) {
                    Some(current_mtime) => {
                        if let Some(cached) = self.dirs.get(dir) {
                            if cached.mtime == current_mtime {
                                DirStatus::Unchanged
                            } else {
                                DirStatus::Changed(current_mtime)
                            }
                        } else {
                            DirStatus::Changed(current_mtime)
                        }
                    }
                    None => DirStatus::Removed,
                };
                (dir.clone(), status)
            })
            .collect();

        let mut stats = GlobStats::default();
        let mut new_cache = IncrementalGlobCache {
            patterns_hash: current_hash,
            dirs: HashMap::new(),
        };
        let mut all_files = Vec::new();

        // Process each directory
        for (dir, status) in dir_status {
            match status {
                DirStatus::Unchanged => {
                    // Use cached files
                    if let Some(cached) = self.dirs.get(&dir) {
                        stats.dirs_cached += 1;
                        stats.files_from_cache += cached.files.len();
                        all_files.extend(cached.files.iter().cloned());
                        new_cache.dirs.insert(dir, cached.clone());
                    }
                }
                DirStatus::Changed(mtime) => {
                    // Re-walk this directory only
                    stats.dirs_walked += 1;
                    let files = walk_directory_for_patterns(&dir, patterns)?;
                    stats.files_from_walk += files.len();
                    all_files.extend(files.iter().cloned());
                    new_cache.dirs.insert(dir, DirEntry { mtime, files });
                }
                DirStatus::Removed => {
                    // Directory no longer exists, skip it
                }
            }
        }

        // Check for new directories (not in our cache)
        // This requires walking to discover them
        let new_dirs = discover_new_directories(patterns, base_dir, &new_cache.dirs)?;
        for (dir, mtime, files) in new_dirs {
            stats.dirs_walked += 1;
            stats.files_from_walk += files.len();
            all_files.extend(files.iter().cloned());
            new_cache.dirs.insert(dir, DirEntry { mtime, files });
        }

        Ok(IncrementalGlobResult {
            files: all_files,
            cache: new_cache,
            stats,
        })
    }

    /// Full glob (when patterns change or no cache)
    fn full_glob(
        &self,
        patterns: &[&str],
        base_dir: &Path,
        patterns_hash: u64,
    ) -> std::io::Result<IncrementalGlobResult> {
        let mut stats = GlobStats::default();
        let mut dirs: HashMap<PathBuf, DirEntry> = HashMap::new();
        let mut all_files = Vec::new();

        // Use the existing glob logic to get all files
        for pattern in patterns {
            if pattern.starts_with('!') {
                continue; // Skip negation patterns for now
            }

            let full_pattern = if Path::new(pattern).is_absolute() {
                pattern.to_string()
            } else {
                base_dir.join(pattern).to_string_lossy().to_string()
            };

            let glob_paths = glob::glob(&full_pattern)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

            for entry in glob_paths.flatten() {
                if !entry.is_file() {
                    continue;
                }

                // Skip ignored directories
                if entry.components().any(|c| {
                    matches!(
                        c.as_os_str().to_str(),
                        Some("node_modules" | ".git" | "target" | "__generated__")
                    )
                }) {
                    continue;
                }

                if let Some(parent) = entry.parent() {
                    let parent = parent.to_path_buf();
                    let dir_entry = dirs.entry(parent.clone()).or_insert_with(|| {
                        let mtime = get_dir_mtime(&parent).unwrap_or(0);
                        DirEntry {
                            mtime,
                            files: Vec::new(),
                        }
                    });
                    dir_entry.files.push(entry.clone());
                }

                all_files.push(entry);
            }
        }

        stats.dirs_walked = dirs.len();
        stats.files_from_walk = all_files.len();

        Ok(IncrementalGlobResult {
            files: all_files,
            cache: IncrementalGlobCache {
                patterns_hash,
                dirs,
            },
            stats,
        })
    }
}

#[derive(Debug)]
enum DirStatus {
    Unchanged,
    Changed(u64),
    Removed,
}

/// Walk a single directory for files matching patterns
fn walk_directory_for_patterns(dir: &Path, patterns: &[&str]) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    let entries = std::fs::read_dir(dir)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            // Check if file matches any pattern
            // Simplified: just check extension for now
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let dominated_by_pattern = patterns.iter().any(|p| {
                    !p.starts_with('!') && (p.contains(&format!("*.{}", ext)) || p.ends_with("*"))
                });
                if dominated_by_pattern {
                    files.push(path);
                }
            }
        }
    }

    Ok(files)
}

/// Discover directories not in our cache
fn discover_new_directories(
    patterns: &[&str],
    base_dir: &Path,
    existing_dirs: &HashMap<PathBuf, DirEntry>,
) -> std::io::Result<Vec<(PathBuf, u64, Vec<PathBuf>)>> {
    let mut new_dirs = Vec::new();

    // For each pattern, check if there are directories we haven't seen
    for pattern in patterns {
        if pattern.starts_with('!') {
            continue;
        }

        let full_pattern = if Path::new(pattern).is_absolute() {
            pattern.to_string()
        } else {
            base_dir.join(pattern).to_string_lossy().to_string()
        };

        // Get parent directories from pattern matches
        let glob_paths = glob::glob(&full_pattern)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

        for entry in glob_paths.flatten() {
            if let Some(parent) = entry.parent() {
                let parent = parent.to_path_buf();
                if !existing_dirs.contains_key(&parent) {
                    // New directory found
                    if let Some(mtime) = get_dir_mtime(&parent) {
                        let files = walk_directory_for_patterns(&parent, patterns)?;
                        new_dirs.push((parent, mtime, files));
                    }
                }
            }
        }
    }

    Ok(new_dirs)
}

fn get_dir_mtime(path: &Path) -> Option<u64> {
    std::fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
}

fn hash_patterns(patterns: &[&str]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for p in patterns {
        p.hash(&mut hasher);
    }
    hasher.finish()
}

// TODO: Add tests once tempfile is a dev dependency
// #[cfg(test)]
// mod tests { ... }
