//! Tests for custom scalar generation with scalar mappings

use std::collections::BTreeMap;

use gql_codegen_core::PluginOptions;

use super::generate_with_options;

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
