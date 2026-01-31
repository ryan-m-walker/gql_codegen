//! Stdout writer implementation.

use std::io::{self, Write};
use std::path::Path;

use super::Writer;

/// Writer that outputs to stdout.
///
/// Useful for:
/// - Piping to other tools
/// - Debugging generated output
/// - Single-file generation preview
#[derive(Debug, Default)]
pub struct StdoutWriter {
    /// Whether to include file path headers
    show_paths: bool,
}

impl StdoutWriter {
    pub fn new() -> Self {
        Self { show_paths: true }
    }

    /// Create a writer that omits file path headers (raw content only)
    pub fn without_paths() -> Self {
        Self { show_paths: false }
    }
}

impl Writer for StdoutWriter {
    fn write(&self, path: &Path, content: &[u8]) -> io::Result<()> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        if self.show_paths {
            writeln!(handle, "// {}", path.display())?;
        }

        handle.write_all(content)?;

        if self.show_paths {
            writeln!(handle)?; // blank line between files
        }

        Ok(())
    }

    fn matches_existing(&self, _path: &Path, _content: &[u8]) -> bool {
        // Stdout never "matches" - always write
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdout_writer_never_matches() {
        let writer = StdoutWriter::new();
        // Should always return false - stdout has no "existing" content
        assert!(!writer.matches_existing(Path::new("foo.txt"), b"hello"));
    }
}
