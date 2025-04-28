use std::io::{Result, Write};

use apollo_compiler::{Schema, validation::Valid};
use gql_codegen_types::ReadResult;

pub mod ts_operation_types;
pub mod ts_schema_types;

pub trait Codegenerator {
    fn generate<T: Write>(
        &self,
        writer: &mut T,
        schema: &Schema,
        read_results: &[ReadResult],
    ) -> Result<()>;
}
