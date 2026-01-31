//! Code generators for different output formats
//!
//! Each generator takes the schema + documents and produces code.

mod document_transform;
mod documents;
mod typescript;
mod typescript_operations;

pub use typescript::generate_typescript;
pub use typescript_operations::generate_typescript_operations;
pub use documents::generate_documents;

use std::io::Write;

use apollo_compiler::validation::Valid;
use apollo_compiler::Schema;
use indexmap::IndexMap;
use apollo_compiler::Name;

use crate::config::PluginOptions;
use crate::documents::{ParsedOperation, ParsedFragment};
use crate::Result;

/// Context passed to all generators
pub struct GeneratorContext<'a> {
    pub schema: &'a Valid<Schema>,
    pub operations: &'a IndexMap<Name, ParsedOperation<'a>>,
    pub fragments: &'a IndexMap<Name, ParsedFragment<'a>>,
    pub options: &'a PluginOptions,
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
pub fn run_generator(
    name: &str,
    ctx: &GeneratorContext,
    writer: &mut dyn Write,
) -> Result<()> {
    match name {
        "typescript" => generate_typescript(ctx, writer),
        "typescript-operations" => generate_typescript_operations(ctx, writer),
        "typescript-documents" | "documents" => generate_documents(ctx, writer),
        _ => Err(crate::Error::UnknownPlugin(name.to_string())),
    }
}
