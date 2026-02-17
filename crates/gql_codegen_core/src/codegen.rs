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
use crate::config::{GeneratorConfig, GeneratorOptions, OutputConfig};
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

/// Default generators when none are specified in config
const DEFAULT_GENERATOR_NAMES: [&str; 2] = ["schema-types", "operation-types"];

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
    pub outputs: &'a HashMap<String, OutputConfig>,
}

/// Pure generation function - NO filesystem access
///
/// Takes pre-loaded schema and documents, returns generated content.
/// Use this for maximum control, testing, or embedding in other tools.
pub fn generate_from_input(input: &GenerateInput) -> Result<GenerateResult> {
    let mut diagnostics = input.documents.diagnostics.clone();

    let mut result = GenerateResult {
        files: Vec::with_capacity(input.outputs.len()),
        diagnostics: Diagnostics::new(),
    };

    // Generate each output file
    for (output_path, output_config) in input.outputs {
        let mut content = String::new();

        // Add prelude if configured
        if let Some(prelude) = &output_config.prelude {
            content.push_str(prelude);
            content.push('\n');
        }

        // Start with SGC defaults, then merge user config
        let mut base_options = GeneratorOptions::default();
        if let Some(ref config_options) = output_config.config {
            base_options = merge_options(&base_options, Some(config_options));
        }

        // Validate resolved options and collect config warnings
        validate_options(&base_options, &mut diagnostics);

        // Resolve generators — use defaults if omitted
        let default_generators: Vec<GeneratorConfig> = DEFAULT_GENERATOR_NAMES
            .iter()
            .map(|name| GeneratorConfig::Name(name.to_string()))
            .collect();
        let generators = output_config
            .generators
            .as_deref()
            .unwrap_or(&default_generators);

        for generator in generators {
            let generator_name = generator.name();
            let options = merge_options(&base_options, generator.options());

            let mut buffer = Vec::new();

            let mut ctx = GeneratorContext {
                schema: input.schema,
                operations: &input.documents.operations,
                fragments: &input.documents.fragments,
                options: &options,
                writer: &mut buffer,
                diagnostics: &mut diagnostics,
                generators,
            };

            let t0 = web_time::Instant::now();
            run_generator(generator_name, &mut ctx)?;
            crate::timing!(format!("  Generator '{}'", generator_name), t0.elapsed());

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
    // TODO: simplify this
    let base_dir = PathBuf::from(".");

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
        outputs: &config.outputs,
    };

    generate_from_input(&input)
}

/// What `#[serde(skip)]` and `#[serde(default)]` produce for each field —
/// the type-level defaults (bool → false, Option → None, etc.),
/// NOT the SGC defaults.
///
/// Used by `merge_options` to detect which fields were explicitly set.
fn serde_field_defaults() -> GeneratorOptions {
    GeneratorOptions {
        scalars: BTreeMap::new(),
        // Exposed Option fields: None means "not set" in deserialized JSON
        immutable_types: None,
        enums_as_types: None,
        future_proof_enums: None,
        future_proof_unions: None,
        declaration_kind: None,
        type_name_prefix: None,
        type_name_suffix: None,
        // Internal fields: type defaults
        strict_scalars: false,
        default_scalar_type: None,
        naming_convention: None,
        typename_policy: None,
        only_referenced_types: false,
    }
}

/// Merge base options with generator-specific overrides.
///
/// Compares each field against `serde_field_defaults()` — the type-level
/// defaults that serde produces for unset fields. If a field differs from
/// that baseline, it was explicitly set and takes precedence over the base
/// (SGC defaults).
fn merge_options(
    base: &GeneratorOptions,
    generator: Option<&GeneratorOptions>,
) -> GeneratorOptions {
    let Some(g) = generator else {
        return base.clone();
    };

    let defaults = serde_field_defaults();

    macro_rules! merge_field {
        ($result:expr, $generator:expr, $defaults:expr, $($field:ident),+ $(,)?) => {
            $(
                if $generator.$field != $defaults.$field {
                    $result.$field = $generator.$field.clone();
                }
            )+
        };
    }

    let mut result = base.clone();

    merge_field!(
        result,
        g,
        defaults,
        scalars,
        strict_scalars,
        default_scalar_type,
        immutable_types,
        enums_as_types,
        future_proof_enums,
        future_proof_unions,
        declaration_kind,
        type_name_prefix,
        type_name_suffix,
        naming_convention,
        typename_policy,
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

    // TODO: simplify this
    let base_dir = PathBuf::from(".");

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
        outputs: &config.outputs,
    };
    let result = generate_from_input(&input)?;
    crate::timing!("Code generation", t0.elapsed());

    // Store cache after successful generation
    cache.store(computed).ok();

    crate::timing!("Total", start.elapsed());

    Ok(GenerateCachedResult::Generated(result))
}
