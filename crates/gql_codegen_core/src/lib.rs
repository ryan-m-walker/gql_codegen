//! # gql_codegen_core
//!
//! Core GraphQL code generation library. Receives configuration and produces
//! generated TypeScript code.
//!
//! This crate is designed to be called from:
//! - Node.js via NAPI-RS bindings
//! - Rust CLI directly
//! - Any other Rust code
//!
//! ## Architecture
//!
//! ```text
//! codegen.ts (user config)
//!     ↓
//! Node.js CLI (loads & validates config)
//!     ↓
//! gql_codegen_core::generate(config)
//!     ↓
//! GenerateResult { files: [...] }
//! ```

pub mod cache;
mod codegen;
mod config;
mod documents;
mod error;
mod extract;
pub mod fs;
mod generators;
mod schema;
mod source_cache;
pub mod writer;

// Public API - Main entry points
pub use codegen::{
    GenerateCachedResult, GenerateInput, GenerateResult, GeneratedFile, generate, generate_cached,
    generate_from_input,
};
pub use config::{
    CodegenConfig, FormattingOptions, GraphqlTag, OutputConfig, PluginConfig, PluginOptions,
    StringOrArray,
};
pub use error::{Error, Result};

// Public API - Building blocks for custom I/O handling
// Use these when you need control over file loading, caching, etc.
pub use documents::{
    CollectedDocuments, ParsedFragment, ParsedOperation, collect_documents,
    expand_document_globs, load_sources, load_sources_from_paths,
};
pub use source_cache::SourceCache;
pub use extract::{ExtractConfig, Extracted};
pub use schema::{load_schema, load_schema_from_contents, resolve_schema_paths};
pub use writer::{FsWriter, MemoryWriter, StdoutWriter, WriteResult, Writer, write_outputs};
