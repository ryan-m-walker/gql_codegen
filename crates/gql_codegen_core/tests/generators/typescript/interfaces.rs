//! Tests for interface generation with different configuration options

use gql_codegen_core::GeneratorOptions;

use super::generate_with_options;

#[test]
fn test_interfaces_default() {
    let output = generate_with_options(&["schemas/interface.graphql"], GeneratorOptions::default());
    insta::assert_snapshot!(output);
}

#[test]
fn test_interfaces_immutable() {
    let output = generate_with_options(
        &["schemas/interface.graphql"],
        GeneratorOptions {
            immutable_types: Some(true),
            ..GeneratorOptions::default()
        },
    );
    insta::assert_snapshot!(output);
}
