//! Filesystem-based writer implementation.

use std::path::Path;
use std::{fs, io};

use super::Writer;

/// Writer that writes to the real filesystem.
#[derive(Debug, Default)]
pub struct FsWriter;

impl Writer for FsWriter {
    fn write(&self, path: &Path, content: &[u8]) -> io::Result<()> {
        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, content)
    }

    fn matches_existing(&self, path: &Path, content: &[u8]) -> bool {
        match fs::read(path) {
            Ok(existing) => existing == content,
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_write_creates_parent_dirs() {
        let temp_dir = env::temp_dir().join("sgc_test_fs_writer");
        let _ = fs::remove_dir_all(&temp_dir); // Clean up from previous runs

        let writer = FsWriter::new();
        let path = temp_dir.join("nested/dir/file.txt");

        writer.write(&path, b"hello").unwrap();

        assert!(path.exists());
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello");

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_matches_existing() {
        let temp_dir = env::temp_dir().join("sgc_test_matches");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let writer = FsWriter::new();
        let path = temp_dir.join("test.txt");

        // File doesn't exist - no match
        assert!(!writer.matches_existing(&path, b"hello"));

        // Write file
        fs::write(&path, b"hello").unwrap();

        // Same content - matches
        assert!(writer.matches_existing(&path, b"hello"));

        // Different content - no match
        assert!(!writer.matches_existing(&path, b"world"));

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
