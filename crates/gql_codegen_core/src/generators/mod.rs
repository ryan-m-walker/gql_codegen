//! Code generators for different output formats
//!
//! Each generator takes the schema + documents and produces code.

mod common;
mod document_transform;
mod documents;
mod typescript;
mod typescript_operations;

pub use documents::generate_documents;
pub use typescript::generate_typescript;
pub use typescript_operations::generate_typescript_operations;

use std::io::Write;

use apollo_compiler::validation::Valid;
use apollo_compiler::{Name, Schema};
use indexmap::IndexMap;

use crate::Result;
use crate::config::PluginOptions;
use crate::documents::{ParsedFragment, ParsedOperation};

/// Context passed to all generators
pub struct GeneratorContext<'a> {
    pub schema: &'a Valid<Schema>,
    pub operations: &'a IndexMap<Name, ParsedOperation<'a>>,
    pub fragments: &'a IndexMap<Name, ParsedFragment<'a>>,
    pub options: &'a PluginOptions,
    pub writer: &'a mut dyn Write,
}

/// Trait for code generators
pub trait Generator {
    fn name(&self) -> &'static str;
    fn generate(&self, ctx: &GeneratorContext, writer: &mut dyn Write) -> Result<()>;
}

/// Output from a generator (for in-memory generation)
#[derive(Debug, Clone)]
pub struct GeneratorOutput {
    pub content: String,
}

/// Run a named generator
pub fn run_generator(name: &str, ctx: &mut GeneratorContext) -> Result<()> {
    match name {
        "typescript" => generate_typescript(ctx),
        "typescript-operations" => generate_typescript_operations(ctx),
        "typescript-documents" | "documents" => generate_documents(ctx),
        _ => Err(crate::Error::UnknownPlugin(name.to_string())),
    }
}
