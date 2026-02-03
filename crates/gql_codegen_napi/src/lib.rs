//! NAPI bindings for gql_codegen
//!
//! Exposes the Rust codegen as a native Node.js module.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::path::PathBuf;

use gql_codegen_core::{
    cache::{Cache, FsCache, NoCache},
    CodegenConfig, GenerateCachedResult,
};

/// Result of code generation
#[napi(object)]
pub struct GenerateResult {
    /// Whether generation was skipped (cache hit)
    pub fresh: bool,
    /// Generated files (only populated if not fresh)
    pub files: Vec<GeneratedFile>,
    /// Warnings encountered during generation
    pub warnings: Vec<String>,
}

#[napi(object)]
pub struct GeneratedFile {
    pub path: String,
    pub content: String,
}

/// Options for the generate function
#[napi(object)]
pub struct GenerateOptions {
    /// JSON string of the config
    pub config_json: String,
    /// Whether to skip caching
    pub no_cache: Option<bool>,
    /// Whether to enable timing output
    pub timing: Option<bool>,
}

/// Generate TypeScript types from GraphQL schema and operations
#[napi]
pub fn generate(options: GenerateOptions) -> Result<GenerateResult> {
    // Enable timing if requested
    if options.timing.unwrap_or(false) {
        gql_codegen_core::timing::enable_timing();
    }

    // Parse config from JSON
    let config: CodegenConfig = serde_json::from_str(&options.config_json)
        .map_err(|e| Error::from_reason(format!("Invalid config JSON: {e}")))?;

    // Set up cache
    let base_dir = config
        .base_dir
        .as_ref()
        .map(|s| PathBuf::from(s.as_str()))
        .unwrap_or_else(|| PathBuf::from("."));

    let mut cache: Box<dyn Cache> = if options.no_cache.unwrap_or(false) {
        Box::new(NoCache)
    } else {
        Box::new(FsCache::new(base_dir.join(".sgc")))
    };

    // Run generation
    let result = gql_codegen_core::generate_cached(&config, cache.as_mut())
        .map_err(|e| Error::from_reason(format!("Generation failed: {e}")))?;

    // Convert to NAPI result
    match result {
        GenerateCachedResult::Fresh => Ok(GenerateResult {
            fresh: true,
            files: vec![],
            warnings: vec![],
        }),
        GenerateCachedResult::Generated(gen_result) => Ok(GenerateResult {
            fresh: false,
            files: gen_result
                .files
                .into_iter()
                .map(|f| GeneratedFile {
                    path: f.path,
                    content: f.content,
                })
                .collect(),
            warnings: gen_result.warnings,
        }),
    }
}

/// Clear the cache directory
#[napi]
pub fn clear_cache(base_dir: String) -> Result<bool> {
    let cache_dir = PathBuf::from(&base_dir).join(".sgc");
    let mut cache = FsCache::new(cache_dir);

    cache
        .clear()
        .map_err(|e| Error::from_reason(format!("Failed to clear cache: {e}")))
}
