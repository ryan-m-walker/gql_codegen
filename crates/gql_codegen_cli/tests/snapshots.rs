//! Snapshot tests for CLI code generation
//!
//! These tests run the code generator against fixtures and snapshot the output.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use gql_codegen_core::{
    CodegenConfig, ExtractConfig, GenerateInput, SourceCache, collect_documents,
    generate_from_input, load_schema, load_sources, resolve_schema_paths,
};

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn load_and_generate(config_name: &str) -> HashMap<String, String> {
    let base_dir = fixtures_dir();
    let config_path = base_dir.join(config_name);
    let config_content = fs::read_to_string(&config_path).expect("Failed to read config");

    let mut config: CodegenConfig =
        serde_json::from_str(&config_content).expect("Failed to parse config");
    config.base_dir = Some(base_dir.to_string_lossy().into_owned());

    let schema_paths = resolve_schema_paths(&config.schema.as_vec(), Some(&base_dir));
    let schema = load_schema(&schema_paths).expect("Failed to load schema");

    let mut source_cache = SourceCache::new();
    load_sources(&config.documents, Some(&base_dir), &mut source_cache)
        .expect("Failed to load sources");

    let extract_config = ExtractConfig::default();
    let documents = collect_documents(&source_cache, &extract_config);

    let input = GenerateInput {
        schema: &schema,
        documents: &documents,
        generates: &config.generates,
    };

    let result = generate_from_input(&input).expect("Failed to generate");

    result
        .files
        .into_iter()
        .map(|f| {
            // Extract just the filename for cleaner snapshot names
            let name = Path::new(&f.path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned();
            (name, f.content)
        })
        .collect()
}

#[test]
fn test_generates_types() {
    let files = load_and_generate("codegen.json");
    let types = files.get("types.ts").expect("types.ts not generated");
    insta::assert_snapshot!("types_ts", types);
}

#[test]
fn test_generates_documents() {
    let files = load_and_generate("codegen.json");
    let docs = files
        .get("documents.ts")
        .expect("documents.ts not generated");
    insta::assert_snapshot!("documents_ts", docs);
}

#[test]
fn test_schema_types_only() {
    let base_dir = fixtures_dir();

    let schema_paths = resolve_schema_paths(&["schemas/schema.graphql"], Some(&base_dir));
    let schema = load_schema(&schema_paths).expect("Failed to load schema");

    let source_cache = SourceCache::new();
    let extract_config = ExtractConfig::default();
    let documents = collect_documents(&source_cache, &extract_config);

    let mut generates = HashMap::new();
    generates.insert(
        "schema-types.ts".to_string(),
        gql_codegen_core::OutputConfig {
            plugins: vec![gql_codegen_core::PluginConfig::Name("typescript".into())],
            prelude: None,
            config: None,
            documents_only: false,
            hooks: None,
        },
    );

    let input = GenerateInput {
        schema: &schema,
        documents: &documents,
        generates: &generates,
    };

    let result = generate_from_input(&input).expect("Failed to generate");
    let content = &result.files[0].content;

    insta::assert_snapshot!("schema_types_only", content);
}
