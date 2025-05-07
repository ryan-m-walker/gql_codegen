use std::{collections::HashMap, io::Write};

use anyhow::{Result, anyhow};
use apollo_compiler::{
    Name, Node, Schema,
    ast::{OperationType, Type},
    executable::{Field, Fragment, Operation, Selection, SelectionSet},
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
    operations: &'a HashMap<Name, Node<Operation>>,
    fragments: &'a HashMap<Name, Node<Fragment>>,
    logger: &'a Logger,
    formatter: Formatter,
}

impl<'a> TsOperationTypesGenerator<'a> {
    pub fn new(
        config: &'a TsOperationTypesGeneratorConfig,
        schema: &'a Valid<Schema>,
        operations: &'a HashMap<Name, Node<Operation>>,
        fragments: &'a HashMap<Name, Node<Fragment>>,
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
            let operation_tree = OperationTree::new(operation, self.fragments)?;

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

            let rendered_key = format!("{}: ", field.field_name);
            write!(writer, "{}", self.formatter.indent(&rendered_key))?;

            if field.selection_refs.is_empty() {
                if field.field_name == "__typename" {
                    write!(writer, "\"{}\"", &field.parent_type_name)?;
                    writeln!(writer, "{}", self.formatter.semicolon())?;
                    continue;
                }

                let rendered_type = self.render_type(&field.field_type);
                write!(writer, "Scalars[\"{rendered_type}\"]",)?;
                writeln!(writer, "{}", self.formatter.semicolon())?;
                continue;
            }

            writeln!(writer, "{{")?;
            self.render_selection_set(writer, operation_tree, &field.selection_refs)?;
            writeln!(writer, "{}", self.formatter.indent_with_semicolon("}"))?;
        }

        self.formatter.dec_indent_level();
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
    operations: &HashMap<Name, Node<Operation>>,
    fragments: &HashMap<Name, Node<Fragment>>,
    config: &TsOperationTypesGeneratorConfig,
    logger: &Logger,
) -> Result<()> {
    let mut generator =
        TsOperationTypesGenerator::new(config, schema, operations, fragments, logger);
    generator.generate(writer)?;
    Ok(())
}
