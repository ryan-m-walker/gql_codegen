//! Tests for union type generation

use gql_codegen_core::GeneratorOptions;

use super::generate_with_options;

#[test]
fn test_unions_default() {
    let output = generate_with_options(&["schemas/union.graphql"], GeneratorOptions::default());
    insta::assert_snapshot!(output);
}

#[test]
fn test_unions_skip_typename() {
    let output = generate_with_options(
        &["schemas/union.graphql"],
        GeneratorOptions {
            skip_typename: true,
            ..GeneratorOptions::default()
        },
    );
    insta::assert_snapshot!(output);
}
