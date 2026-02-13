//! Tests for typescript-operations plugin (operation type generation)

mod config;
mod lists;

use std::collections::HashMap;
use std::path::PathBuf;

use gql_codegen_core::{
    ExtractConfig, GenerateInput, OutputConfig, PluginConfig, PluginOptions, SourceCache,
    StringOrArray, collect_documents, generate_from_input, load_schema, load_sources,
    resolve_schema_paths,
};

/// Get the fixtures directory path
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Helper to generate operation types from schema and document files
fn generate_operations(
    schema_files: &[&str],
    document_files: &[&str],
    options: PluginOptions,
) -> String {
    let schema_refs: Vec<&str> = schema_files.to_vec();
    let schema = load_schema(&resolve_schema_paths(&schema_refs, Some(&fixtures_dir()))).unwrap();

    let mut cache = SourceCache::new();
    let doc_patterns =
        StringOrArray::Multiple(document_files.iter().map(|s| s.to_string()).collect());
    load_sources(&doc_patterns, Some(&fixtures_dir()), &mut cache).unwrap();
    let docs = collect_documents(&cache, &ExtractConfig::default());

    let mut generates = HashMap::new();
    generates.insert(
        "output.ts".to_string(),
        OutputConfig {
            plugins: vec![PluginConfig::Name("typescript-operations".to_string())],
            config: Some(options),
            prelude: None,
            documents_only: false,
            hooks: None,
        },
    );

    let input = GenerateInput {
        schema: &schema,
        documents: &docs,
        generates: &generates,
    };

    let result = generate_from_input(&input).unwrap();
    assert_eq!(result.files.len(), 1);
    result.files[0].content.clone()
}

#[test]
fn test_operations_default() {
    let output = generate_operations(
        &["schemas/basic.graphql"],
        &["documents/queries.graphql"],
        PluginOptions::default(),
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_operations_with_fragments() {
    let output = generate_operations(
        &["schemas/basic.graphql"],
        &["documents/fragments.graphql"],
        PluginOptions::default(),
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_operations_union() {
    let output = generate_operations(
        &["schemas/basic.graphql", "schemas/union.graphql"],
        &["documents/union_queries.graphql"],
        PluginOptions::default(),
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_operations_interface() {
    let output = generate_operations(
        &["schemas/basic.graphql", "schemas/interface.graphql"],
        &["documents/interface_queries.graphql"],
        PluginOptions::default(),
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_operations_immutable() {
    let output = generate_operations(
        &["schemas/basic.graphql"],
        &["documents/queries.graphql"],
        PluginOptions {
            immutable_types: true,
            ..PluginOptions::default()
        },
    );
    insta::assert_snapshot!(output);
}
