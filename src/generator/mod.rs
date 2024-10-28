use apollo_compiler::RootDatabase;
use apollo_parser::cst::Document;

pub(self) mod common;
#[cfg(test)]
mod tests;
pub mod ts_operation_types;
pub mod ts_schema_types;

pub trait CodeGenerator {
    fn generate(&self, document: &Document, db: &RootDatabase) -> String;
}
