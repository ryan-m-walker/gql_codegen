//! Unified diagnostic system
//!
//! Every error, warning, and informational message uses a single [`Diagnostic`]
//! type. Fatal errors are returned via `Result<T, Diagnostics>`, non-fatal
//! issues are collected into a [`Diagnostics`] bag.

use std::fmt;
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

pub use apollo_compiler::diagnostic::Color;
use apollo_compiler::diagnostic::ToCliReport;
use apollo_compiler::validation::DiagnosticList;
use serde::{Deserialize, Serialize};

use crate::source_cache::SourceCache;

// ── Core types ──────────────────────────────────────────────────────────────

/// Severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
    Internal,
}

/// Diagnostic category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticCategory {
    Config,
    Schema,
    Document,
    Generation,
}

/// Source location for rich error display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticLocation {
    pub file: PathBuf,
    pub line: usize,   // 1-based
    pub column: usize, // 1-based
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<usize>,
}

/// A related source location with a label (e.g., "first defined here")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedLocation {
    pub message: String,
    pub location: DiagnosticLocation,
    #[serde(skip)]
    pub inline_source: Option<String>,
}

/// A single diagnostic message — the unified type for all errors and warnings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub category: DiagnosticCategory,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<DiagnosticLocation>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<RelatedLocation>,
    /// Source text for diagnostics whose source isn't in SourceCache
    /// (e.g., Apollo parse errors, config errors). When None, renderer
    /// looks up source from SourceCache via location.file.
    #[serde(skip)]
    pub inline_source: Option<String>,
}

/// Collection of diagnostics — the error type for `Result<T>` and the
/// accumulator for non-fatal warnings.
#[derive(Debug, Clone, Default)]
pub struct Diagnostics(pub Vec<Diagnostic>);

// ── Diagnostic constructors ─────────────────────────────────────────────────

impl Diagnostic {
    pub fn error(category: DiagnosticCategory, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            category,
            message: message.into(),
            location: None,
            related: Vec::new(),
            inline_source: None,
        }
    }

    pub fn warning(category: DiagnosticCategory, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            category,
            message: message.into(),
            location: None,
            related: Vec::new(),
            inline_source: None,
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Internal,
            category: DiagnosticCategory::Generation,
            message: message.into(),
            location: None,
            related: Vec::new(),
            inline_source: None,
        }
    }

    pub fn with_location(mut self, location: DiagnosticLocation) -> Self {
        self.location = Some(location);
        self
    }

    pub fn with_inline_source(mut self, source: String) -> Self {
        self.inline_source = Some(source);
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

// ── Diagnostics (collection) ────────────────────────────────────────────────

impl Diagnostics {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, d: Diagnostic) {
        self.0.push(d);
    }

    pub fn extend(&mut self, other: Diagnostics) {
        self.0.extend(other.0);
    }

    pub fn has_errors(&self) -> bool {
        self.0
            .iter()
            .any(|d| matches!(d.severity, Severity::Error | Severity::Internal))
    }

    pub fn warnings(&self) -> impl Iterator<Item = &Diagnostic> {
        self.0.iter().filter(|d| d.severity == Severity::Warning)
    }

    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.0
            .iter()
            .filter(|d| matches!(d.severity, Severity::Error | Severity::Internal))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Diagnostic> {
        self.0.iter()
    }

    /// Convert Apollo compiler diagnostics into our unified format.
    pub fn from_apollo(
        list: &DiagnosticList,
        severity: Severity,
        category: DiagnosticCategory,
    ) -> Self {
        let diagnostics = list
            .iter()
            .map(|diag| {
                let msg = diag.error.to_string();

                let (location, inline_source) = diag
                    .error
                    .location()
                    .and_then(|span| {
                        let file = diag.sources.get(&span.file_id())?;
                        let start = file.get_line_column(span.offset())?;
                        let len = span.end_offset().saturating_sub(span.offset());

                        Some((
                            DiagnosticLocation {
                                file: file.path().to_path_buf(),
                                line: start.line,
                                column: start.column,
                                length: if len > 1 { Some(len) } else { None },
                            },
                            file.source_text().to_string(),
                        ))
                    })
                    .map(|(loc, src)| (Some(loc), Some(src)))
                    .unwrap_or((None, None));

                Diagnostic {
                    severity,
                    category,
                    message: msg,
                    location,
                    related: Vec::new(),
                    inline_source,
                }
            })
            .collect();

        Self(diagnostics)
    }
}

impl fmt::Display for Diagnostics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.first() {
            Some(d) => write!(f, "{}", d.message),
            None => write!(f, "unknown error"),
        }
    }
}

impl std::error::Error for Diagnostics {}

// ── From implementations ────────────────────────────────────────────────────

impl From<Diagnostic> for Diagnostics {
    fn from(d: Diagnostic) -> Self {
        Self(vec![d])
    }
}

impl From<Vec<Diagnostic>> for Diagnostics {
    fn from(v: Vec<Diagnostic>) -> Self {
        Self(v)
    }
}

impl From<std::io::Error> for Diagnostics {
    fn from(e: std::io::Error) -> Self {
        Diagnostic::internal(format!("IO error: {e}")).into()
    }
}

impl IntoIterator for Diagnostics {
    type Item = Diagnostic;
    type IntoIter = std::vec::IntoIter<Diagnostic>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Diagnostics {
    type Item = &'a Diagnostic;
    type IntoIter = std::slice::Iter<'a, Diagnostic>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

// ── ANSI style helpers ──────────────────────────────────────────────────────

pub(crate) struct Styles {
    bold: &'static str,
    red: &'static str,
    yellow: &'static str,
    cyan: &'static str,
    dim: &'static str,
    underline: &'static str,
    reset: &'static str,
}

const COLORED: Styles = Styles {
    bold: "\x1b[1m",
    red: "\x1b[31m",
    yellow: "\x1b[33m",
    cyan: "\x1b[36m",
    dim: "\x1b[2m",
    underline: "\x1b[4m",
    reset: "\x1b[0m",
};

const PLAIN: Styles = Styles {
    bold: "",
    red: "",
    yellow: "",
    cyan: "",
    dim: "",
    underline: "",
    reset: "",
};

fn styles_for(color: Color) -> &'static Styles {
    match color {
        Color::Never => &PLAIN,
        _ => {
            if io::stderr().is_terminal() {
                &COLORED
            } else {
                &PLAIN
            }
        }
    }
}

// ── Rendering helpers ───────────────────────────────────────────────────────

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

fn render_source_line(
    w: &mut dyn Write,
    line_num: usize,
    text: &str,
    gutter_width: usize,
    s: &Styles,
    highlight: &str,
) -> io::Result<()> {
    writeln!(
        w,
        "  {bold}{num:>width$}{reset} {dim}│{reset} {hl}{text}{reset}",
        bold = s.bold,
        num = line_num,
        width = gutter_width,
        dim = s.dim,
        hl = highlight,
        reset = s.reset,
        text = text,
    )
}

/// Render a source snippet with line context and caret indicator.
fn render_snippet(
    w: &mut dyn Write,
    loc: &DiagnosticLocation,
    source: &str,
    severity_color: &str,
    s: &Styles,
) -> io::Result<()> {
    writeln!(w)?;

    writeln!(
        w,
        " {underline}{file}{reset}{dim}:{line}:{col}{reset}",
        underline = s.underline,
        file = loc.file.display(),
        dim = s.dim,
        line = loc.line,
        col = loc.column,
        reset = s.reset,
    )?;

    let lines: Vec<&str> = source.lines().collect();
    let line_idx = loc.line.saturating_sub(1);
    let max_line_shown = (loc.line + 1).min(lines.len());
    let gutter = digit_count(max_line_shown);

    let line_highlight = format!("{}{}", severity_color, s.bold);

    // Previous line
    if loc.line >= 2 {
        if let Some(prev) = lines.get(line_idx.wrapping_sub(1)) {
            render_source_line(w, loc.line - 1, prev, gutter, s, "")?;
        }
    }

    // Error line (highlighted)
    if let Some(current) = lines.get(line_idx) {
        render_source_line(w, loc.line, current, gutter, s, &line_highlight)?;
    }

    // Caret line
    write!(
        w,
        "  {bold}{0:>gutter$}{reset} {dim}│{reset} {0:>col$}{color}{bold}",
        "",
        bold = s.bold,
        gutter = gutter,
        dim = s.dim,
        reset = s.reset,
        col = loc.column.saturating_sub(1),
        color = severity_color,
    )?;
    for _ in 0..loc.length.unwrap_or(1) {
        write!(w, "^")?;
    }
    writeln!(w, "{reset}", reset = s.reset)?;

    // Next line
    if let Some(next) = lines.get(line_idx + 1) {
        render_source_line(w, loc.line + 1, next, gutter, s, "")?;
    }

    writeln!(w)?;

    Ok(())
}

// ── Public rendering API ────────────────────────────────────────────────────

/// Default cap for diagnostics displayed from a single Diagnostics bag.
pub const DEFAULT_MAX_DIAGNOSTICS: usize = 3;

/// Resolve source text for a diagnostic: inline_source first, then SourceCache.
fn resolve_source<'a>(d: &'a Diagnostic, cache: Option<&'a SourceCache>) -> Option<&'a str> {
    if let Some(ref src) = d.inline_source {
        return Some(src.as_str());
    }
    if let (Some(loc), Some(cache)) = (&d.location, cache) {
        return cache.get_by_path(&loc.file);
    }
    None
}

/// Render a single diagnostic to a writer.
pub fn render_diagnostic(
    d: &Diagnostic,
    cache: Option<&SourceCache>,
    color: Color,
    w: &mut dyn Write,
) -> io::Result<()> {
    let s = styles_for(color);
    let (severity_color, label) = match d.severity {
        Severity::Info => (s.cyan, "Info"),
        Severity::Warning => (s.yellow, "Warning"),
        Severity::Error => (s.red, "Error"),
        Severity::Internal => (s.red, "Internal Error"),
    };

    writeln!(
        w,
        "{color}{bold}[{label}]: {msg}{reset}",
        color = severity_color,
        bold = s.bold,
        label = label,
        msg = d.message,
        reset = s.reset,
    )?;

    if let Some(loc) = &d.location {
        let source = resolve_source(d, cache);
        if let Some(src) = source {
            render_snippet(w, loc, src, severity_color, s)?;
        } else {
            // File location without source snippet
            writeln!(
                w,
                " {underline}{file}{reset}{dim}:{line}:{col}{reset}\n",
                underline = s.underline,
                file = loc.file.display(),
                dim = s.dim,
                line = loc.line,
                col = loc.column,
                reset = s.reset,
            )?;
        }
    }

    // Related locations
    for related in &d.related {
        writeln!(
            w,
            " {dim}= note: {msg}{reset}",
            dim = s.dim,
            msg = related.message,
            reset = s.reset,
        )?;
        if let Some(ref src) = related.inline_source {
            render_snippet(w, &related.location, src, s.dim, s)?;
        }
    }

    Ok(())
}

/// Render a single diagnostic to a String with no color.
pub fn render_diagnostic_string(d: &Diagnostic) -> String {
    let mut buf = Vec::new();
    render_diagnostic(d, None, Color::Never, &mut buf).ok();
    String::from_utf8(buf).unwrap_or_else(|_| d.to_string())
}

/// Render all diagnostics, capped at `max` (0 = unlimited).
pub fn render_diagnostics(
    ds: &Diagnostics,
    cache: Option<&SourceCache>,
    color: Color,
    max: usize,
    w: &mut dyn Write,
) -> io::Result<()> {
    let s = styles_for(color);
    let total = ds.len();

    for (i, d) in ds.iter().enumerate() {
        if max > 0 && i >= max {
            let remaining = total - max;
            writeln!(
                w,
                "{red}... and {remaining} more{pl}{reset} {dim}(Hint: run with --max-diagnostics=0 to show all)\n",
                red = s.red,
                remaining = remaining,
                pl = if remaining == 1 { "" } else { "s" },
                reset = s.reset,
                dim = s.dim,
            )?;
            break;
        }

        render_diagnostic(d, cache, color, w)?;
    }

    Ok(())
}

/// Render all diagnostics to a String with no color (for NAPI/WASM boundaries).
pub fn render_diagnostics_string(ds: &Diagnostics, max: usize) -> String {
    let mut buf = Vec::new();
    render_diagnostics(ds, None, Color::Never, max, &mut buf).ok();
    String::from_utf8(buf).unwrap_or_else(|_| ds.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_without_snippet() {
        let d = Diagnostic::warning(DiagnosticCategory::Config, "something went wrong");
        let mut buf = Vec::new();
        render_diagnostic(&d, None, Color::Never, &mut buf).unwrap();
        assert_eq!(
            String::from_utf8(buf).unwrap(),
            "[Warning]: something went wrong\n"
        );
    }

    #[test]
    fn test_diagnostic_with_snippet() {
        let src = "{\n    \"key\": value\n}";
        let d = Diagnostic::error(DiagnosticCategory::Config, "expected a string")
            .with_location(DiagnosticLocation {
                file: PathBuf::from("config.json"),
                line: 2,
                column: 5,
                length: None,
            })
            .with_inline_source(src.to_string());

        let mut buf = Vec::new();
        render_diagnostic(&d, None, Color::Never, &mut buf).unwrap();
        insta::assert_snapshot!(String::from_utf8(buf).unwrap());
    }

    #[test]
    fn test_diagnostics_display() {
        let ds = Diagnostics(vec![
            Diagnostic::error(DiagnosticCategory::Schema, "first error"),
            Diagnostic::error(DiagnosticCategory::Schema, "second error"),
        ]);
        assert_eq!(ds.to_string(), "first error");
    }

    #[test]
    fn test_diagnostics_has_errors() {
        let mut ds = Diagnostics::new();
        assert!(!ds.has_errors());

        ds.push(Diagnostic::warning(DiagnosticCategory::Config, "warn"));
        assert!(!ds.has_errors());

        ds.push(Diagnostic::error(DiagnosticCategory::Schema, "err"));
        assert!(ds.has_errors());
    }

    #[test]
    fn test_diagnostics_max_cap() {
        let ds = Diagnostics(vec![
            Diagnostic::error(DiagnosticCategory::Schema, "error 1"),
            Diagnostic::error(DiagnosticCategory::Schema, "error 2"),
            Diagnostic::error(DiagnosticCategory::Schema, "error 3"),
            Diagnostic::error(DiagnosticCategory::Schema, "error 4"),
        ]);
        let mut buf = Vec::new();
        render_diagnostics(&ds, None, Color::Never, 2, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("[Error]: error 1"));
        assert!(output.contains("[Error]: error 2"));
        assert!(!output.contains("[Error]: error 3"));
        assert!(output.contains("... and 2 more"));
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
}
