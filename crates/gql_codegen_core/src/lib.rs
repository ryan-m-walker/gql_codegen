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

mod codegen;
mod config;
mod documents;
mod error;
mod extract;
mod generators;
mod schema;

// Public API
pub use codegen::{GenerateResult, GeneratedFile, generate};
pub use config::{CodegenConfig, FormattingOptions, OutputConfig, PluginConfig, PluginOptions};
pub use error::{Error, Result};
