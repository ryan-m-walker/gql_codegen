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

pub type Result<T> = std::result::Result<T, CodegenError>;
