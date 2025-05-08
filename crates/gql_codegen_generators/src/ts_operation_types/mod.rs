use std::{collections::HashMap, io::Write};

use anyhow::Result;
use apollo_compiler::{
    Name, Node, Schema,
    ast::{FragmentDefinition, OperationDefinition, OperationType, Type},
    executable::Operation,
    validation::Valid,
};
use gql_codegen_logger::Logger;
use indexmap::IndexSet;
use operation_tree::OperationTree;
use serde::{Deserialize, Serialize};

use gql_codegen_formatter::{Formatter, FormatterConfig};

mod operation_tree;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TsOperationTypesGeneratorConfig {
    readonly: Option<bool>,
    operation_name_prefix: Option<String>,
    add_operation_type_suffix: Option<bool>,
    formatting: Option<FormatterConfig>,
}

#[derive(Debug)]
struct TsOperationTypesGenerator<'a> {
    config: &'a TsOperationTypesGeneratorConfig,
    schema: &'a Valid<Schema>,
    operations: &'a HashMap<Name, Node<OperationDefinition>>,
    fragments: &'a HashMap<Name, Node<FragmentDefinition>>,
    logger: &'a Logger,
    formatter: Formatter,
}

impl<'a> TsOperationTypesGenerator<'a> {
    pub fn new(
        config: &'a TsOperationTypesGeneratorConfig,
        schema: &'a Valid<Schema>,
        operations: &'a HashMap<Name, Node<OperationDefinition>>,
        fragments: &'a HashMap<Name, Node<FragmentDefinition>>,
        logger: &'a Logger,
    ) -> Self {
        let formatter_config = config.formatting.unwrap_or_default();

        Self {
            config,
            schema,
            operations,
            fragments,
            logger,
            formatter: Formatter::from_config(formatter_config),
        }
    }

    fn generate<T: Write>(&mut self, writer: &mut T) -> Result<()> {
        for (name, operation) in self.operations {
            let operation_tree = OperationTree::new(self.schema, operation, self.fragments)?;

            writeln!(writer, "\nexport interface {name} {{")?;
            self.render_selection_set(
                writer,
                &operation_tree,
                &operation_tree.root_selection_refs,
            )?;
            writeln!(writer, "}}")?;
        }

        Ok(())
    }

    fn render_selection_set<T: Write>(
        &mut self,
        writer: &mut T,
        operation_tree: &OperationTree,
        selection_refs: &IndexSet<String>,
    ) -> Result<()> {
        self.formatter.inc_indent_level();

        for selection_ref in selection_refs {
            let Some(field) = operation_tree.normalized_fields.get(selection_ref) else {
                continue;
            };

            // TODO: skip and include directives
            let rendered_key = self
                .render_field_key(&field.field_name, &field.field_type)
                .to_string();

            write!(writer, "{}", self.formatter.indent(&rendered_key))?;

            if field.selection_refs.is_empty() {
                if field.field_name == "__typename" {
                    write!(writer, "\"{}\"", &field.parent_type_name)?;
                    writeln!(writer, "{}", self.formatter.semicolon())?;
                    continue;
                }

                let rendered_type = self.render_type(&field.field_type);
                write!(writer, "{rendered_type}",)?;
                writeln!(writer, "{}", self.formatter.semicolon())?;
                continue;
            }

            writeln!(
                writer,
                "{}",
                self.render_selection_opening(&field.field_type)
            )?;

            self.render_selection_set(writer, operation_tree, &field.selection_refs)?;
            write!(
                writer,
                "{}",
                self.formatter
                    .indent_with_semicolon(&self.render_selection_closing(&field.field_type))
            )?;

            writeln!(writer)?;
        }

        self.formatter.dec_indent_level();
        Ok(())
    }

    fn render_field_key(&self, field_name: &str, ty: &Type) -> String {
        let optional = if ty.is_non_null() {
            String::new()
        } else {
            String::from("?")
        };

        format!("{field_name}{optional}: ")
    }

    fn render_selection_opening(&self, ty: &Type) -> String {
        match ty {
            Type::Named(_) => String::from("{"),
            Type::NonNullNamed(_) => String::from("{"),
            Type::List(_) => String::from("Array<{"),
            Type::NonNullList(_) => String::from("Array<{"),
        }
    }

    fn render_selection_closing(&self, ty: &Type) -> String {
        match ty {
            Type::Named(_) => String::from("}"),
            Type::NonNullNamed(_) => String::from("}"),
            Type::List(_) => String::from("}> | null"),
            Type::NonNullList(_) => String::from("}>"),
        }
    }

    fn render_type(&self, ty: &Type) -> String {
        match ty {
            Type::Named(name) => format!("{name} | null"),
            Type::NonNullNamed(name) => name.to_string(),
            Type::List(inner) => {
                format!("Array<{}> | null", self.render_type(inner))
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
    operations: &HashMap<Name, Node<OperationDefinition>>,
    fragments: &HashMap<Name, Node<FragmentDefinition>>,
    config: &TsOperationTypesGeneratorConfig,
    logger: &Logger,
) -> Result<()> {
    let mut generator =
        TsOperationTypesGenerator::new(config, schema, operations, fragments, logger);
    generator.generate(writer)?;
    Ok(())
}
