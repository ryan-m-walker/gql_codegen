use std::collections::HashMap;
use std::io::Write;

use anyhow::Result;
use apollo_compiler::{
    Name, Schema,
    ast::{OperationType, Type},
    validation::Valid,
};
use gql_codegen_logger::Logger;
use gql_codegen_types::{FragmentResult, OperationResult};
use indexmap::{IndexMap, IndexSet};
use operation_tree::{OperationTree, OperationTreeInput};
use serde::{Deserialize, Serialize};

use crate::common::gql_scalar_to_ts_scalar;
use gql_codegen_formatter::{Formatter, FormatterConfig};

mod operation_tree;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TsOperationTypesGeneratorConfig {
    scalars: Option<HashMap<String, String>>,
    readonly: Option<bool>,
    operation_name_prefix: Option<String>,
    add_operation_type_suffix: Option<bool>,
    formatting: Option<FormatterConfig>,
}

#[derive(Debug)]
struct TsOperationTypesGenerator<'a> {
    config: &'a TsOperationTypesGeneratorConfig,
    schema: &'a Valid<Schema>,
    operations: &'a IndexMap<Name, OperationResult>,
    fragments: &'a IndexMap<Name, FragmentResult>,
    logger: &'a Logger,
    formatter: Formatter,
}

impl<'a> TsOperationTypesGenerator<'a> {
    pub fn new(
        config: &'a TsOperationTypesGeneratorConfig,
        schema: &'a Valid<Schema>,
        operations: &'a IndexMap<Name, OperationResult>,
        fragments: &'a IndexMap<Name, FragmentResult>,
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
        for (name, fragment) in self.fragments {
            let fragment_tree = OperationTree::new(
                self.schema,
                OperationTreeInput::Fragment(fragment),
                self.fragments,
            )?;

            writeln!(writer, "\nexport interface {name} {{")?;
            self.render_selection_set(writer, &fragment_tree, &fragment_tree.root_selection_refs)?;
            writeln!(writer, "}}")?;
        }

        for (name, operation) in self.operations {
            let operation_tree = OperationTree::new(
                self.schema,
                OperationTreeInput::Operation(operation),
                self.fragments,
            )?;

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
        self.formatter.inc_indent();

        for selection_ref in selection_refs {
            let Some(field) = operation_tree.normalized_fields.get(selection_ref) else {
                continue;
            };

            let readonly = self.config.readonly.is_some_and(|r| r);

            let include_directive = field.directives.get("include");
            let skip_directive = field.directives.get("skip");

            let required = include_directive.is_none()
                && skip_directive.is_none()
                && field.field_type.is_non_null();

            self.formatter
                .empty()
                .append_if(readonly, "readonly ")
                .append(&field.field_name)
                .indent()
                .append_if(!required, "?")
                .append(": ")
                .write(writer)?;

            if field.selection_refs.is_empty() {
                // TODO: figure out why this isn't working
                // TODO: render optional if no explicit selection for __typename
                if field.field_name == "__typename" {
                    self.formatter
                        .format(&field.parent_type_name)
                        .quote()
                        .semi()
                        .writeln(writer)?;

                    continue;
                }

                self.formatter
                    .format(&self.render_type(&field.field_type))
                    .semi()
                    .writeln(writer)?;

                continue;
            }

            self.formatter
                .format(match field.field_type {
                    Type::Named(_) => "{",
                    Type::NonNullNamed(_) => "{",
                    Type::List(_) => "Array<{",
                    Type::NonNullList(_) => "Array<{",
                })
                .writeln(writer)?;

            self.render_selection_set(writer, operation_tree, &field.selection_refs)?;

            self.formatter
                .format(match field.field_type {
                    Type::Named(_) => "}",
                    Type::NonNullNamed(_) => "}",
                    Type::List(_) => "}> | null",
                    Type::NonNullList(_) => "}>",
                })
                .indent()
                .semi()
                .writeln(writer)?;
        }

        self.formatter.dec_indent();
        Ok(())
    }

    fn render_type(&self, ty: &Type) -> String {
        match ty {
            Type::Named(name) => format!("{} | null", self.render_scalar_type(name)),
            Type::NonNullNamed(name) => self.render_scalar_type(name).to_string(),
            Type::List(inner) => {
                format!("Array<{}> | null", self.render_type(inner))
            }
            Type::NonNullList(inner) => format!("Array<{}>", self.render_type(inner)),
        }
    }

    fn render_scalar_type(&self, name: &str) -> String {
        if let Some(scalar_type) = self
            .config
            .scalars
            .as_ref()
            .and_then(|scalars| scalars.get(name))
        {
            return scalar_type.to_string();
        }

        gql_scalar_to_ts_scalar(name).to_string()
    }
}

pub fn generate_operation_types(
    writer: &mut impl Write,
    schema: &Valid<Schema>,
    operations: &IndexMap<Name, OperationResult>,
    fragments: &IndexMap<Name, FragmentResult>,
    config: &TsOperationTypesGeneratorConfig,
    logger: &Logger,
) -> Result<()> {
    let mut generator =
        TsOperationTypesGenerator::new(config, schema, operations, fragments, logger);
    generator.generate(writer)?;
    Ok(())
}
