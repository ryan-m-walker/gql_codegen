//! Snapshot tests for diagnostic rendering output.
//!
//! Uses `insta` to capture and verify the exact output format of error/warning
//! rendering. All tests use `Color::Never` for deterministic, ANSI-free output.

use std::path::PathBuf;

use gql_codegen_core::diagnostic::{Color, render_diagnostic, render_diagnostics};
use gql_codegen_core::{Diagnostic, DiagnosticCategory, DiagnosticLocation, Diagnostics};

/// Helper: render diagnostics to a String (no color).
fn render_ds(ds: &Diagnostics) -> String {
    let mut buf = Vec::new();
    render_diagnostics(ds, None, Color::Never, 0, &mut buf).expect("render should not fail");
    String::from_utf8(buf).expect("output should be valid UTF-8")
}

/// Helper: render a single diagnostic to a String (no color).
fn render_d(d: &Diagnostic) -> String {
    let mut buf = Vec::new();
    render_diagnostic(d, None, Color::Never, &mut buf).expect("render should not fail");
    String::from_utf8(buf).expect("output should be valid UTF-8")
}

#[test]
fn schema_parse_error() {
    let broken_sdl = "type Query { field: }".to_string();
    let err = gql_codegen_core::load_schema_from_contents(&[(
        PathBuf::from("schema.graphql"),
        broken_sdl,
    )])
    .unwrap_err();

    assert!(err.has_errors());
    assert!(err.errors().any(|d| d.category == DiagnosticCategory::Schema));
    insta::assert_snapshot!(render_ds(&err));
}

#[test]
fn schema_validation_error() {
    // References an unknown type — parses fine but fails validation
    let sdl = "type Query { user: UnknownType }".to_string();
    let err =
        gql_codegen_core::load_schema_from_contents(&[(PathBuf::from("schema.graphql"), sdl)])
            .unwrap_err();

    assert!(err.has_errors());
    assert!(err.errors().any(|d| d.category == DiagnosticCategory::Schema));
    insta::assert_snapshot!(render_ds(&err));
}

#[test]
fn config_parse_error() {
    let d = Diagnostic::error(DiagnosticCategory::Config, "expected a string")
        .with_location(DiagnosticLocation {
            file: PathBuf::from("config.json"),
            line: 1,
            column: 10,
            length: None,
        })
        .with_inline_source("{ \"key\": value }".to_string());

    insta::assert_snapshot!(render_d(&d));
}

#[test]
fn document_warning_duplicate_name() {
    let d = Diagnostic::warning(
        DiagnosticCategory::Document,
        "Duplicate operation 'GetUser' (skipped)",
    );
    insta::assert_snapshot!(render_d(&d));
}
