use std::path::{Path, PathBuf};

/// Cache of source file contents - keeps sources alive for borrowing
#[derive(Debug, Default)]
pub struct SourceCache {
    /// (file path, file contents) pairs
    files: Vec<(PathBuf, String)>,
}

impl SourceCache {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            files: Vec::with_capacity(capacity),
        }
    }

    /// Add pre-loaded content to the cache
    pub fn push(&mut self, path: PathBuf, content: String) -> usize {
        let idx = self.files.len();
        self.files.push((path, content));
        idx
    }

    /// Get a reference to a loaded file
    #[inline]
    pub fn get(&self, idx: usize) -> Option<(&Path, &str)> {
        self.files.get(idx).map(|(p, c)| (p.as_path(), c.as_str()))
    }

    /// Iterate over all loaded files
    pub fn iter(&self) -> impl Iterator<Item = (usize, &Path, &str)> {
        self.files
            .iter()
            .enumerate()
            .map(|(i, (p, c))| (i, p.as_path(), c.as_str()))
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}
