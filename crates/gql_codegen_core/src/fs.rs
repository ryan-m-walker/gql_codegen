//! Filesystem abstraction for testability and flexibility
//!
//! Provides a trait for file operations so core logic can work with
//! real filesystem or in-memory files (for testing).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::SystemTime;
use std::{fs, io};

use globset::{Glob, GlobSetBuilder};
use walkdir::WalkDir;

/// File metadata for caching
#[derive(Debug, Clone)]
pub struct FileMeta {
    pub mtime_secs: u64,
    pub size: u64,
}

/// Filesystem trait - implement for different backends
pub trait FileSystem: Send + Sync {
    /// Read file contents as string
    fn read_to_string(&self, path: &Path) -> io::Result<String>;

    /// Read file contents as bytes
    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;

    /// Write content to file
    fn write(&self, path: &Path, content: &[u8]) -> io::Result<()>;

    /// Get file metadata (mtime, size)
    fn metadata(&self, path: &Path) -> io::Result<FileMeta>;

    /// Check if path exists
    fn exists(&self, path: &Path) -> bool;

    /// Create directory and parents
    fn create_dir_all(&self, path: &Path) -> io::Result<()>;

    /// Expand glob patterns to matching file paths
    fn glob(&self, patterns: &[&str], base_dir: &Path) -> Vec<PathBuf>;
}

/// Real filesystem implementation
pub struct RealFs;

impl FileSystem for RealFs {
    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        fs::read_to_string(path)
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        fs::read(path)
    }

    fn write(&self, path: &Path, content: &[u8]) -> io::Result<()> {
        fs::write(path, content)
    }

    fn metadata(&self, path: &Path) -> io::Result<FileMeta> {
        let meta = fs::metadata(path)?;
        let mtime = meta.modified()?;
        let mtime_secs = mtime
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Ok(FileMeta {
            mtime_secs,
            size: meta.len(),
        })
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        fs::create_dir_all(path)
    }

    fn glob(&self, patterns: &[&str], base_dir: &Path) -> Vec<PathBuf> {
        let mut builder = GlobSetBuilder::new();
        for pattern in patterns {
            if let Ok(glob) = Glob::new(pattern) {
                builder.add(glob);
            }
        }
        let glob_set = match builder.build() {
            Ok(gs) => gs,
            Err(_) => return Vec::new(),
        };

        WalkDir::new(base_dir)
            .into_iter()
            .filter_entry(|e| !is_ignored(e))
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                let path = e.path();
                let relative = path.strip_prefix(base_dir).unwrap_or(path);
                glob_set.is_match(relative)
            })
            .map(|e| e.path().to_path_buf())
            .collect()
    }
}

fn is_ignored(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.') || s == "node_modules" || s == "target")
        .unwrap_or(false)
}

/// In-memory filesystem for testing
pub struct MemoryFs {
    files: RwLock<HashMap<PathBuf, Vec<u8>>>,
}

impl MemoryFs {
    pub fn new() -> Self {
        Self {
            files: RwLock::new(HashMap::new()),
        }
    }

    /// Add a file to the in-memory filesystem
    pub fn add_file(&self, path: impl Into<PathBuf>, content: impl Into<Vec<u8>>) {
        self.files
            .write()
            .unwrap()
            .insert(path.into(), content.into());
    }
}

impl Default for MemoryFs {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystem for MemoryFs {
    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        let files = self.files.read().unwrap();
        files
            .get(path)
            .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "file not found"))
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        let files = self.files.read().unwrap();
        files
            .get(path)
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "file not found"))
    }

    fn write(&self, path: &Path, content: &[u8]) -> io::Result<()> {
        self.files
            .write()
            .unwrap()
            .insert(path.to_path_buf(), content.to_vec());
        Ok(())
    }

    fn metadata(&self, path: &Path) -> io::Result<FileMeta> {
        let files = self.files.read().unwrap();
        files
            .get(path)
            .map(|bytes| FileMeta {
                mtime_secs: 0, // In-memory doesn't track mtime
                size: bytes.len() as u64,
            })
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "file not found"))
    }

    fn exists(&self, path: &Path) -> bool {
        self.files.read().unwrap().contains_key(path)
    }

    fn create_dir_all(&self, _path: &Path) -> io::Result<()> {
        Ok(()) // No-op for in-memory
    }

    fn glob(&self, patterns: &[&str], _base_dir: &Path) -> Vec<PathBuf> {
        let mut builder = GlobSetBuilder::new();
        for pattern in patterns {
            if let Ok(glob) = Glob::new(pattern) {
                builder.add(glob);
            }
        }
        let glob_set = match builder.build() {
            Ok(gs) => gs,
            Err(_) => return Vec::new(),
        };

        let files = self.files.read().unwrap();
        files
            .keys()
            .filter(|path| glob_set.is_match(path))
            .cloned()
            .collect()
    }
}
