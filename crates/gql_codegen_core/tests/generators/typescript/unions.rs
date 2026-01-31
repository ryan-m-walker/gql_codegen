//! Tests for union type generation

use gql_codegen_core::PluginOptions;

use super::generate_with_options;

#[test]
fn test_unions_default() {
    let output = generate_with_options(&["schemas/union.graphql"], PluginOptions::default());
    insta::assert_snapshot!(output);
}

#[test]
fn test_unions_skip_typename() {
    let output = generate_with_options(
        &["schemas/union.graphql"],
        PluginOptions {
            skip_typename: true,
            ..Default::default()
        },
    );
    insta::assert_snapshot!(output);
}
