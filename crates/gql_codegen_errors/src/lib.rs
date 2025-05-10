use std::fmt;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, CodegenError>;

#[derive(Debug, Error)]
pub struct GQLValidationError {
    pub message: String,
    pub locations: Vec<(usize, usize)>,
}

impl GQLValidationError {
    pub fn new(message: String, locations: Vec<(usize, usize)>) -> Self {
        Self { message, locations }
    }
}

impl fmt::Display for GQLValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GQL Validation Error: {}", self.message)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CodegenError {
    #[error("Invalid glob pattern: {0}")]
    InvalidGlobPattern(String),

    #[error("Error reading file")]
    FileReadError,

    #[error("File has no extension")]
    NoExtension,

    #[error("Unsupported file type: {0}")]
    InvalidFileType(String),

    #[error("Parsing error: {0}")]
    ParsingError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}
