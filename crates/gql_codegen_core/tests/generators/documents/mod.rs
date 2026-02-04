//! Tests for documents plugin (GraphQL document constants generation)

use std::collections::HashMap;
use std::path::PathBuf;

use gql_codegen_core::{
    ExtractConfig, GenerateInput, GraphqlTag, OutputConfig, PluginConfig, PluginOptions, Preset,
    SourceCache, StringOrArray, collect_documents, generate_from_input, load_schema, load_sources,
    resolve_schema_paths,
};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn generate_docs(schema_files: &[&str], document_files: &[&str], options: PluginOptions) -> String {
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
            plugins: vec![PluginConfig::Name("documents".to_string())],
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

#[test]
fn test_documents_plain() {
    let output = generate_docs(
        &["schemas/basic.graphql"],
        &["documents/queries.graphql"],
        PluginOptions::default(),
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_documents_with_gql_tag() {
    let output = generate_docs(
        &["schemas/basic.graphql"],
        &["documents/queries.graphql"],
        PluginOptions {
            graphql_tag: Some(GraphqlTag::Gql),
            ..PluginOptions::serde_default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_documents_with_fragments() {
    let output = generate_docs(
        &["schemas/basic.graphql"],
        &["documents/fragments.graphql"],
        PluginOptions::default(),
    );
    insta::assert_snapshot!(output);
}
