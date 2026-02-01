//! Tests for custom scalar generation with scalar mappings

use std::collections::BTreeMap;

use gql_codegen_core::PluginOptions;

use super::{generate_with_options, try_generate_with_options};

#[test]
fn test_scalars_default() {
    // Without mappings, custom scalars become `unknown`
    let output = generate_with_options(&["schemas/scalar.graphql"], PluginOptions::default());
    insta::assert_snapshot!(output);
}

#[test]
fn test_scalars_with_mappings() {
    let mut scalars = BTreeMap::new();
    scalars.insert("DateTime".to_string(), "string".to_string());
    scalars.insert("Money".to_string(), "number".to_string());
    scalars.insert("JSON".to_string(), "Record<string, unknown>".to_string());

    let output = generate_with_options(
        &["schemas/scalar.graphql"],
        PluginOptions {
            scalars,
            ..Default::default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_scalars_default_scalar_type() {
    // Use "any" instead of "unknown" for unmapped scalars
    let output = generate_with_options(
        &["schemas/scalar.graphql"],
        PluginOptions {
            default_scalar_type: Some("any".to_string()),
            ..Default::default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_scalars_strict_with_all_mapped() {
    // strict_scalars should pass when all scalars are mapped
    let mut scalars = BTreeMap::new();
    scalars.insert("DateTime".to_string(), "string".to_string());
    scalars.insert("Money".to_string(), "number".to_string());
    scalars.insert("JSON".to_string(), "Record<string, unknown>".to_string());

    let output = generate_with_options(
        &["schemas/scalar.graphql"],
        PluginOptions {
            scalars,
            strict_scalars: true,
            ..Default::default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_scalars_strict_missing_scalar_errors() {
    // strict_scalars should error when a scalar is not mapped
    let result = try_generate_with_options(
        &["schemas/scalar.graphql"],
        PluginOptions {
            strict_scalars: true,
            ..Default::default()
        },
    );

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("strictScalars"), "Error should mention strictScalars: {}", err);
}
