use std::{
    collections::HashSet,
    io::{Result, Write},
};

use apollo_compiler::{
    ExecutableDocument, Node, Schema, ast::OperationType, executable::Operation,
};
use gql_codegen_types::ReadResult;
use serde::{Deserialize, Serialize};

use crate::Codegenerator;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TsOperationTypesGeneratorConfig {
    readonly: Option<bool>,
    operation_name_prefix: Option<String>,
    add_operation_type_suffix: Option<bool>,
}

#[derive(Debug)]
struct TsOperationTypesGenerator<'a> {
    config: &'a TsOperationTypesGeneratorConfig,
}

impl<'a> TsOperationTypesGenerator<'a> {
    pub fn new(config: &'a TsOperationTypesGeneratorConfig) -> Self {
        Self { config }
    }
}

impl Codegenerator for TsOperationTypesGenerator<'_> {
    fn generate<T: Write>(
        &self,
        writer: &mut T,
        schema: &Schema,
        read_results: &[ReadResult],
    ) -> Result<()> {
        let mut anonymous_op_count = 0;

        let mut op_names = HashSet::new();

        for read_result in read_results {
            for (i, source_text) in read_result.documents.iter().enumerate() {
                let path = if i == 0 {
                    read_result.path.clone()
                } else {
                    format!("{}#{}", read_result.path, i + 1)
                };

                let document =
                    ExecutableDocument::parse_and_validate(schema, source_text, path).unwrap();

                for op in document.operations.iter() {
                    let add_operation_type_suffix =
                        self.config.add_operation_type_suffix.unwrap_or(false);

                    let op_type_name = if add_operation_type_suffix {
                        get_op_type_name(op)
                    } else {
                        String::new()
                    };

                    let op_prefix = self.config.operation_name_prefix.as_deref().unwrap_or("");

                    let name = match &op.name {
                        Some(name) => name.to_string(),
                        None => {
                            anonymous_op_count += 1;
                            format!("Anonymous{anonymous_op_count}{op_type_name}")
                        }
                    };

                    let op_name = format!("{op_prefix}{name}{op_type_name}");

                    if op_names.contains(&op_name) {
                        panic!("Duplicate operation name: {op_name}");
                    }

                    op_names.insert(op_name.clone());

                    writeln!(writer, "export interface {op_name} {{")?;

                    writeln!(writer, "}};")?;
                }
            }
        }

        Ok(())
    }
}

fn get_op_type_name(op: &Node<Operation>) -> String {
    match op.operation_type {
        OperationType::Query => String::from("Query"),
        OperationType::Mutation => String::from("Mutation"),
        OperationType::Subscription => String::from("Subscription"),
    }
}

pub fn generate_operation_types(
    writer: &mut impl Write,
    schema: &Schema,
    read_results: &[ReadResult],
    config: &TsOperationTypesGeneratorConfig,
) -> Result<()> {
    let generator = TsOperationTypesGenerator::new(config);
    generator.generate(writer, schema, read_results)?;
    Ok(())
}
