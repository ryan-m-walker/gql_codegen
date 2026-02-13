//! Main code generation orchestration
//!
//! Provides a layered API:
//! - `generate_from_input`: Pure function, no I/O (for testing/embedding)
//! - `generate`: Convenience wrapper that handles file I/O
//! - `generate_cached`: Full caching support with two-phase optimization

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::PathBuf;

use apollo_compiler::Schema;
use apollo_compiler::validation::Valid;

use crate::cache::{
    Cache, MetadataCheckResult, compute_hashes_from_cache, create_glob_cache, is_glob_cache_valid,
};
use crate::config::{AvoidOptionals, OutputConfig, PluginOptions};
use crate::diagnostic::{Diagnostic, DiagnosticCategory, Diagnostics};
use crate::documents::{
    CollectedDocuments, collect_documents, expand_document_globs, load_sources_from_paths,
};
use crate::extract::ExtractConfig;
use crate::generators::{GeneratorContext, run_generator};
use crate::schema::{load_schema_from_contents, resolve_schema_paths};
use crate::source_cache::SourceCache;
use crate::validation::validate_options;
use crate::{CodegenConfig, Result};

/// Result of code generation
#[derive(Debug, Clone)]
pub struct GenerateResult {
    /// Generated files (path -> content)
    pub files: Vec<GeneratedFile>,
    /// Diagnostics encountered during generation (warnings, info)
    pub diagnostics: Diagnostics,
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
pub fn generate_from_input(input: &GenerateInput) -> Result<GenerateResult> {
    let mut diagnostics = input.documents.diagnostics.clone();

    let mut result = GenerateResult {
        files: Vec::with_capacity(input.generates.len()),
        diagnostics: Diagnostics::new(),
    };

    // Generate each output file
    for (output_path, output_config) in input.generates {
        let mut content = String::new();

        // Add prelude if configured
        if let Some(prelude) = &output_config.prelude {
            content.push_str(prelude);
            content.push('\n');
        }

        // Start with SGC defaults, then merge user scalars from config
        let mut base_options = PluginOptions::default();
        if let Some(ref config_options) = output_config.config {
            base_options = merge_options(&base_options, Some(config_options));
        }

        // Validate resolved options and collect config warnings
        validate_options(&base_options, &mut diagnostics);

        for plugin in &output_config.plugins {
            let plugin_name = plugin.name();
            let options = merge_options(&base_options, plugin.options());

            let mut buffer = Vec::new();

            let mut ctx = GeneratorContext {
                schema: input.schema,
                operations: &input.documents.operations,
                fragments: &input.documents.fragments,
                options: &options,
                writer: &mut buffer,
                diagnostics: &mut diagnostics,
            };

            let t0 = web_time::Instant::now();
            run_generator(plugin_name, &mut ctx)?;
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

    result.diagnostics = diagnostics;
    Ok(result)
}

/// Convenience function that handles file I/O
///
/// Reads schema and document files from disk based on config paths.
/// For more control over I/O and caching, use [`generate_from_input`] instead.
pub fn generate(config: &CodegenConfig) -> Result<GenerateResult> {
    let base_dir = config
        .base_dir
        .as_ref()
        .map(|s| PathBuf::from(s.as_str()))
        .unwrap_or_else(|| PathBuf::from("."));

    // Build schema from both file paths and pre-resolved SDL content
    let schema_paths = resolve_schema_paths(&config.schema.as_vec(), Some(&base_dir));
    let mut schema_files: Vec<(PathBuf, String)> = Vec::new();

    for path in &schema_paths {
        let content = fs::read_to_string(path).map_err(|e| {
            Diagnostics::from(Diagnostic::error(
                DiagnosticCategory::Schema,
                format!("Failed to read schema '{}': {}", path.display(), e),
            ))
        })?;
        schema_files.push((path.clone(), content));
    }

    if let Some(contents) = &config.schema_content {
        for (i, sdl) in contents.iter().enumerate() {
            schema_files.push((PathBuf::from(format!("<schema:{i}>")), sdl.clone()));
        }
    }

    let schema = load_schema_from_contents(&schema_files)?;

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

/// What `#[serde(skip)]` produces for each field — the type-level defaults
/// (bool → false, Option → None, etc.), NOT the SGC defaults.
///
/// Used by `merge_options` to detect which fields were explicitly set.
fn serde_field_defaults() -> PluginOptions {
    PluginOptions {
        scalars: BTreeMap::new(),
        use_utility_types: false,
        inline_fragments: false,
        dedupe_selections: false,
        disable_descriptions: false,
        strict_scalars: false,
        default_scalar_type: None,
        immutable_types: false,
        enums_as_types: None,
        enums_as_const: false,
        future_proof_enums: false,
        future_proof_unions: false,
        enum_prefix: None,
        enum_suffix: None,
        const_enums: false,
        numeric_enums: false,
        only_enums: false,
        no_export: false,
        only_operation_types: false,
        skip_typename: false,
        non_optional_typename: false,
        avoid_optionals: AvoidOptionals::default(),
        maybe_value: None,
        input_maybe_value: None,
        declaration_kind: None,
        types_prefix: None,
        types_suffix: None,
        use_type_imports: false,
        graphql_tag: None,
        naming_convention: None,
        typename_policy: None,
        pretty_documents: false,
    }
}

/// Merge base options with plugin-specific overrides.
///
/// Compares each field against `serde_field_defaults()` — the type-level
/// defaults that `#[serde(skip)]` produces. If a field differs from that
/// baseline, it was explicitly set (either in test code or by a future
/// config path) and takes precedence over the base (SGC defaults).
///
/// In the production path, deserialized JSON configs have all `#[serde(skip)]`
/// fields at type defaults, so only `scalars` can differ and override the base.
/// In tests, Rust code can construct `PluginOptions { immutable_types: false, .. }`
/// and the override is preserved.
fn merge_options(base: &PluginOptions, plugin: Option<&PluginOptions>) -> PluginOptions {
    let Some(p) = plugin else {
        return base.clone();
    };

    let defaults = serde_field_defaults();

    macro_rules! merge_field {
        ($result:expr, $plugin:expr, $defaults:expr, $($field:ident),+ $(,)?) => {
            $(
                if $plugin.$field != $defaults.$field {
                    $result.$field = $plugin.$field.clone();
                }
            )+
        };
    }

    let mut result = base.clone();
    merge_field!(
        result, p, defaults,
        scalars,
        use_utility_types,
        inline_fragments,
        dedupe_selections,
        disable_descriptions,
        strict_scalars,
        default_scalar_type,
        immutable_types,
        enums_as_types,
        enums_as_const,
        future_proof_enums,
        future_proof_unions,
        enum_prefix,
        enum_suffix,
        const_enums,
        numeric_enums,
        only_enums,
        no_export,
        only_operation_types,
        skip_typename,
        non_optional_typename,
        avoid_optionals,
        maybe_value,
        input_maybe_value,
        declaration_kind,
        types_prefix,
        types_suffix,
        use_type_imports,
        graphql_tag,
        naming_convention,
        typename_policy,
        pretty_documents,
    );
    result
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
    let glob_cache = cache.stored().and_then(|c| c.glob_cache.as_ref());

    let (document_paths, glob_cache_hit) = match glob_cache {
        Some(cached) if is_glob_cache_valid(cached, &patterns) => {
            crate::timing!(
                "Glob cache hit",
                t0.elapsed(),
                "{} files",
                cached.files.len()
            );
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
        // TODO: can we avoid cloning here?
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
    let inline_content = config.schema_content.clone();

    let load_schema = || {
        let mut schema_files: Vec<(PathBuf, String)> = schema_paths
            .into_iter()
            .filter_map(|p| fs::read_to_string(&p).ok().map(|c| (p, c)))
            .collect();

        // Append pre-resolved SDL content from Node CLI (.ts/.js schemas)
        if let Some(contents) = inline_content {
            for (i, sdl) in contents.into_iter().enumerate() {
                schema_files.push((PathBuf::from(format!("<schema:{i}>")), sdl));
            }
        }

        load_schema_from_contents(&schema_files).map(|s| (s, schema_files))
    };

    let load_sources = || {
        let mut source_cache = SourceCache::with_capacity(doc_paths_len);
        load_sources_from_paths(&document_paths, &mut source_cache).map(|_| source_cache)
    };

    let (schema_result, docs_result) = rayon::join(load_schema, load_sources);
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
    crate::timing!(
        "GraphQL extraction",
        t0.elapsed(),
        "{} ops, {} frags",
        documents.operations.len(),
        documents.fragments.len()
    );

    let t0 = web_time::Instant::now();
    let input = GenerateInput {
        schema: &schema,
        documents: &documents,
        generates: &config.generates,
    };
    let result = generate_from_input(&input)?;
    crate::timing!("Code generation", t0.elapsed());

    // Store cache after successful generation
    cache.store(computed).ok();

    crate::timing!("Total", start.elapsed());

    Ok(GenerateCachedResult::Generated(result))
}
