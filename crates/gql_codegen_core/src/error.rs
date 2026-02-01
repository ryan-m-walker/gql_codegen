//! Error types for gql_codegen_core

use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to read schema '{0}': {1}")]
    SchemaRead(PathBuf, String),

    #[error("Failed to parse schema: {0}")]
    SchemaParse(String),

    #[error("Schema validation error: {0}")]
    SchemaValidation(String),

    #[error("Failed to read '{0}': {1}")]
    FileRead(PathBuf, String),

    #[error("Failed to parse GraphQL document in '{file}': {message}")]
    DocumentParse { file: PathBuf, message: String },

    #[error("Invalid glob pattern '{pattern}': {message}")]
    InvalidGlob { pattern: String, message: String },

    #[error("Unknown plugin: '{0}'")]
    UnknownPlugin(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Generation error: {0}")]
    Generation(String),

    #[error("Duplicate operation name '{name}' in {file}")]
    DuplicateOperation { name: String, file: PathBuf },

    #[error("Duplicate fragment name '{name}' in {file}")]
    DuplicateFragment { name: String, file: PathBuf },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
