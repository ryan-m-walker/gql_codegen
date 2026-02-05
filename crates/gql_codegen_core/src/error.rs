//! Error types for gql_codegen_core

use std::fmt;
use std::path::PathBuf;

use apollo_compiler::validation::DiagnosticList;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// Structured config error with source location for rich rendering
#[derive(Debug)]
pub struct ConfigError {
    pub message: String,
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    /// Config file contents for source display
    pub source_text: String,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Config error in {}: {}", self.file.display(), self.message)
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to read schema '{0}': {1}")]
    SchemaRead(PathBuf, String),

    #[error("{0}")]
    SchemaParse(DiagnosticList),

    #[error("{0}")]
    SchemaValidation(DiagnosticList),

    #[error("Failed to read '{0}': {1}")]
    FileRead(PathBuf, String),

    #[error("Failed to parse GraphQL document in '{file}': {message}")]
    DocumentParse { file: PathBuf, message: String },

    #[error("Invalid glob pattern '{pattern}': {message}")]
    InvalidGlob { pattern: String, message: String },

    #[error("Unknown plugin: '{0}'")]
    UnknownPlugin(String),

    #[error("{0}")]
    Config(ConfigError),

    #[error("Generation error: {0}")]
    Generation(String),

    #[error("Duplicate operation name '{name}' in {file}")]
    DuplicateOperation { name: String, file: PathBuf },

    #[error("Duplicate fragment name '{name}' in {file}")]
    DuplicateFragment { name: String, file: PathBuf },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error(
        "Unknown scalar type {0}. Please override it using the \"scalars\" configuration field!"
    )]
    UnknownScalar(String),
}
