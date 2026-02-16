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

use std::borrow::Cow;
use std::io::Write;

use apollo_compiler::validation::Valid;
use apollo_compiler::{Name, Schema};
use indexmap::IndexMap;

use crate::Result;
use crate::config::{NamingCase, NamingConvention, GeneratorOptions};
use crate::diagnostic::{Diagnostic, DiagnosticCategory, Diagnostics};
use crate::documents::{ParsedFragment, ParsedOperation};

/// Context passed to all generators
pub struct GeneratorContext<'a> {
    pub schema: &'a Valid<Schema>,
    pub operations: &'a IndexMap<Name, ParsedOperation<'a>>,
    pub fragments: &'a IndexMap<Name, ParsedFragment<'a>>,
    pub options: &'a GeneratorOptions,
    pub writer: &'a mut dyn Write,
    pub diagnostics: &'a mut Diagnostics,
}

impl GeneratorContext<'_> {
    /// Convenience: push a warning diagnostic.
    pub fn warn(&mut self, category: DiagnosticCategory, message: impl Into<String>) {
        self.diagnostics
            .push(Diagnostic::warning(category, message));
    }
}

impl GeneratorContext<'_> {
    /// Apply the configured `typeNames` naming convention to a type name,
    /// then apply `types_prefix` / `types_suffix` if configured.
    pub fn transform_type_name<'a>(&self, name: &'a str) -> Cow<'a, str> {
        let (case, transform_underscore) = get_type_name_case(self.options);
        let cased = case.apply(name, transform_underscore);

        match (&self.options.type_name_prefix, &self.options.type_name_suffix) {
            (None, None) => cased,
            (prefix, suffix) => {
                let prefix = prefix.as_deref().unwrap_or("");
                let suffix = suffix.as_deref().unwrap_or("");
                Cow::Owned(format!("{prefix}{cased}{suffix}"))
            }
        }
    }
}

/// Extract the naming case for type names from options.
fn get_type_name_case(options: &GeneratorOptions) -> (NamingCase, bool) {
    match &options.naming_convention {
        None => (NamingCase::Keep, false),
        Some(NamingConvention::Simple(case)) => (*case, false),
        Some(NamingConvention::Detailed(config)) => {
            let case = config.type_names.unwrap_or(NamingCase::Keep);
            (case, config.transform_underscore)
        }
    }
}

/// Run a named generator
pub fn run_generator(name: &str, ctx: &mut GeneratorContext) -> Result<()> {
    match name {
        "schema-types" | "typescript" => generate_typescript(ctx),
        "operation-types" | "typescript-operations" => generate_typescript_operations(ctx),
        "typed-documents" | "typescript-documents" | "documents" => generate_documents(ctx),
        _ => Err(Diagnostic::error(
            DiagnosticCategory::Generation,
            format!("Unknown generator: '{name}'"),
        )
        .into()),
    }
}
