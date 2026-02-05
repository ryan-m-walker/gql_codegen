//! Diagnostic rendering for errors and warnings
//!
//! All errors and warnings render through a single `Diagnostic` struct for
//! consistent Rust-compiler-style output with optional ANSI color.

use std::io::{self, IsTerminal};
use std::path::Path;

pub use apollo_compiler::diagnostic::Color;
use apollo_compiler::diagnostic::ToCliReport;

use crate::documents::DocumentWarning;
use crate::error::{ConfigError, Error};

// ── ANSI style helpers ──────────────────────────────────────────────────────

/// Resolved ANSI escape sequences (empty strings when color is disabled)
pub(crate) struct Styles {
    bold: &'static str,
    red: &'static str,
    yellow: &'static str,
    cyan: &'static str,
    dim: &'static str,
    reset: &'static str,
}

const COLORED: Styles = Styles {
    bold: "\x1b[1m",
    red: "\x1b[31m",
    yellow: "\x1b[33m",
    cyan: "\x1b[36m",
    dim: "\x1b[2m",
    reset: "\x1b[0m",
};

const PLAIN: Styles = Styles {
    bold: "",
    red: "",
    yellow: "",
    cyan: "",
    dim: "",
    reset: "",
};

/// Resolve apollo-compiler's `Color` enum to concrete style codes
fn styles_for(color: Color) -> &'static Styles {
    match color {
        Color::Never => &PLAIN,
        // StderrIsTerminal or any future variant — check at call time
        _ => {
            if io::stderr().is_terminal() {
                &COLORED
            } else {
                &PLAIN
            }
        }
    }
}

// ── Core types ──────────────────────────────────────────────────────────────

/// Severity level for a diagnostic message
#[derive(Debug, Clone, Copy)]
pub enum Severity {
    Error,
    Warning,
}

/// Source location snippet for rich error display
pub struct Snippet<'a> {
    pub file: &'a Path,
    pub source: &'a str,
    pub line: usize,   // 1-based
    pub column: usize, // 1-based
    /// Number of characters to underline. `None` renders a single `^`,
    /// `Some(n)` renders `n` carets (`^^^^^`). Use this when span length
    /// is available (e.g. from apollo-compiler byte offsets).
    pub length: Option<usize>,
}

/// A renderable diagnostic with optional source snippet
pub struct Diagnostic<'a> {
    pub severity: Severity,
    pub message: &'a str,
    pub snippet: Option<Snippet<'a>>,
}

// ── Private helpers ─────────────────────────────────────────────────────────

/// Count the number of decimal digits in a positive integer
fn digit_count(n: usize) -> usize {
    if n == 0 {
        return 1;
    }
    let mut count = 0;
    let mut val = n;
    while val > 0 {
        count += 1;
        val /= 10;
    }
    count
}

/// Render a single source line with a dim gutter
///
/// Format: `  {line_num} │ {text}`
/// where line_num is right-aligned to `gutter_width`.
fn render_source_line(
    w: &mut dyn io::Write,
    line_num: usize,
    text: &str,
    gutter_width: usize,
    s: &Styles,
) -> io::Result<()> {
    writeln!(
        w,
        "  {dim}{num:>width$} │{reset} {text}",
        dim = s.dim,
        num = line_num,
        width = gutter_width,
        reset = s.reset,
        text = text,
    )
}

// ── Diagnostic rendering ────────────────────────────────────────────────────

impl Diagnostic<'_> {
    /// Render this diagnostic to a writer in Rust-compiler style.
    ///
    /// Uses the provided `Styles` for optional ANSI coloring:
    /// - Severity label: **bold** + red/yellow
    /// - File location: **cyan**
    /// - Gutter (line numbers + `│`): **dim**
    /// - Caret (`^`): **bold** + red/yellow (matches severity)
    ///
    /// **Without snippet:**
    /// ```text
    /// [Error]: message
    /// ```
    ///
    /// **With snippet:**
    /// ```text
    /// [Error]: expected a string
    ///
    ///  config.json:2:5
    ///   1 │ {
    ///   2 │     "key": value
    ///     │     ^
    ///   3 │ }
    /// ```
    pub(crate) fn render(&self, w: &mut dyn io::Write, s: &Styles) -> io::Result<()> {
        match self.severity {
            Severity::Error => {
                writeln!(w, "{}{}[Error]: {}{}", s.red, s.bold, self.message, s.reset)?;
            }
            Severity::Warning => {
                writeln!(
                    w,
                    "{}{}[Warning]: {}{}",
                    s.yellow, s.bold, self.message, s.reset
                )?;
            }
        }

        let Some(snippet) = &self.snippet else {
            return Ok(());
        };

        writeln!(w)?;

        // File location
        writeln!(
            w,
            " {cyan}{file}:{line}:{col}{reset}",
            cyan = s.cyan,
            file = snippet.file.display(),
            line = snippet.line,
            col = snippet.column,
            reset = s.reset,
        )?;

        // Source context lines
        let lines: Vec<&str> = snippet.source.lines().collect();
        let line_idx = snippet.line.saturating_sub(1);
        let max_line_shown = (snippet.line + 1).min(lines.len());
        let gutter = digit_count(max_line_shown);

        let severity_color = match self.severity {
            Severity::Error => s.red,
            Severity::Warning => s.yellow,
        };

        // Previous line
        if snippet.line >= 2 {
            if let Some(prev) = lines.get(line_idx.wrapping_sub(1)) {
                render_source_line(w, snippet.line - 1, prev, gutter, s)?;
            }
        }

        // Error line
        if let Some(current) = lines.get(line_idx) {
            render_source_line(w, snippet.line, current, gutter, s)?;
        }

        // Caret line — blank gutter, offset to column, then carets
        write!(
            w,
            "  {dim}{0:>gutter$} │{reset} {0:>col$}{color}{bold}",
            "",
            dim = s.dim,
            gutter = gutter,
            reset = s.reset,
            col = snippet.column.saturating_sub(1),
            color = severity_color,
            bold = s.bold,
        )?;
        for _ in 0..snippet.length.unwrap_or(1) {
            write!(w, "^")?;
        }
        writeln!(w, "{reset}", reset = s.reset)?;

        // Next line
        if let Some(next) = lines.get(line_idx + 1) {
            render_source_line(w, snippet.line + 1, next, gutter, s)?;
        }

        writeln!(w)?;

        Ok(())
    }
}

// ── Public API ──────────────────────────────────────────────────────────────

/// Render an error to a writer with optional color support.
///
/// All error variants render through our `Diagnostic` struct for consistent output.
/// Apollo schema errors are extracted from `DiagnosticList` into `Snippet`s
/// using the public `SourceSpan` / `SourceFile` APIs.
pub fn render_error(err: &Error, color: Color, w: &mut dyn io::Write) -> io::Result<()> {
    let s = styles_for(color);
    match err {
        Error::SchemaParse(diagnostics) | Error::SchemaValidation(diagnostics) => {
            for diag in diagnostics.iter() {
                let msg = diag.error.to_string();

                let snippet = diag.error.location().and_then(|span| {
                    let file = diag.sources.get(&span.file_id())?;
                    let start = file.get_line_column(span.offset())?;
                    let len = span.end_offset().saturating_sub(span.offset());

                    Some(Snippet {
                        file: file.path(),
                        source: file.source_text(),
                        line: start.line,
                        column: start.column,
                        length: if len > 1 { Some(len) } else { None },
                    })
                });

                Diagnostic {
                    severity: Severity::Error,
                    message: &msg,
                    snippet,
                }
                .render(w, s)?;
            }
            Ok(())
        }
        Error::Config(config_err) => render_config_diagnostic(config_err, s, w),
        other => {
            let msg = other.to_string();
            Diagnostic {
                severity: Severity::Error,
                message: &msg,
                snippet: None,
            }
            .render(w, s)
        }
    }
}

/// Render a document warning to a writer.
pub fn render_warning(
    warn: &DocumentWarning,
    color: Color,
    w: &mut dyn io::Write,
) -> io::Result<()> {
    let s = styles_for(color);
    match warn {
        DocumentWarning::ParseErrors(diagnostics) => {
            for diag in diagnostics.iter() {
                let msg = diag.error.to_string();

                let snippet = diag.error.location().and_then(|span| {
                    let file = diag.sources.get(&span.file_id())?;
                    let start = file.get_line_column(span.offset())?;
                    let len = span.end_offset().saturating_sub(span.offset());

                    Some(Snippet {
                        file: file.path(),
                        source: file.source_text(),
                        line: start.line,
                        column: start.column,
                        length: if len > 1 { Some(len) } else { None },
                    })
                });

                Diagnostic {
                    severity: Severity::Warning,
                    message: &msg,
                    snippet,
                }
                .render(w, s)?;
            }
            Ok(())
        }
        other => {
            let msg = other.to_string();
            Diagnostic {
                severity: Severity::Warning,
                message: &msg,
                snippet: None,
            }
            .render(w, s)
        }
    }
}

/// Build and render a Diagnostic for a config error (has source location).
fn render_config_diagnostic(
    err: &ConfigError,
    s: &Styles,
    w: &mut dyn io::Write,
) -> io::Result<()> {
    Diagnostic {
        severity: Severity::Error,
        message: &err.message,
        snippet: Some(Snippet {
            file: &err.file,
            source: &err.source_text,
            line: err.line,
            column: err.column,
            length: None, // serde_json only provides point location
        }),
    }
    .render(w, s)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_render_config_error() {
        let src = r#"{
    "key": value
}"#;

        let err = ConfigError {
            file: PathBuf::from("config.json"),
            line: 2,
            column: 5,
            message: "expected a string".to_string(),
            source_text: src.to_string(),
        };

        let mut buf = Vec::new();
        render_config_diagnostic(&err, &PLAIN, &mut buf).unwrap();
        insta::assert_snapshot!(String::from_utf8(buf).unwrap());
    }

    #[test]
    fn test_digit_count() {
        assert_eq!(digit_count(0), 1);
        assert_eq!(digit_count(1), 1);
        assert_eq!(digit_count(9), 1);
        assert_eq!(digit_count(10), 2);
        assert_eq!(digit_count(99), 2);
        assert_eq!(digit_count(100), 3);
    }

    #[test]
    fn test_diagnostic_without_snippet() {
        let diag = Diagnostic {
            severity: Severity::Warning,
            message: "something went wrong",
            snippet: None,
        };
        let mut buf = Vec::new();
        diag.render(&mut buf, &PLAIN).unwrap();
        assert_eq!(
            String::from_utf8(buf).unwrap(),
            "[Warning]: something went wrong\n"
        );
    }

    #[test]
    fn test_diagnostic_with_color() {
        let diag = Diagnostic {
            severity: Severity::Error,
            message: "bad thing",
            snippet: None,
        };
        let mut buf = Vec::new();
        diag.render(&mut buf, &COLORED).unwrap();
        let output = String::from_utf8(buf).unwrap();
        // Should contain ANSI escape codes
        assert!(output.contains("\x1b["));
        insta::assert_snapshot!(output);
    }
}
