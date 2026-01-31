//! Tests for input object generation with different configuration options

use gql_codegen_core::PluginOptions;

use super::generate_with_options;

#[test]
fn test_inputs_default() {
    let output = generate_with_options(&["schemas/input.graphql"], PluginOptions::default());
    insta::assert_snapshot!(output);
}

#[test]
fn test_inputs_immutable() {
    let output = generate_with_options(
        &["schemas/input.graphql"],
        PluginOptions {
            immutable_types: true,
            ..Default::default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_inputs_avoid_optionals() {
    let output = generate_with_options(
        &["schemas/input.graphql"],
        PluginOptions {
            avoid_optionals: true,
            ..Default::default()
        },
    );
    insta::assert_snapshot!(output);
}
