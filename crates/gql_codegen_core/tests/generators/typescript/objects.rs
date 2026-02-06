//! Tests for object type generation with different configuration options

use gql_codegen_core::{AvoidOptionals, PluginOptions};

use super::generate_with_options;

#[test]
fn test_objects_default() {
    let output = generate_with_options(&["schemas/object.graphql"], PluginOptions::default());
    insta::assert_snapshot!(output);
}

#[test]
fn test_objects_immutable() {
    let output = generate_with_options(
        &["schemas/object.graphql"],
        PluginOptions {
            immutable_types: true,
            ..PluginOptions::serde_default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_objects_skip_typename() {
    let output = generate_with_options(
        &["schemas/object.graphql"],
        PluginOptions {
            skip_typename: true,
            ..PluginOptions::serde_default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_objects_avoid_optionals() {
    let output = generate_with_options(
        &["schemas/object.graphql"],
        PluginOptions {
            avoid_optionals: AvoidOptionals::Boolean(true),
            ..PluginOptions::serde_default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_objects_all_options() {
    let output = generate_with_options(
        &["schemas/object.graphql"],
        PluginOptions {
            immutable_types: true,
            skip_typename: true,
            avoid_optionals: AvoidOptionals::Boolean(true),
            ..PluginOptions::serde_default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_no_export() {
    let output = generate_with_options(
        &["schemas/object.graphql"],
        PluginOptions {
            no_export: true,
            ..PluginOptions::serde_default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_only_operation_types_no_ops() {
    // With no operations, only_operation_types should generate nothing (except Maybe)
    let output = generate_with_options(
        &["schemas/object.graphql"],
        PluginOptions {
            only_operation_types: true,
            ..PluginOptions::serde_default()
        },
    );
    insta::assert_snapshot!(output);
}
