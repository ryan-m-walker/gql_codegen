//! Schema loading and validation

use std::path::{Path, PathBuf};

use apollo_compiler::Schema;
use apollo_compiler::validation::Valid;

use crate::diagnostic::{Diagnostic, DiagnosticCategory, Diagnostics, Severity};
use crate::error::Result;

/// Load and validate a GraphQL schema from one or more file paths.
pub fn load_schema(paths: &[PathBuf]) -> Result<Valid<Schema>> {
    let mut builder = Schema::builder();

    for path in paths {
        let content = std::fs::read_to_string(path).map_err(|e| {
            Diagnostics::from(
                Diagnostic::error(
                    DiagnosticCategory::Schema,
                    format!("Failed to read schema '{}': {}", path.display(), e),
                ),
            )
        })?;

        builder = builder.parse(content, path);
    }

    let schema = builder
        .build()
        .map_err(|e| Diagnostics::from_apollo(&e.errors, Severity::Error, DiagnosticCategory::Schema))?;

    schema
        .validate()
        .map_err(|e| Diagnostics::from_apollo(&e.errors, Severity::Error, DiagnosticCategory::Schema))
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
        .map_err(|e| Diagnostics::from_apollo(&e.errors, Severity::Error, DiagnosticCategory::Schema))?;

    schema
        .validate()
        .map_err(|e| Diagnostics::from_apollo(&e.errors, Severity::Error, DiagnosticCategory::Schema))
}

/// Helper to resolve schema paths from config (convenience for simple cases)
pub fn resolve_schema_paths(patterns: &[&str], base_dir: Option<&Path>) -> Vec<PathBuf> {
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
