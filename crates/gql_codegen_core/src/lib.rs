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
mod generators;
mod schema;

// Public API - Main entry points
pub use codegen::{GenerateInput, GenerateResult, GeneratedFile, generate, generate_from_input};
pub use config::{CodegenConfig, FormattingOptions, OutputConfig, PluginConfig, PluginOptions, StringOrArray};
pub use error::{Error, Result};

// Public API - Building blocks for custom I/O handling
// Use these when you need control over file loading, caching, etc.
pub use documents::{
    CollectedDocuments, ParsedFragment, ParsedOperation, SourceCache, collect_documents,
    load_sources,
};
pub use extract::{ExtractConfig, Extracted};
pub use schema::load_schema;
