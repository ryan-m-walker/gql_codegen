//! WASM bindings for gql_codegen_core
//!
//! Provides browser-compatible GraphQL code generation.

use std::collections::HashMap;
use std::path::PathBuf;

use gql_codegen_core::{
    CollectedDocuments, ExtractConfig, GenerateInput, OutputConfig, PluginConfig, PluginOptions,
    SourceCache, collect_documents, generate_from_input, load_schema_from_contents,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Initialize panic hook for better error messages in browser console
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Result of code generation
#[derive(Serialize, Deserialize)]
pub struct GenerateResult {
    pub output: String,
    pub error: Option<String>,
    pub warnings: Vec<String>,
}

/// String or array of strings (matches core StringOrArray)
#[derive(Deserialize)]
#[serde(untagged)]
pub enum StringOrArray {
    Single(String),
    Multiple(Vec<String>),
}

impl StringOrArray {
    fn into_vec(self) -> Vec<String> {
        match self {
            Self::Single(s) => vec![s],
            Self::Multiple(v) => v,
        }
    }
}

/// Config input from JavaScript - matches SGC config format
#[derive(Deserialize)]
pub struct WasmConfig {
    #[serde(default)]
    pub schema: Option<StringOrArray>,
    #[serde(default)]
    pub documents: Option<StringOrArray>,
    pub generates: HashMap<String, WasmOutputConfig>,
}

#[derive(Deserialize)]
pub struct WasmOutputConfig {
    pub plugins: Vec<String>,
    #[serde(default)]
    pub config: Option<PluginOptions>,
}

/// Generate TypeScript types from GraphQL schema and operations
///
/// # Arguments
/// * `schema` - GraphQL schema SDL string or array of strings
/// * `operations` - GraphQL operations string or array of strings
/// * `config` - JSON config object matching SGC config format
///
/// # Returns
/// JSON object with `output` (generated code) or `error` (error message)
#[wasm_bindgen]
pub fn generate(schema: JsValue, operations: JsValue, config: JsValue) -> JsValue {
    // Parse inputs
    let schemas: Vec<String> = serde_wasm_bindgen::from_value::<StringOrArray>(schema)
        .map(|s| s.into_vec())
        .unwrap_or_default();
    let ops: Vec<String> = serde_wasm_bindgen::from_value::<StringOrArray>(operations)
        .map(|s| s.into_vec())
        .unwrap_or_default();
    let wasm_config: Option<WasmConfig> = serde_wasm_bindgen::from_value(config).ok();

    let result = generate_internal(&schemas, &ops, wasm_config);
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

fn generate_internal(
    schemas: &[String],
    operations: &[String],
    wasm_config: Option<WasmConfig>,
) -> GenerateResult {
    // Parse schema(s)
    let schema_files: Vec<(PathBuf, String)> = schemas
        .iter()
        .enumerate()
        .map(|(i, s)| (PathBuf::from(format!("schema_{i}.graphql")), s.clone()))
        .collect();

    if schema_files.is_empty() {
        return GenerateResult {
            output: String::new(),
            error: Some("No schema provided".to_string()),
            warnings: vec![],
        };
    }

    let schema = match load_schema_from_contents(&schema_files) {
        Ok(s) => s,
        Err(e) => {
            return GenerateResult {
                output: String::new(),
                error: Some(format!("{e}")),
                warnings: vec![],
            };
        }
    };

    // Create source cache with operations
    let mut source_cache = SourceCache::new();
    for (i, op) in operations.iter().enumerate() {
        if !op.trim().is_empty() {
            source_cache.push(PathBuf::from(format!("operations_{i}.graphql")), op.clone());
        }
    }

    // Collect documents (operations and fragments)
    let extract_config = ExtractConfig::default();
    let documents: CollectedDocuments = collect_documents(&source_cache, &extract_config);

    // Collect warnings from document parsing
    let warnings = documents.warnings.clone();

    // Build generates config from wasm_config or use defaults
    let generates: HashMap<String, OutputConfig> = match wasm_config {
        Some(cfg) => {
            cfg.generates
                .into_iter()
                .map(|(path, out)| {
                    let output_config = OutputConfig {
                        plugins: out
                            .plugins
                            .into_iter()
                            .map(PluginConfig::Name)
                            .collect(),
                        prelude: None,
                        config: out.config,
                        documents_only: false,
                    };
                    (path, output_config)
                })
                .collect()
        }
        None => {
            // Default: typescript + typescript-operations
            let mut map = HashMap::new();
            map.insert(
                "types.ts".to_string(),
                OutputConfig {
                    plugins: vec![
                        PluginConfig::Name("typescript".to_string()),
                        PluginConfig::Name("typescript-operations".to_string()),
                    ],
                    prelude: None,
                    config: None,
                    documents_only: false,
                },
            );
            map
        }
    };

    // Generate code
    let input = GenerateInput {
        schema: &schema,
        documents: &documents,
        generates: &generates,
    };

    match generate_from_input(&input) {
        Ok(result) => {
            // Combine document warnings with generation warnings
            let mut all_warnings = warnings;
            all_warnings.extend(result.warnings.clone());

            // Return the first (and only) generated file's content
            let output = result
                .files
                .into_iter()
                .next()
                .map(|f| f.content)
                .unwrap_or_default();

            GenerateResult {
                output,
                error: None,
                warnings: all_warnings,
            }
        }
        Err(e) => GenerateResult {
            output: String::new(),
            error: Some(format!("{e}")),
            warnings,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_basic() {
        let schema = r#"
            type Query {
                user(id: ID!): User
            }
            type User {
                id: ID!
                name: String!
            }
        "#;

        let operations = r#"
            query GetUser($id: ID!) {
                user(id: $id) {
                    id
                    name
                }
            }
        "#;

        let result = generate_internal(&[schema.to_string()], &[operations.to_string()], None);
        assert!(result.error.is_none(), "Expected no error: {:?}", result.error);
        assert!(result.output.contains("GetUserQuery"));
        assert!(result.output.contains("GetUserQueryVariables"));
    }

    #[test]
    fn test_generate_schema_only() {
        let schema = r#"
            type Query {
                users: [User!]!
            }
            type User {
                id: ID!
                name: String!
            }
        "#;

        let result = generate_internal(&[schema.to_string()], &[], None);
        assert!(result.error.is_none());
        assert!(result.output.contains("type User"));
    }

    #[test]
    fn test_invalid_schema() {
        let schema = "type Query { invalid";
        let result = generate_internal(&[schema.to_string()], &[], None);
        assert!(result.error.is_some());
    }
}
