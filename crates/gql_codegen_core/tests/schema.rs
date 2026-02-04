//! Integration tests for schema loading

use std::path::PathBuf;

use gql_codegen_core::{Error, load_schema, resolve_schema_paths};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn schema_paths(names: &[&str]) -> Vec<PathBuf> {
    resolve_schema_paths(names, Some(&fixtures_dir()))
}

#[test]
fn test_load_single_schema() {
    let paths = schema_paths(&["schemas/basic.graphql"]);
    let result = load_schema(&paths);

    assert!(result.is_ok());
    let schema = result.unwrap();
    assert!(schema.types.contains_key("User"));
    assert!(schema.types.contains_key("Post"));
    assert!(schema.types.contains_key("Query"));
    assert!(schema.types.contains_key("Mutation"));
    assert!(schema.types.contains_key("Status")); // enum
}

#[test]
fn test_load_multiple_schema_files() {
    let paths = schema_paths(&["schemas/base.graphql", "schemas/object.graphql"]);
    let result = load_schema(&paths);

    assert!(result.is_ok());
    let schema = result.unwrap();
    assert!(schema.types.contains_key("Query"));
    assert!(schema.types.contains_key("User"));
    assert!(schema.types.contains_key("Post"));
}

#[test]
fn test_load_schema_file_not_found() {
    let paths = schema_paths(&["schemas/nonexistent.graphql"]);
    let result = load_schema(&paths);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::SchemaRead(_, _)));
}

#[test]
fn test_load_schema_invalid_syntax() {
    let paths = schema_paths(&["schemas/broken.graphql"]);
    let result = load_schema(&paths);

    assert!(result.is_err());
}

#[test]
fn test_load_schema_validation_error() {
    let paths = schema_paths(&["schemas/invalid.graphql"]);
    let result = load_schema(&paths);

    assert!(result.is_err());
}
