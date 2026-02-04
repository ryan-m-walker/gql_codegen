mod enums;
mod inputs;
mod interfaces;
mod misc;
mod naming;
mod objects;
mod scalars;
mod unions;

use std::collections::HashMap;
use std::path::PathBuf;

use gql_codegen_core::{
    Error, ExtractConfig, GenerateInput, GenerateResult, OutputConfig, PluginConfig, PluginOptions,
    Preset, SourceCache, collect_documents, generate_from_input, load_schema, resolve_schema_paths,
};

/// Get the fixtures directory path
pub fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Helper to generate TypeScript output from schema files with given options
///
/// Always includes base.graphql plus any additional schema files specified.
pub fn generate_with_options(schema_files: &[&str], options: PluginOptions) -> String {
    // Always include base schema
    let mut all_schemas = vec!["schemas/base.graphql"];
    all_schemas.extend(schema_files.iter());

    let schema_refs: Vec<&str> = all_schemas.to_vec();
    let schema = load_schema(&resolve_schema_paths(&schema_refs, Some(&fixtures_dir()))).unwrap();

    let cache = SourceCache::new();
    let extract_config = ExtractConfig::default();
    let docs = collect_documents(&cache, &extract_config);

    let mut generates = HashMap::new();
    generates.insert(
        "output.ts".to_string(),
        OutputConfig {
            plugins: vec![PluginConfig::Name("typescript".to_string())],
            config: Some(options),
            prelude: None,
            documents_only: false,
        },
    );

    let input = GenerateInput {
        schema: &schema,
        documents: &docs,
        generates: &generates,
        preset: Preset::default(),
    };

    let result = generate_from_input(&input).unwrap();
    assert_eq!(result.files.len(), 1);
    result.files[0].content.clone()
}

/// Like generate_with_options but returns Result for error testing
pub fn try_generate_with_options(
    schema_files: &[&str],
    options: PluginOptions,
) -> Result<GenerateResult, Error> {
    let mut all_schemas = vec!["schemas/base.graphql"];
    all_schemas.extend(schema_files.iter());

    let schema_refs: Vec<&str> = all_schemas.to_vec();
    let schema = load_schema(&resolve_schema_paths(&schema_refs, Some(&fixtures_dir()))).unwrap();

    let cache = SourceCache::new();
    let extract_config = ExtractConfig::default();
    let docs = collect_documents(&cache, &extract_config);

    let mut generates = HashMap::new();
    generates.insert(
        "output.ts".to_string(),
        OutputConfig {
            plugins: vec![PluginConfig::Name("typescript".to_string())],
            config: Some(options),
            prelude: None,
            documents_only: false,
        },
    );

    let input = GenerateInput {
        schema: &schema,
        documents: &docs,
        generates: &generates,
        preset: Preset::default(),
    };

    generate_from_input(&input)
}
