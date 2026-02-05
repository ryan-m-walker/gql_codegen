//! Snapshot tests for diagnostic rendering output.
//!
//! Uses `insta` to capture and verify the exact output format of error/warning
//! rendering. All tests use `Color::Never` for deterministic, ANSI-free output.

use std::path::PathBuf;

use gql_codegen_core::diagnostic::{Color, render_error, render_warning};
use gql_codegen_core::{ConfigError, DocumentWarning, Error};

/// Strip ANSI escape codes from rendered output.
///
/// Stripping here keeps snapshots deterministic regardless of the renderer.
fn strip_ansi(bytes: Vec<u8>) -> String {
    let stripped = strip_ansi_escapes::strip(&bytes);
    String::from_utf8(stripped).expect("output should be valid UTF-8")
}

/// Helper: render an error to a String (no color).
fn render_err(err: &Error) -> String {
    let mut buf = Vec::new();
    render_error(err, Color::Never, 0, &mut buf).expect("render should not fail");
    strip_ansi(buf)
}

/// Helper: render a warning to a String (no color).
fn render_warn(warn: &DocumentWarning) -> String {
    let mut buf = Vec::new();
    render_warning(warn, Color::Never, 0, &mut buf).expect("render should not fail");
    strip_ansi(buf)
}

#[test]
fn schema_parse_error() {
    let broken_sdl = "type Query { field: }".to_string();
    let err = gql_codegen_core::load_schema_from_contents(&[(
        PathBuf::from("schema.graphql"),
        broken_sdl,
    )])
    .unwrap_err();

    assert!(matches!(err, Error::SchemaParse(_)));
    insta::assert_snapshot!(render_err(&err));
}

#[test]
fn schema_validation_error() {
    // References an unknown type — parses fine but fails validation
    let sdl = "type Query { user: UnknownType }".to_string();
    let err =
        gql_codegen_core::load_schema_from_contents(&[(PathBuf::from("schema.graphql"), sdl)])
            .unwrap_err();

    assert!(matches!(err, Error::SchemaValidation(_)));
    insta::assert_snapshot!(render_err(&err));
}

#[test]
fn config_parse_error() {
    let err = ConfigError {
        file: PathBuf::from("config.json"),
        line: 1,
        column: 10,
        message: "expected a string".to_string(),
        source_text: "{ \"key\": value }".to_string(),
    };

    insta::assert_snapshot!(render_err(&Error::Config(err)));
}

#[test]
fn document_warning_duplicate_name() {
    let warn = DocumentWarning::DuplicateName {
        kind: "operation",
        name: "GetUser".to_string(),
    };
    insta::assert_snapshot!(render_warn(&warn));
}

