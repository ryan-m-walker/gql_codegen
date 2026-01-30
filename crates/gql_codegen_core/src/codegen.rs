//! Main code generation orchestration
//!
//! Provides both pure (no I/O) and convenience (with I/O) APIs.

use std::collections::HashMap;
use std::path::Path;

use apollo_compiler::validation::Valid;
use apollo_compiler::Schema;

use crate::config::{OutputConfig, PluginOptions};
use crate::documents::{CollectedDocuments, SourceCache, collect_documents, load_sources};
use crate::extract::ExtractConfig;
use crate::generators::{GeneratorContext, run_generator};
use crate::schema::load_schema;
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
        files: Vec::new(),
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

        // Get merged options (output-level + plugin-level)
        let base_options = output_config.config.clone().unwrap_or_default();

        // Run each plugin
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
            content.push_str(&String::from_utf8_lossy(&buffer));
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
    let base_dir = config.base_dir.as_ref().map(|s| Path::new(s.as_str()));

    // Load and validate schema
    let schema = load_schema(&config.schema, base_dir)?;

    // Load all source files into cache
    let mut source_cache = SourceCache::new();
    load_sources(&config.documents, base_dir, &mut source_cache)?;

    // Extract config from first output's config (or use defaults)
    // TODO: Make this configurable per-output
    let extract_config = ExtractConfig::default();

    // Collect all operations and fragments
    let documents = collect_documents(&source_cache, &extract_config);

    // Use the pure generation function
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

            if p.use_null_for_optional {
                merged.use_null_for_optional = true;
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
