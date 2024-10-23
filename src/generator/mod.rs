use apollo_compiler::RootDatabase;
use apollo_parser::cst::Document;

#[cfg(test)]
mod tests;
pub mod ts_schema_types;
pub mod typescript;

pub trait CodeGenerator {
    fn generate(&self, document: &Document, db: &RootDatabase) -> String;
}
