//! Tests for object type generation with different configuration options

use gql_codegen_core::{AvoidOptionals, GeneratorOptions};

use super::generate_with_options;

#[test]
fn test_objects_default() {
    let output = generate_with_options(&["schemas/object.graphql"], GeneratorOptions::default());
    insta::assert_snapshot!(output);
}

#[test]
fn test_objects_immutable() {
    let output = generate_with_options(
        &["schemas/object.graphql"],
        GeneratorOptions {
            immutable_types: Some(true),
            ..GeneratorOptions::default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_objects_skip_typename() {
    let output = generate_with_options(
        &["schemas/object.graphql"],
        GeneratorOptions {
            skip_typename: true,
            ..GeneratorOptions::default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_objects_avoid_optionals() {
    let output = generate_with_options(
        &["schemas/object.graphql"],
        GeneratorOptions {
            avoid_optionals: AvoidOptionals::Boolean(true),
            ..GeneratorOptions::default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_objects_all_options() {
    let output = generate_with_options(
        &["schemas/object.graphql"],
        GeneratorOptions {
            immutable_types: Some(true),
            skip_typename: true,
            avoid_optionals: AvoidOptionals::Boolean(true),
            ..GeneratorOptions::default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_no_export() {
    let output = generate_with_options(
        &["schemas/object.graphql"],
        GeneratorOptions {
            no_export: true,
            ..GeneratorOptions::default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_only_operation_types_no_ops() {
    // With no operations, only_operation_types should generate nothing (except Maybe)
    let output = generate_with_options(
        &["schemas/object.graphql"],
        GeneratorOptions {
            only_operation_types: true,
            ..GeneratorOptions::default()
        },
    );
    insta::assert_snapshot!(output);
}
