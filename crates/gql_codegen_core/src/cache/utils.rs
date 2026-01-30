//! Internal hashing utilities for cache

use std::collections::HashMap;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::CodegenConfig;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct CacheData {
    pub inputs_hash: u64,
    pub config_hash: u64,
    pub file_hashes: HashMap<PathBuf, u64>,
}

pub(crate) fn compute_hashes(
    config: &CodegenConfig,
    config_content: &str,
    base_dir: &Path,
) -> CacheData {
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
