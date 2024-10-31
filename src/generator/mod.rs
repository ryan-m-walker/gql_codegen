use apollo_compiler::RootDatabase;

pub(self) mod common;
#[cfg(test)]
mod tests;
pub mod ts_operation_types;
pub mod ts_schema_types;
pub mod zod_schemas;

pub trait CodeGenerator {
    fn generate(&self, db: &RootDatabase) -> String;
}
