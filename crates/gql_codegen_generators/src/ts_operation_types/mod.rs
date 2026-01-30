use std::collections::HashMap;
use std::io::Write;

use crate::operation_tree::{OperationTree, OperationTreeInput};
use anyhow::Result;
use apollo_compiler::{Name, Schema, ast::Type, validation::Valid};
use gql_codegen_logger::Logger;
use gql_codegen_types::{Context, FragmentResult, OperationResult};
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};

use crate::common::gql_scalar_to_ts_scalar;
use gql_codegen_formatter::{Formatter, FormatterConfig};

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
    pub ctx: Context<'a>,
}

impl<'a> TsOperationTypesGenerator<'a> {
    pub fn new(config: &'a TsOperationTypesGeneratorConfig, ctx: Context<'a>) -> Self {
        Self { config, ctx }
    }

    fn generate<T: Write>(&mut self, writer: &mut T) -> Result<()> {
        for (name, fragment) in self.ctx.fragments {
            let fragment_tree =
                OperationTree::new(OperationTreeInput::Fragment(fragment), self.ctx.clone())?;

            writeln!(writer, "\nexport interface {name} {{")?;
            self.render_selection_set(writer, &fragment_tree, &fragment_tree.root_selection_refs)?;
            writeln!(writer, "}}")?;
        }

        for (name, operation) in self.ctx.operations {
            let operation_tree =
                OperationTree::new(OperationTreeInput::Operation(operation), self.ctx.clone())?;

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
        self.ctx.formatter.inc_indent();

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

            self.ctx
                .formatter
                .empty()
                .append_if(readonly, "readonly ")
                .append(&field.field_name)
                .indent()
                .append_if(!required, "?")
                .append(": ")
                .write(writer)?;

            if field.root_selection_refs.is_empty() {
                // TODO: figure out why this isn't working
                // TODO: render optional if no explicit selection for __typename
                if field.field_name == "__typename" {
                    self.ctx
                        .formatter
                        .empty()
                        .append(&field.parent_type_name)
                        .quote()
                        .semi()
                        .writeln(writer)?;

                    continue;
                }

                self.ctx
                    .formatter
                    .empty()
                    .append(&self.render_type(&field.field_type))
                    .semi()
                    .writeln(writer)?;

                continue;
            }

            self.ctx
                .formatter
                .empty()
                .append(match field.field_type {
                    Type::Named(_) => "{",
                    Type::NonNullNamed(_) => "{",
                    Type::List(_) => "Array<{",
                    Type::NonNullList(_) => "Array<{",
                })
                .writeln(writer)?;

            self.render_selection_set(writer, operation_tree, &field.root_selection_refs)?;

            self.ctx
                .formatter
                .empty()
                .append(match field.field_type {
                    Type::Named(_) => "}",
                    Type::NonNullNamed(_) => "}",
                    Type::List(_) => "}> | null",
                    Type::NonNullList(_) => "}>",
                })
                .indent()
                .semi()
                .writeln(writer)?;
        }

        self.ctx.formatter.dec_indent();
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
    let formatter_config = config.formatting.unwrap_or_default();

    let ctx = Context::new(
        schema,
        operations,
        fragments,
        Formatter::from_config(formatter_config),
        logger,
    );

    let mut generator = TsOperationTypesGenerator::new(config, ctx);
    generator.generate(writer)?;
    Ok(())
}
