use anyhow::Result;
use apollo_compiler::{
    Name, Node, Schema,
    ast::{Argument, OperationType, Type, Value, VariableDefinition},
    validation::Valid,
};
use gql_codegen_formatter::{Formatter, FormatterConfig};
use gql_codegen_logger::Logger;
use gql_codegen_types::{Context, FragmentResult, OperationResult};
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use std::io::Write;

use crate::operation_tree::{OperationTree, OperationTreeInput};

mod field_formatter;

const MAX_CHARACTERS_PER_LINE: usize = 80;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphQLTag {
    #[serde(alias = "gql")]
    GQL,
    #[serde(alias = "graphql")]
    GraphQL,
    #[serde(alias = "comment")]
    Comment,
    #[serde(alias = "none")]
    None,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DocumentGeneratorConfig {
    formatting: Option<FormatterConfig>,
    graphql_tag: Option<GraphQLTag>,
    sort_fields: Option<bool>,
}

struct DocumentsGenerator<'a> {
    config: &'a DocumentGeneratorConfig,
    ctx: Context<'a>,
}

impl<'a> DocumentsGenerator<'a> {
    pub fn new(config: &'a DocumentGeneratorConfig, ctx: Context<'a>) -> Self {
        Self { config, ctx }
    }

    pub fn generate<T: Write>(&mut self, writer: &mut T) -> Result<()> {
        for (name, operation) in self.ctx.operations {
            let operation_tree =
                OperationTree::new(OperationTreeInput::Operation(operation), self.ctx.clone())?;

            let graphql_tag = get_graphql_tag(self.config);

            writeln!(writer, "\nexport const {name} = {graphql_tag}`")?;

            let operation_name = match &operation.operation.name {
                Some(name) => name.clone().to_string(),
                None => String::from(""),
            };

            self.ctx.formatter.inc_indent();
            self.ctx
                .formatter
                .empty()
                .indent()
                .append(match operation.operation.operation_type {
                    OperationType::Query => "query",
                    OperationType::Mutation => "mutation",
                    OperationType::Subscription => "subscription",
                })
                .append_if(!operation_name.is_empty(), &format!(" {operation_name}"))
                .write(writer)?;

            if !operation.operation.variables.is_empty() {
                self.render_variables(writer, &operation.operation.variables)?;
            }

            self.render_selection_set(
                writer,
                &operation_tree,
                &operation_tree.root_selection_refs,
            )?;

            self.ctx.formatter.dec_indent();
            writeln!(writer, "`")?;
        }

        Ok(())
    }

    fn render_variables<T: Write>(
        &mut self,
        writer: &mut T,
        variables: &[Node<VariableDefinition>],
    ) -> Result<()> {
        self.ctx.formatter.empty().append("(").write(writer)?;

        for (i, variable) in variables.iter().enumerate() {
            self.ctx
                .formatter
                .empty()
                .append("$")
                .append(variable.name.as_ref())
                .append(": ")
                .append(&Self::render_type(&variable.ty))
                .write(writer)?;

            if i < variables.len() - 1 {
                self.ctx.formatter.empty().append(", ").write(writer)?;
            }
        }

        write!(writer, ")")?;

        Ok(())
    }

    fn render_type(ty: &Type) -> String {
        match ty {
            Type::Named(name) => name.to_string(),
            Type::NonNullNamed(name) => format!("{name}!"),
            Type::List(inner) => format!("[{}]", Self::render_type(inner)),
            Type::NonNullList(inner) => format!("[{}]!", Self::render_type(inner)),
        }
    }

    fn render_selection_set<T: Write>(
        &mut self,
        writer: &mut T,
        operation_tree: &OperationTree,
        selection_refs: &IndexSet<String>,
    ) -> Result<()> {
        self.ctx.formatter.empty().append(" {").writeln(writer)?;
        self.ctx.formatter.inc_indent();

        for selection_ref in selection_refs {
            let Some(field) = operation_tree.normalized_fields.get(selection_ref) else {
                continue;
            };

            self.ctx
                .formatter
                .empty()
                .append(&field.field_name)
                .indent()
                .write(writer)?;

            if !field.arguments.is_empty() {
                self.render_arguments(writer, &field.arguments)?;
            }

            if field.selection_refs.is_empty() {
                writeln!(writer)?;
            } else {
                self.render_selection_set(writer, operation_tree, &field.selection_refs)?;
            }
        }

        self.ctx.formatter.dec_indent();
        self.ctx
            .formatter
            .empty()
            .indent()
            .append("}")
            .writeln(writer)?;

        Ok(())
    }

    fn render_arguments<T: Write>(
        &mut self,
        writer: &mut T,
        arguments: &[Node<Argument>],
    ) -> Result<()> {
        self.ctx.formatter.empty().append("(").write(writer)?;

        for (i, argument) in arguments.iter().enumerate() {
            self.ctx
                .formatter
                .empty()
                .append(&argument.name)
                .append(": ")
                .append(&Self::render_value(&argument.value))
                .write(writer)?;

            if i < arguments.len() - 1 {
                self.ctx.formatter.empty().append(", ").write(writer)?;
            }
        }

        write!(writer, ")")?;

        Ok(())
    }

    fn render_value(value: &Value) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::String(str) => format!("\"{str}\""),
            Value::Int(int) => int.to_string(),
            Value::Float(float) => float.to_string(),
            Value::Boolean(bool) => bool.to_string(),
            Value::Variable(name) => format!("${name}"),
            Value::Enum(name) => name.to_string(),
            Value::List(list) => {
                let value = list
                    .iter()
                    .map(|v| Self::render_value(v))
                    .collect::<Vec<_>>()
                    .join(", ");

                format!("[{value}]")
            }
            Value::Object(obj) => {
                let value = obj
                    .iter()
                    .map(|(name, value)| format!("{name}: {value}"))
                    .collect::<Vec<_>>()
                    .join(", ");

                format!("{{ {value} }}")
            }
        }
    }
}

pub fn generate_documents(
    writer: &mut impl Write,
    schema: &Valid<Schema>,
    operations: &IndexMap<Name, OperationResult>,
    fragments: &IndexMap<Name, FragmentResult>,
    config: &DocumentGeneratorConfig,
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

    let mut generator = DocumentsGenerator::new(config, ctx);
    generator.generate(writer)?;
    Ok(())
}

/// Takes ownership so that we can return the original value if
/// it's not too long and not do an unnecessary allocation
fn format_field(field: String, max_chars_per_line: usize) -> String {
    let len = field.len();

    if len <= max_chars_per_line {
        return field;
    }

    let mut new_field = String::with_capacity(len + len / 2);

    new_field
}

fn get_graphql_tag(config: &DocumentGeneratorConfig) -> String {
    let graphql_tag = config.graphql_tag.unwrap_or(GraphQLTag::GQL);

    match graphql_tag {
        GraphQLTag::GQL => String::from("gql"),
        GraphQLTag::GraphQL => String::from("graphql"),
        GraphQLTag::Comment => String::from("/** GraphQL */"),
        GraphQLTag::None => String::new(),
    }
}
