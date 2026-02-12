//! NAPI bindings for gql_codegen
//!
//! Exposes the Rust codegen as a native Node.js module.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::path::PathBuf;

use gql_codegen_core::cache::{Cache, FsCache, NoCache};
use gql_codegen_core::diagnostic::{
    self, Diagnostic, DiagnosticCategory, DiagnosticLocation, Diagnostics,
};
use gql_codegen_core::writer::{write_outputs, FsWriter};
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
    /// Max diagnostics to show per error group (0 = unlimited, default 3)
    pub max_diagnostics: Option<u32>,
}

/// Generate TypeScript types from GraphQL schema and operations
#[napi]
pub fn generate(options: GenerateOptions) -> Result<GenerateResult> {
    // Enable timing if requested
    if options.timing.unwrap_or(false) {
        gql_codegen_core::timing::enable_timing();
    }

    let max_diag = options
        .max_diagnostics
        .unwrap_or(diagnostic::DEFAULT_MAX_DIAGNOSTICS as u32) as usize;

    // Parse config from JSON — render structured error for parse failures
    let config: CodegenConfig = serde_json::from_str(&options.config_json).map_err(|e| {
        let d = Diagnostic::error(DiagnosticCategory::Config, e.to_string())
            .with_location(DiagnosticLocation {
                file: PathBuf::from("<config>"),
                line: e.line(),
                column: e.column(),
                length: None,
            })
            .with_inline_source(options.config_json.clone());
        let ds = Diagnostics::from(d);
        Error::from_reason(diagnostic::render_diagnostics_string(&ds, max_diag))
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
        Error::from_reason(diagnostic::render_diagnostics_string(&e, max_diag))
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
            warnings: gen_result
                .diagnostics
                .warnings()
                .map(|d| diagnostic::render_diagnostic_string(d))
                .collect(),
        }),
    }
}

/// Result of writing files to disk
#[napi(object)]
pub struct WriteFilesResult {
    /// Paths that were written
    pub written: Vec<String>,
    /// Paths skipped because content already matched
    pub skipped: Vec<String>,
    /// Paths that failed to write, with error messages
    pub errors: Vec<WriteError>,
}

#[napi(object)]
pub struct WriteError {
    pub path: String,
    pub message: String,
}

/// Write generated files to disk using parallel I/O.
///
/// Uses Rayon for parallel writes and skips files whose content already matches,
/// avoiding unnecessary filesystem events (useful for watch mode).
#[napi]
pub fn write_files(files: Vec<GeneratedFile>) -> WriteFilesResult {
    let core_files: Vec<gql_codegen_core::GeneratedFile> = files
        .into_iter()
        .map(|f| gql_codegen_core::GeneratedFile {
            path: f.path,
            content: f.content,
        })
        .collect();

    let result = write_outputs(&core_files, &FsWriter::new());

    WriteFilesResult {
        written: result
            .written
            .into_iter()
            .map(|p| p.display().to_string())
            .collect(),
        skipped: result
            .skipped
            .into_iter()
            .map(|p| p.display().to_string())
            .collect(),
        errors: result
            .errors
            .into_iter()
            .map(|(p, msg)| WriteError {
                path: p.display().to_string(),
                message: msg,
            })
            .collect(),
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
