//! Main code generation orchestration
//!
//! Provides a layered API:
//! - `generate_from_input`: Pure function, no I/O (for testing/embedding)
//! - `generate`: Convenience wrapper that handles file I/O
//! - `generate_cached`: Full caching support with two-phase optimization

use std::collections::HashMap;
use std::path::PathBuf;

use apollo_compiler::Schema;
use apollo_compiler::validation::Valid;

use crate::cache::{Cache, MetadataCheckResult, compute_hashes_from_cache, create_glob_cache, is_glob_cache_valid};
use crate::config::{OutputConfig, PluginOptions, Preset};
use crate::documents::{
    CollectedDocuments, collect_documents, expand_document_globs, load_sources_from_paths,
};
use crate::extract::ExtractConfig;
use crate::generators::{GeneratorContext, run_generator};
use crate::schema::{load_schema, load_schema_from_contents, resolve_schema_paths};
use crate::source_cache::SourceCache;
use crate::{CodegenConfig, Result};

/// Result of code generation
#[derive(Debug, Clone)]
pub struct GenerateResult {
    /// Generated files (path -> content)
    pub files: Vec<GeneratedFile>,
    /// Warnings encountered during generation
    pub warnings: Vec<String>,
}

/// A single generated file
#[derive(Debug, Clone)]
pub struct GeneratedFile {
    /// Output path
    pub path: String,
    /// Generated content
    pub content: String,
}

/// Result of cached generation
#[derive(Debug)]
pub enum GenerateCachedResult {
    /// Cache hit - inputs unchanged, no regeneration needed
    Fresh,
    /// Generated new output
    Generated(GenerateResult),
}

/// Pre-loaded input for pure generation (no filesystem access)
///
/// Use this when you want full control over I/O, caching, etc.
pub struct GenerateInput<'a> {
    /// Validated schema
    pub schema: &'a Valid<Schema>,
    /// Collected operations and fragments
    pub documents: &'a CollectedDocuments<'a>,
    /// Output configurations
    pub generates: &'a HashMap<String, OutputConfig>,
    /// Preset for default options
    pub preset: Preset,
}

/// Pure generation function - NO filesystem access
///
/// Takes pre-loaded schema and documents, returns generated content.
/// Use this for maximum control, testing, or embedding in other tools.
///
/// # Example
/// ```ignore
/// let schema = load_schema_from_str(&schema_content)?;
/// let documents = parse_documents(&source_cache, &extract_config);
/// let input = GenerateInput { schema: &schema, documents: &documents, generates: &config };
/// let result = generate_from_input(&input)?;
/// ```
pub fn generate_from_input(input: &GenerateInput) -> Result<GenerateResult> {
    let mut result = GenerateResult {
        files: Vec::with_capacity(input.generates.len()),
        warnings: input.documents.warnings.clone(),
    };

    // Generate each output file
    for (output_path, output_config) in input.generates {
        let mut content = String::new();

        // Add prelude if configured
        if let Some(prelude) = &output_config.prelude {
            content.push_str(prelude);
            content.push('\n');
        }

        // Start with preset defaults, then override with config
        let mut base_options = input.preset.default_options();
        if let Some(ref config_options) = output_config.config {
            base_options = merge_options(&base_options, Some(config_options));
        }

        for plugin in &output_config.plugins {
            let plugin_name = plugin.name();
            let options = merge_options(&base_options, plugin.options());

            let ctx = GeneratorContext {
                schema: input.schema,
                operations: &input.documents.operations,
                fragments: &input.documents.fragments,
                options: &options,
            };

            let t0 = web_time::Instant::now();
            let mut buffer = Vec::new();
            run_generator(plugin_name, &ctx, &mut buffer)?;
            crate::timing!(format!("  Plugin '{}'", plugin_name), t0.elapsed());

            // Safe: our generators only output valid UTF-8
            content.push_str(
                &String::from_utf8(buffer).expect("generator output should be valid UTF-8"),
            );
        }

        result.files.push(GeneratedFile {
            path: output_path.clone(),
            content,
        });
    }

    Ok(result)
}

/// Convenience function that handles file I/O
///
/// Reads schema and document files from disk based on config paths.
/// For more control over I/O and caching, use [`generate_from_input`] instead.
///
/// # Filesystem Access
/// This function reads files from disk. If you need a pure API without
/// filesystem side effects, load files yourself and use [`generate_from_input`].
pub fn generate(config: &CodegenConfig) -> Result<GenerateResult> {
    let base_dir = config
        .base_dir
        .as_ref()
        .map(|s| PathBuf::from(s.as_str()))
        .unwrap_or_else(|| PathBuf::from("."));

    let schema_paths = resolve_schema_paths(&config.schema.as_vec(), Some(&base_dir));
    let schema = load_schema(&schema_paths)?;

    let document_paths = expand_document_globs(&config.documents, &base_dir)?;
    let mut source_cache = SourceCache::with_capacity(document_paths.len());
    load_sources_from_paths(&document_paths, &mut source_cache)?;

    // TODO: Make this configurable per-output
    let extract_config = ExtractConfig::default();

    let documents = collect_documents(&source_cache, &extract_config);

    let input = GenerateInput {
        schema: &schema,
        documents: &documents,
        generates: &config.generates,
        preset: config.preset,
    };

    generate_from_input(&input)
}

/// Merge base options with plugin-specific options
///
/// Plugin options fully override base options. This is intentional:
/// - Preset defaults provide the base
/// - User config (via plugin options) overrides everything
///
/// Note: Since serde gives default values for unset fields, the caller
/// should ensure plugin options contain all desired values (including
/// preset defaults for fields not being overridden).
fn merge_options(base: &PluginOptions, plugin: Option<&PluginOptions>) -> PluginOptions {
    match plugin {
        // Plugin options completely override base - the caller is responsible
        // for including preset defaults in the plugin options if needed
        Some(p) => p.clone(),
        None => base.clone(),
    }
}

/// Generate with caching support (two-phase optimization)
///
/// This handles the full caching flow:
/// 1. Phase 1: Quick metadata check (stat only, no file reads)
/// 2. Phase 2: Content hash verification (from already-loaded files)
/// 3. Generation if cache miss
/// 4. Cache update on success
///
/// Returns `Fresh` if nothing changed, `Generated` with the output otherwise.
///
/// # Example
/// ```ignore
/// let mut cache = FsCache::new(base_dir.join(".sgc"));
///
/// match generate_cached(&config, &mut cache)? {
///     GenerateCachedResult::Fresh => println!("Nothing changed"),
///     GenerateCachedResult::Generated(result) => {
///         for file in result.files {
///             fs::write(&file.path, &file.content)?;
///         }
///     }
/// }
/// ```
pub fn generate_cached(
    config: &CodegenConfig,
    cache: &mut dyn Cache,
) -> Result<GenerateCachedResult> {
    let start = web_time::Instant::now();

    let base_dir = config
        .base_dir
        .as_ref()
        .map(|s| PathBuf::from(s.as_str()))
        .unwrap_or_else(|| PathBuf::from("."));

    let schema_paths = resolve_schema_paths(&config.schema.as_vec(), Some(&base_dir));

    // Try to use cached glob results
    let t0 = web_time::Instant::now();
    let patterns: Vec<&str> = config.documents.as_vec();
    let (document_paths, glob_cache_hit) = match cache.stored().and_then(|c| c.glob_cache.as_ref()) {
        Some(cached) if is_glob_cache_valid(cached, &patterns) => {
            crate::timing!("Glob cache hit", t0.elapsed(), "{} files", cached.files.len());
            (cached.files.clone(), true)
        }
        _ => {
            let paths = expand_document_globs(&config.documents, &base_dir)?;
            crate::timing!("Glob expansion", t0.elapsed(), "{} files", paths.len());
            (paths, false)
        }
    };

    let all_paths: Vec<PathBuf> = schema_paths
        .iter()
        .chain(document_paths.iter())
        .cloned()
        .collect();

    // Phase 1: Quick metadata check (no file reads, just stat)
    let t0 = web_time::Instant::now();
    let metadata_result = cache.check_metadata(&all_paths);
    crate::timing!("Cache metadata check", t0.elapsed());
    if matches!(metadata_result, MetadataCheckResult::AllMatch) {
        crate::timing!("Total (cache hit - metadata)", start.elapsed());
        return Ok(GenerateCachedResult::Fresh);
    }

    // Load schema and documents in parallel
    let t0 = web_time::Instant::now();
    let doc_paths_len = document_paths.len();
    let (schema_result, docs_result) = rayon::join(
        || {
            let schema_files: Vec<(PathBuf, String)> = schema_paths
                .into_iter()
                .filter_map(|p| std::fs::read_to_string(&p).ok().map(|c| (p, c)))
                .collect();
            load_schema_from_contents(&schema_files).map(|s| (s, schema_files))
        },
        || {
            let mut source_cache = SourceCache::with_capacity(doc_paths_len);
            load_sources_from_paths(&document_paths, &mut source_cache).map(|_| source_cache)
        },
    );
    let (schema, schema_files) = schema_result?;
    let source_cache = docs_result?;
    crate::timing!("Schema + docs parallel load", t0.elapsed());

    // Phase 2: Compute hashes from loaded content
    let t0 = web_time::Instant::now();
    let mut computed = compute_hashes_from_cache(config, &source_cache, &schema_files);

    // Store glob cache if it was a miss
    if !glob_cache_hit {
        computed.glob_cache = Some(create_glob_cache(&patterns, document_paths.clone()));
    } else {
        // Preserve existing glob cache
        computed.glob_cache = cache.stored().and_then(|c| c.glob_cache.clone());
    }
    crate::timing!("Hash computation", t0.elapsed());

    if cache.is_fresh(&computed) {
        // Metadata was stale but content matches - update cache and return fresh
        cache.store(computed).ok();
        crate::timing!("Total (cache hit - content)", start.elapsed());
        return Ok(GenerateCachedResult::Fresh);
    }

    // TODO: actually use real passed config
    let t0 = web_time::Instant::now();
    let extract_config = ExtractConfig::default();
    let documents = collect_documents(&source_cache, &extract_config);
    crate::timing!("GraphQL extraction", t0.elapsed(), "{} ops, {} frags",
        documents.operations.len(), documents.fragments.len());

    let t0 = web_time::Instant::now();
    let input = GenerateInput {
        schema: &schema,
        documents: &documents,
        generates: &config.generates,
        preset: config.preset,
    };
    let result = generate_from_input(&input)?;
    crate::timing!("Code generation", t0.elapsed());

    // Store cache after successful generation
    cache.store(computed).ok();

    crate::timing!("Total", start.elapsed());

    Ok(GenerateCachedResult::Generated(result))
}
