//! Schema loading and validation

use std::path::{Path, PathBuf};

use apollo_compiler::{Schema, validation::Valid};

use crate::{Error, Result};

/// Load and validate a GraphQL schema from one or more file paths.
pub fn load_schema(paths: &[PathBuf]) -> Result<Valid<Schema>> {
    let mut builder = Schema::builder();

    for path in paths {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::SchemaRead(path.clone(), e.to_string()))?;

        builder = builder.parse(content, path);
    }

    let schema = builder
        .build()
        .map_err(|e| Error::SchemaParse(e.to_string()))?;

    schema
        .validate()
        .map_err(|e| Error::SchemaValidation(format_validation_errors(&e.errors)))
}

/// Load and validate a GraphQL schema from pre-loaded content.
///
/// Use this when you've already read the schema files (e.g., for caching)
/// to avoid reading them twice.
pub fn load_schema_from_contents(files: &[(PathBuf, String)]) -> Result<Valid<Schema>> {
    let mut builder = Schema::builder();

    for (path, content) in files {
        builder = builder.parse(content, path);
    }

    let schema = builder
        .build()
        .map_err(|e| Error::SchemaParse(e.to_string()))?;

    schema
        .validate()
        .map_err(|e| Error::SchemaValidation(format_validation_errors(&e.errors)))
}

/// Helper to resolve schema paths from config (convenience for simple cases)
pub fn resolve_schema_paths(
    patterns: &[&str],
    base_dir: Option<&Path>,
) -> Vec<PathBuf> {
    patterns
        .iter()
        .map(|p| {
            if let Some(base) = base_dir {
                base.join(p)
            } else {
                Path::new(p).to_path_buf()
            }
        })
        .collect()
}

fn format_validation_errors(errors: &apollo_compiler::validation::DiagnosticList) -> String {
    errors
        .iter()
        .map(|e| e.error.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
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
        let paths = schema_paths(&["schemas/base.graphql", "schemas/types.graphql"]);
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
        // Could be parse or validation error depending on how apollo handles it
    }

    #[test]
    fn test_load_schema_validation_error() {
        let paths = schema_paths(&["schemas/invalid.graphql"]);
        let result = load_schema(&paths);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::SchemaValidation(_)));
    }
}
