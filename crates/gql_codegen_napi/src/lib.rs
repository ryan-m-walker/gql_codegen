//! NAPI bindings for gql_codegen
//!
//! Exposes the Rust codegen as a native Node.js module.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::path::PathBuf;

use gql_codegen_core::cache::{Cache, FsCache, NoCache};
use gql_codegen_core::{CodegenConfig, GenerateCachedResult};

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

    // Parse config from JSON — render structured error for parse failures
    let config: CodegenConfig = serde_json::from_str(&options.config_json).map_err(|e| {
        let mut buf = Vec::new();
        let config_err = gql_codegen_core::ConfigError {
            message: e.to_string(),
            file: std::path::PathBuf::from("<config>"),
            line: e.line(),
            column: e.column(),
            source_text: options.config_json.clone(),
        };
        let core_err = gql_codegen_core::Error::Config(config_err);
        let _ = gql_codegen_core::diagnostic::render_error(
            &core_err,
            gql_codegen_core::diagnostic::Color::StderrIsTerminal,
            &mut buf,
        );
        Error::from_reason(String::from_utf8(buf).unwrap_or_else(|_| format!("{core_err}")))
    })?;

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

    // Run generation — render structured diagnostics on error
    let result = gql_codegen_core::generate_cached(&config, cache.as_mut()).map_err(|e| {
        let mut buf = Vec::new();
        let _ = gql_codegen_core::diagnostic::render_error(
            &e,
            gql_codegen_core::diagnostic::Color::StderrIsTerminal,
            &mut buf,
        );
        Error::from_reason(String::from_utf8(buf).unwrap_or_else(|_| format!("{e}")))
    })?;

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
            warnings: gen_result.warnings.iter().map(|w| w.to_string()).collect(),
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
