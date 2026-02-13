//! Tests for input object generation with different configuration options

use gql_codegen_core::{AvoidOptionals, PluginOptions};

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
            ..PluginOptions::default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_inputs_avoid_optionals() {
    let output = generate_with_options(
        &["schemas/input.graphql"],
        PluginOptions {
            avoid_optionals: AvoidOptionals::Boolean(true),
            ..PluginOptions::default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_inputs_maybe_value() {
    // maybe_value affects both input and output types when input_maybe_value is not set
    let output = generate_with_options(
        &["schemas/input.graphql"],
        PluginOptions {
            maybe_value: Some("T | null | undefined".to_string()),
            ..PluginOptions::default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_inputs_input_maybe_value() {
    // input_maybe_value provides separate nullability handling for input fields
    let output = generate_with_options(
        &["schemas/input.graphql"],
        PluginOptions {
            input_maybe_value: Some("InputMaybe<T>".to_string()),
            ..PluginOptions::default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_inputs_separate_maybe_types() {
    // When both are set, input_maybe_value is used for inputs, maybe_value for outputs
    let output = generate_with_options(
        &["schemas/input.graphql"],
        PluginOptions {
            maybe_value: Some("Maybe<T>".to_string()),
            input_maybe_value: Some("InputMaybe<T>".to_string()),
            ..PluginOptions::default()
        },
    );
    insta::assert_snapshot!(output);
}
