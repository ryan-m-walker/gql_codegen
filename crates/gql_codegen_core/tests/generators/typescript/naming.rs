//! Tests for naming convention generation options

use gql_codegen_core::{NamingCase, NamingConvention, NamingConventionConfig, GeneratorOptions};

use super::generate_with_options;

#[test]
fn test_naming_keep() {
    // "keep" preserves original names
    let output = generate_with_options(
        &["schemas/naming.graphql"],
        GeneratorOptions {
            naming_convention: Some(NamingConvention::Simple(NamingCase::Keep)),
            ..GeneratorOptions::default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_naming_pascal_case() {
    // PascalCase transforms all type names
    let output = generate_with_options(
        &["schemas/naming.graphql"],
        GeneratorOptions {
            naming_convention: Some(NamingConvention::Simple(NamingCase::PascalCase)),
            ..GeneratorOptions::default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_naming_camel_case() {
    // camelCase transforms all type names
    let output = generate_with_options(
        &["schemas/naming.graphql"],
        GeneratorOptions {
            naming_convention: Some(NamingConvention::Simple(NamingCase::CamelCase)),
            ..GeneratorOptions::default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_naming_constant_case() {
    // CONSTANT_CASE transforms all type names
    let output = generate_with_options(
        &["schemas/naming.graphql"],
        GeneratorOptions {
            naming_convention: Some(NamingConvention::Simple(NamingCase::ConstantCase)),
            ..GeneratorOptions::default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_naming_detailed_config() {
    // Separate conventions for type names and enum values
    let output = generate_with_options(
        &["schemas/naming.graphql"],
        GeneratorOptions {
            naming_convention: Some(NamingConvention::Detailed(NamingConventionConfig {
                type_names: Some(NamingCase::PascalCase),
                enum_values: Some(NamingCase::ConstantCase),
                transform_underscore: true,
            })),
            ..GeneratorOptions::default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_naming_transform_underscore() {
    // Transform underscore treats underscores as word boundaries
    let output = generate_with_options(
        &["schemas/naming.graphql"],
        GeneratorOptions {
            naming_convention: Some(NamingConvention::Detailed(NamingConventionConfig {
                type_names: Some(NamingCase::PascalCase),
                enum_values: Some(NamingCase::CamelCase),
                transform_underscore: true,
            })),
            ..GeneratorOptions::default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_enum_prefix() {
    let output = generate_with_options(
        &["schemas/naming.graphql"],
        GeneratorOptions {
            enum_prefix: Some("E".to_string()),
            ..GeneratorOptions::default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_enum_suffix() {
    let output = generate_with_options(
        &["schemas/naming.graphql"],
        GeneratorOptions {
            enum_suffix: Some("Enum".to_string()),
            ..GeneratorOptions::default()
        },
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_enum_prefix_and_suffix() {
    let output = generate_with_options(
        &["schemas/naming.graphql"],
        GeneratorOptions {
            enum_prefix: Some("E".to_string()),
            enum_suffix: Some("Type".to_string()),
            ..GeneratorOptions::default()
        },
    );
    insta::assert_snapshot!(output);
}
