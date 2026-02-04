//! Tests for enum generation with different configuration options

use gql_codegen_core::PluginOptions;

use super::generate_with_options;

#[test]
fn test_maybe_value() {
    let output = generate_with_options(
        &["schemas/base.graphql"],
        PluginOptions {
            maybe_value: Some("T | null | undefined".to_string()),
            ..Default::default()
        },
    );

    insta::assert_snapshot!(output);
}
