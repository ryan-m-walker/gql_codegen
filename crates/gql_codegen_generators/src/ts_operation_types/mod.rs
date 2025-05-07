use std::{collections::HashSet, io::Write};

use anyhow::Result;
use apollo_compiler::{
    ExecutableDocument, Node, Schema,
    ast::{OperationType, Type},
    executable::{Field, Operation, Selection, SelectionSet},
    validation::Valid,
};
use gql_codegen_logger::Logger;
use serde::{Deserialize, Serialize};

use gql_codegen_formatter::{Formatter, FormatterConfig};
use gql_codegen_types::ReadResult;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TsOperationTypesGeneratorConfig {
    readonly: Option<bool>,
    operation_name_prefix: Option<String>,
    add_operation_type_suffix: Option<bool>,
    formatting: Option<FormatterConfig>,
}

#[derive(Debug)]
struct TsOperationTypesGenerator<'a, 'b> {
    config: &'a TsOperationTypesGeneratorConfig,
    schema: &'b Valid<Schema>,
    formatter: Formatter,
}

impl<'a, 'b> TsOperationTypesGenerator<'a, 'b> {
    pub fn new(config: &'a TsOperationTypesGeneratorConfig, schema: &'b Valid<Schema>) -> Self {
        let formatter_config = config.formatting.unwrap_or_default();

        Self {
            config,
            schema,
            formatter: Formatter::from_config(formatter_config),
        }
    }

    fn generate<T: Write>(&mut self, writer: &mut T, read_results: &[ReadResult]) -> Result<()> {
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
                    ExecutableDocument::parse_and_validate(self.schema, source_text, path).unwrap();

                for op in document.operations.iter() {
                    let add_operation_type_suffix =
                        self.config.add_operation_type_suffix.unwrap_or(false);

                    let op_type_name = if add_operation_type_suffix {
                        get_op_type_name(op)
                    } else {
                        String::new()
                    };

                    let op_prefix = self.config.operation_name_prefix.as_deref().unwrap_or("");

                    writeln!(writer)?;

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

                    write!(writer, "export interface {op_name}")?;
                    self.render_selection_set(writer, &op.selection_set)?;
                    writeln!(writer, ";\n")?;
                }
            }
        }

        Ok(())
    }

    fn render_selection_set<T: Write>(
        &mut self,
        writer: &mut T,
        selection_set: &SelectionSet,
    ) -> Result<()> {
        let selection_type = self.schema.get_object(selection_set.ty.as_str());

        if let Some(selection_type) = selection_type {
            self.formatter.inc_indent_level();
            writeln!(writer, " {{")?;

            // TODO: make non-nulll if selected
            write!(writer, "{}: ", self.formatter.indent("__typename"))?;
            writeln!(writer, "\"{}\";", selection_type.name)?;

            for selection in &selection_set.selections {
                match selection {
                    Selection::Field(field) => {
                        self.render_field(writer, &field)?;
                    }
                    _ => {}
                }
            }

            self.formatter.dec_indent_level();
            write!(writer, "{}", self.formatter.indent("}"))?;
        }

        Ok(())
    }

    fn render_field<T: Write>(&mut self, writer: &mut T, field: &Field) -> Result<()> {
        let field_name = field.alias.clone().unwrap_or(field.name.clone());

        write!(writer, "{}:", self.formatter.indent(&field_name),)?;

        if !field.selection_set.selections.is_empty() {
            self.render_selection_set(writer, &field.selection_set)?;
            writeln!(writer, ";")?;
        } else {
            let ty = self.render_type(field.ty());
            writeln!(writer, " {ty};")?;
        }

        Ok(())
    }

    fn render_type(&self, ty: &Type) -> String {
        match ty {
            Type::Named(name) => format!("{name} | null | undefined"),
            Type::NonNullNamed(name) => name.to_string(),
            Type::List(inner) => {
                format!("Array<{}> | null | undefined", self.render_type(inner))
            }
            Type::NonNullList(inner) => format!("Array<{}>", self.render_type(inner)),
        }
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
    schema: &Valid<Schema>,
    read_results: &[ReadResult],
    config: &TsOperationTypesGeneratorConfig,
    logger: &Logger,
) -> Result<()> {
    logger.debug("Running ts_operation_types generator...");
    let mut generator = TsOperationTypesGenerator::new(config, schema);
    generator.generate(writer, read_results)?;
    Ok(())
}
