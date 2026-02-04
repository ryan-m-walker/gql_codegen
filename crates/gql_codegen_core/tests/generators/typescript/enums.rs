//! Tests for enum generation with different configuration options

use gql_codegen_core::PluginOptions;

use super::generate_with_options;

#[test]
fn test_enums_default() {
    let output = generate_with_options(&["schemas/enum.graphql"], PluginOptions::default());
    insta::assert_snapshot!(output);
}

#[test]
fn test_enums_as_types() {
    let output = generate_with_options(
        &["schemas/enum.graphql"],
        PluginOptions {
            enums_as_types: true,
            ..PluginOptions::serde_default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_enums_future_proof() {
    let output = generate_with_options(
        &["schemas/enum.graphql"],
        PluginOptions {
            enums_as_types: true,
            future_proof_enums: true,
            ..PluginOptions::serde_default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_const_enums() {
    let output = generate_with_options(
        &["schemas/enum.graphql"],
        PluginOptions {
            enums_as_types: false,
            const_enums: true,
            ..PluginOptions::serde_default()
        },
    );
    insta::assert_snapshot!(output);
}
