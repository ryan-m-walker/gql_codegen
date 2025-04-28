use std::io::{Result, Write};

use apollo_compiler::{Schema, collections::HashMap, schema::ExtendedType};
use generate_default_scalars::generate_default_scalars;
use generate_interface::generate_interface;
use generate_object::generate_object;
use generate_scalar::generate_scalar;
use generate_union::generate_union;
use gql_codegen_types::ReadResult;
use serde::{Deserialize, Serialize};

use crate::Codegenerator;

mod common;
mod generate_default_scalars;
mod generate_enum;
mod generate_input_object;
mod generate_interface;
mod generate_object;
mod generate_scalar;
mod generate_union;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TsSchemaTypesGeneratorConfig {
    scalars: Option<HashMap<String, String>>,
    future_proof_enums: Option<bool>,
    use_native_enums: Option<bool>,
    readonly: Option<bool>,
}

#[derive(Debug)]
struct TsSchemaTypesGenerator<'a> {
    config: &'a TsSchemaTypesGeneratorConfig,
}

impl<'a> TsSchemaTypesGenerator<'a> {
    pub fn new(config: &'a TsSchemaTypesGeneratorConfig) -> Self {
        Self { config }
    }
}

impl Codegenerator for TsSchemaTypesGenerator<'_> {
    fn generate<T: Write>(
        &self,
        writer: &mut T,
        schema: &Schema,
        _read_results: &[ReadResult],
    ) -> Result<()> {
        generate_default_scalars(writer, self.config).unwrap();

        for schema_type in schema.types.values() {
            if schema_type.is_built_in() {
                continue;
            }

            match schema_type {
                ExtendedType::Scalar(node) => generate_scalar(writer, node, self.config)?,
                ExtendedType::Interface(node) => generate_interface(writer, node, self.config)?,
                ExtendedType::Enum(node) => {
                    generate_enum::generate_enum(writer, node, self.config)?
                }
                ExtendedType::Union(node) => generate_union(writer, node, self.config)?,
                ExtendedType::Object(node) => generate_object(writer, node, self.config)?,
                ExtendedType::InputObject(node) => {
                    generate_input_object::generate_input_object(writer, node, self.config)?
                }
            }
        }

        Ok(())
    }
}

pub fn generate_ts_schema_types(
    writer: &mut impl Write,
    schema: &Schema,
    config: &TsSchemaTypesGeneratorConfig,
) -> Result<()> {
    let generator = TsSchemaTypesGenerator::new(config);
    generator.generate(writer, schema, &[])?;
    Ok(())
}
