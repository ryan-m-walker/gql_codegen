//! Tests for enum generation with different configuration options

use gql_codegen_core::PluginOptions;

use super::generate_with_options;

#[test]
fn test_maybe_value() {
    // maybe_value only applies when use_utility_types is true
    let output = generate_with_options(
        &[],
        PluginOptions {
            maybe_value: Some("T | null | undefined".to_string()),
            use_utility_types: true,
            ..PluginOptions::default()
        },
    );

    insta::assert_snapshot!(output);
}
