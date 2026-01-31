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

use crate::cache::{Cache, MetadataCheckResult, compute_hashes_from_cache};
use crate::config::{OutputConfig, PluginOptions};
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

        let base_options = output_config.config.clone().unwrap_or_default();

        for plugin in &output_config.plugins {
            let plugin_name = plugin.name();
            let options = merge_options(&base_options, plugin.options());

            let ctx = GeneratorContext {
                schema: input.schema,
                operations: &input.documents.operations,
                fragments: &input.documents.fragments,
                options: &options,
            };

            let mut buffer = Vec::new();
            run_generator(plugin_name, &ctx, &mut buffer)?;

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
    };

    generate_from_input(&input)
}

/// Merge base options with plugin-specific options
fn merge_options(base: &PluginOptions, plugin: Option<&PluginOptions>) -> PluginOptions {
    match plugin {
        Some(p) => {
            let mut merged = base.clone();

            // Plugin options override base options
            if !p.scalars.is_empty() {
                merged.scalars = p.scalars.clone();
            }

            if p.immutable_types {
                merged.immutable_types = true;
            }

            if !p.enums_as_types {
                merged.enums_as_types = false;
            }

            if p.future_proof_enums {
                merged.future_proof_enums = true;
            }

            if p.skip_typename {
                merged.skip_typename = true;
            }

            if p.avoid_optionals {
                merged.avoid_optionals = true;
            }

            if p.graphql_tag.is_some() {
                merged.graphql_tag = p.graphql_tag;
            }

            if p.formatting.is_some() {
                merged.formatting = p.formatting.clone();
            }

            merged
        }
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
    let base_dir = config
        .base_dir
        .as_ref()
        .map(|s| PathBuf::from(s.as_str()))
        .unwrap_or_else(|| PathBuf::from("."));

    let schema_paths = resolve_schema_paths(&config.schema.as_vec(), Some(&base_dir));
    let document_paths = expand_document_globs(&config.documents, &base_dir)?;

    let all_paths: Vec<PathBuf> = schema_paths
        .iter()
        .chain(document_paths.iter())
        .cloned()
        .collect();

    // Phase 1: Quick metadata check (no file reads, just stat)
    let metadata_result = cache.check_metadata(&all_paths);
    if matches!(metadata_result, MetadataCheckResult::AllMatch) {
        return Ok(GenerateCachedResult::Fresh);
    }

    let schema_files: Vec<(PathBuf, String)> = schema_paths
        .into_iter()
        .filter_map(|p| std::fs::read_to_string(&p).ok().map(|c| (p, c)))
        .collect();

    let schema = load_schema_from_contents(&schema_files)?;
    let mut source_cache = SourceCache::with_capacity(document_paths.len());
    load_sources_from_paths(&document_paths, &mut source_cache)?;

    // Phase 2: Compute hashes from loaded content
    let computed = compute_hashes_from_cache(config, &source_cache, &schema_files);

    if cache.is_fresh(&computed) {
        // Metadata was stale but content matches - update cache and return fresh
        cache.store(computed).ok();
        return Ok(GenerateCachedResult::Fresh);
    }

    // TODO: actually use real passed config
    let extract_config = ExtractConfig::default();
    let documents = collect_documents(&source_cache, &extract_config);

    let input = GenerateInput {
        schema: &schema,
        documents: &documents,
        generates: &config.generates,
    };

    let result = generate_from_input(&input)?;

    // Store cache after successful generation
    cache.store(computed).ok();

    Ok(GenerateCachedResult::Generated(result))
}
