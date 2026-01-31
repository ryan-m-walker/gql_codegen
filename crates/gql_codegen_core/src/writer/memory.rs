//! In-memory writer implementation.

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use super::Writer;

/// Writer that stores output in memory.
///
/// Useful for:
/// - Testing (inspect generated output without filesystem)
/// - NAPI integration (return files to Node.js)
/// - Dry-run mode
#[derive(Debug, Default)]
pub struct MemoryWriter {
    files: RwLock<HashMap<PathBuf, Vec<u8>>>,
}

impl MemoryWriter {
    pub fn new() -> Self {
        Self {
            files: RwLock::new(HashMap::new()),
        }
    }

    /// Get all written files.
    pub fn files(&self) -> HashMap<PathBuf, Vec<u8>> {
        self.files.read().unwrap().clone()
    }

    /// Get content of a specific file.
    pub fn get(&self, path: &Path) -> Option<Vec<u8>> {
        self.files.read().unwrap().get(path).cloned()
    }

    /// Get content as string (convenience for text files).
    pub fn get_string(&self, path: &Path) -> Option<String> {
        self.get(path)
            .and_then(|bytes| String::from_utf8(bytes).ok())
    }

    /// Check if a file was written.
    pub fn contains(&self, path: &Path) -> bool {
        self.files.read().unwrap().contains_key(path)
    }

    /// Number of files written.
    pub fn len(&self) -> usize {
        self.files.read().unwrap().len()
    }

    /// Check if no files were written.
    pub fn is_empty(&self) -> bool {
        self.files.read().unwrap().is_empty()
    }
}

impl Writer for MemoryWriter {
    fn write(&self, path: &Path, content: &[u8]) -> io::Result<()> {
        self.files
            .write()
            .unwrap()
            .insert(path.to_path_buf(), content.to_vec());
        Ok(())
    }

    fn matches_existing(&self, path: &Path, content: &[u8]) -> bool {
        self.files
            .read()
            .unwrap()
            .get(path)
            .map(|existing| existing == content)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_and_read() {
        let writer = MemoryWriter::new();

        writer.write(Path::new("foo.txt"), b"hello").unwrap();
        writer.write(Path::new("bar.txt"), b"world").unwrap();

        assert_eq!(writer.len(), 2);
        assert_eq!(writer.get_string(Path::new("foo.txt")), Some("hello".into()));
        assert_eq!(writer.get_string(Path::new("bar.txt")), Some("world".into()));
    }

    #[test]
    fn test_matches_existing() {
        let writer = MemoryWriter::new();

        // No file - no match
        assert!(!writer.matches_existing(Path::new("foo.txt"), b"hello"));

        // Write file
        writer.write(Path::new("foo.txt"), b"hello").unwrap();

        // Same content - matches
        assert!(writer.matches_existing(Path::new("foo.txt"), b"hello"));

        // Different content - no match
        assert!(!writer.matches_existing(Path::new("foo.txt"), b"world"));
    }

    #[test]
    fn test_overwrite() {
        let writer = MemoryWriter::new();

        writer.write(Path::new("foo.txt"), b"first").unwrap();
        writer.write(Path::new("foo.txt"), b"second").unwrap();

        assert_eq!(writer.len(), 1);
        assert_eq!(writer.get_string(Path::new("foo.txt")), Some("second".into()));
    }
}
