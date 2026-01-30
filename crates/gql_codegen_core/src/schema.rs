//! Schema loading and validation

use std::path::Path;

use apollo_compiler::{Schema, validation::Valid};

use crate::{Error, Result, config::StringOrArray};

/// Load and validate a GraphQL schema from one or more files.
pub fn load_schema(sources: &StringOrArray, base_dir: Option<&Path>) -> Result<Valid<Schema>> {
    let paths = sources.as_vec();
    let mut builder = Schema::builder();

    for path_str in paths {
        let path = if let Some(base) = base_dir {
            base.join(path_str)
        } else {
            Path::new(path_str).to_path_buf()
        };

        let content = std::fs::read_to_string(&path)
            .map_err(|e| Error::SchemaRead(path.clone(), e.to_string()))?;

        builder = builder.parse(content, &path);
    }

    let schema = builder
        .build()
        .map_err(|e| Error::SchemaParse(e.to_string()))?;

    schema
        .validate()
        .map_err(|e| Error::SchemaValidation(format_validation_errors(&e.errors)))
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
    use std::path::PathBuf;

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
    }

    #[test]
    fn test_load_single_schema() {
        let sources = StringOrArray::Single("schemas/basic.graphql".into());
        let result = load_schema(&sources, Some(&fixtures_dir()));

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
        let sources = StringOrArray::Multiple(vec![
            "schemas/base.graphql".into(),
            "schemas/types.graphql".into(),
        ]);
        let result = load_schema(&sources, Some(&fixtures_dir()));

        assert!(result.is_ok());
        let schema = result.unwrap();
        assert!(schema.types.contains_key("Query"));
        assert!(schema.types.contains_key("User"));
        assert!(schema.types.contains_key("Post"));
    }

    #[test]
    fn test_load_schema_file_not_found() {
        let sources = StringOrArray::Single("schemas/nonexistent.graphql".into());
        let result = load_schema(&sources, Some(&fixtures_dir()));

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::SchemaRead(_, _)));
    }

    #[test]
    fn test_load_schema_invalid_syntax() {
        let sources = StringOrArray::Single("schemas/broken.graphql".into());
        let result = load_schema(&sources, Some(&fixtures_dir()));

        assert!(result.is_err());
        // Could be parse or validation error depending on how apollo handles it
    }

    #[test]
    fn test_load_schema_validation_error() {
        let sources = StringOrArray::Single("schemas/invalid.graphql".into());
        let result = load_schema(&sources, Some(&fixtures_dir()));

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::SchemaValidation(_)));
    }
}
