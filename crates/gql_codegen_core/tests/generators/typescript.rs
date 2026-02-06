mod enums;
mod inputs;
mod interfaces;
mod misc;
mod naming;
mod objects;
mod scalars;
mod unions;

use gql_codegen_core::test_utils::TestGen;
use gql_codegen_core::{Error, GenerateResult, PluginOptions};

/// Helper to generate TypeScript output from schema files with given options.
///
/// Always includes base.graphql plus any additional schema files specified.
pub fn generate_with_options(schema_files: &[&str], options: PluginOptions) -> String {
    let mut builder = TestGen::new();
    for file in schema_files {
        builder = builder.schema(file);
    }
    builder.options(options).generate()
}

/// Like generate_with_options but returns Result for error testing.
pub fn try_generate_with_options(
    schema_files: &[&str],
    options: PluginOptions,
) -> Result<GenerateResult, Error> {
    let mut builder = TestGen::new();
    for file in schema_files {
        builder = builder.schema(file);
    }
    builder.options(options).try_generate()
}
