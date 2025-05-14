use std::io::Write;

use anyhow::Result;
use apollo_compiler::{
    Name, Node, Schema,
    ast::{OperationType, Type},
    executable::Operation,
    validation::Valid,
};
use gql_codegen_logger::Logger;
use gql_codegen_types::{FragmentResult, OperationResult};
use indexmap::{IndexMap, IndexSet};
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
        self.formatter.inc_indent();

        for selection_ref in selection_refs {
            let Some(field) = operation_tree.normalized_fields.get(selection_ref) else {
                continue;
            };

            let include_directive = field.directives.get("include");
            let skip_directive = field.directives.get("skip");

            let required = include_directive.is_none()
                && skip_directive.is_none()
                && field.field_type.is_non_null();

            self.formatter
                .format(&field.field_name)
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
            Type::Named(name) => format!("{} | null", self.wrap_scalar_type(name)),
            Type::NonNullNamed(name) => self.wrap_scalar_type(name).to_string(),
            Type::List(inner) => {
                format!("Array<{}> | null", self.render_type(inner))
            }
            Type::NonNullList(inner) => format!("Array<{}>", self.render_type(inner)),
        }
    }

    fn wrap_scalar_type(&self, name: &str) -> String {
        let is_scalar = self.schema.get_scalar(name).is_some();
        if is_scalar {
            return self
                .formatter
                .format(name)
                .quote()
                .prepend("Scalars[")
                .append("]")
                .to_string();
        }

        name.to_string()
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
