//! Output writing abstractions.
//!
//! Provides a `Writer` trait for writing generated files to different backends:
//! - `FsWriter` - writes to the real filesystem
//! - `MemoryWriter` - writes to an in-memory buffer (for testing, NAPI)

mod fs;
mod memory;
mod noop;
mod stdout;

pub use fs::FsWriter;
pub use memory::MemoryWriter;
pub use noop::NoopWriter;
pub use stdout::StdoutWriter;

use std::io;
use std::path::{Path, PathBuf};

use rayon::prelude::*;

use crate::codegen::GeneratedFile;

/// Trait for writing generated output files.
pub trait Writer: Send + Sync {
    /// Write content to path, creating parent directories as needed.
    fn write(&self, path: &Path, content: &[u8]) -> io::Result<()>;

    /// Check if file at path already has the given content.
    /// Used for skip optimization (avoid unnecessary writes).
    fn matches_existing(&self, path: &Path, content: &[u8]) -> bool;
}

/// Result of writing output files.
#[derive(Debug, Default)]
pub struct WriteResult {
    /// Files that were written (sorted for determinism)
    pub written: Vec<PathBuf>,
    /// Files skipped because content already matched (sorted for determinism)
    pub skipped: Vec<PathBuf>,
    /// Files that failed to write (sorted for determinism)
    pub errors: Vec<(PathBuf, String)>,
}

impl WriteResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Outcome of writing a single file.
enum WriteOutcome {
    Written(PathBuf),
    Skipped(PathBuf),
    Error(PathBuf, String),
}

/// Write generated files using the provided writer.
///
/// Uses parallel iteration for I/O performance (each file written concurrently).
/// Results are sorted by path for deterministic output.
pub fn write_outputs(files: &[GeneratedFile], writer: &dyn Writer) -> WriteResult {
    // Parallel I/O - each iteration returns an outcome
    let outcomes: Vec<WriteOutcome> = files
        .par_iter()
        .map(|file| {
            let path = PathBuf::from(&file.path);
            let content = file.content.as_bytes();

            // Skip if content already matches
            if writer.matches_existing(&path, content) {
                return WriteOutcome::Skipped(path);
            }

            match writer.write(&path, content) {
                Ok(()) => WriteOutcome::Written(path),
                Err(e) => WriteOutcome::Error(path, e.to_string()),
            }
        })
        .collect();

    // Partition outcomes into result vectors
    let mut written = Vec::new();
    let mut skipped = Vec::new();
    let mut errors = Vec::new();

    for outcome in outcomes {
        match outcome {
            WriteOutcome::Written(p) => written.push(p),
            WriteOutcome::Skipped(p) => skipped.push(p),
            WriteOutcome::Error(p, e) => errors.push((p, e)),
        }
    }

    // Sort for deterministic output
    written.sort();
    skipped.sort();
    errors.sort_by(|a, b| a.0.cmp(&b.0));

    WriteResult {
        written,
        skipped,
        errors,
    }
}
