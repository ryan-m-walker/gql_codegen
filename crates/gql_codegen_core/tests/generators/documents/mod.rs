//! Tests for typed-documents generator (GraphQL document constants generation)

use std::collections::HashMap;
use std::path::PathBuf;

use gql_codegen_core::{
    ExtractConfig, GenerateInput, GraphqlTag, OutputConfig, GeneratorConfig, GeneratorOptions,
    SourceCache, StringOrArray, collect_documents, generate_from_input, load_schema, load_sources,
    resolve_schema_paths,
};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn generate_docs(schema_files: &[&str], document_files: &[&str], options: GeneratorOptions) -> String {
    let schema_refs: Vec<&str> = schema_files.to_vec();
    let schema = load_schema(&resolve_schema_paths(&schema_refs, Some(&fixtures_dir()))).unwrap();

    let mut cache = SourceCache::new();
    let doc_patterns =
        StringOrArray::Multiple(document_files.iter().map(|s| s.to_string()).collect());
    load_sources(&doc_patterns, Some(&fixtures_dir()), &mut cache).unwrap();
    let docs = collect_documents(&cache, &ExtractConfig::default());

    let mut outputs = HashMap::new();
    outputs.insert(
        "output.ts".to_string(),
        OutputConfig {
            generators: Some(vec![GeneratorConfig::Name("typed-documents".to_string())]),
            config: Some(options),
            prelude: None,
        },
    );

    let input = GenerateInput {
        schema: &schema,
        documents: &docs,
        outputs: &outputs,
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
        GeneratorOptions::default(),
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_documents_with_gql_tag() {
    let output = generate_docs(
        &["schemas/basic.graphql"],
        &["documents/queries.graphql"],
        GeneratorOptions {
            graphql_tag: Some(GraphqlTag::Gql),
            ..GeneratorOptions::default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_documents_with_fragments() {
    let output = generate_docs(
        &["schemas/basic.graphql"],
        &["documents/fragments.graphql"],
        GeneratorOptions::default(),
    );
    insta::assert_snapshot!(output);
}
