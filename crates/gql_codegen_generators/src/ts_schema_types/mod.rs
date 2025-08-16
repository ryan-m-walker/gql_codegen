use std::io::Write;

use crate::common::gql_scalar_to_ts_scalar;
use anyhow::Result;
use apollo_compiler::{
    Name, Node, Schema,
    ast::Type,
    collections::HashMap,
    schema::{EnumType, ExtendedType, InputObjectType, InterfaceType, ObjectType, UnionType},
    validation::Valid,
};
use gql_codegen_formatter::{Formatter, FormatterConfig};
use gql_codegen_logger::Logger;
use gql_codegen_types::{Context, FragmentResult, OperationResult};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

mod helpers;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TsSchemaTypesGeneratorConfig {
    scalars: Option<HashMap<String, String>>,
    future_proof_enums: Option<bool>,
    use_native_enums: Option<bool>,
    readonly: Option<bool>,
    formatting: Option<FormatterConfig>,
}

#[derive(Debug)]
struct TsSchemaTypesGenerator<'a> {
    config: &'a TsSchemaTypesGeneratorConfig,
    ctx: Context<'a>,
}

impl<'a, 'b> TsSchemaTypesGenerator<'a> {
    pub fn new(config: &'a TsSchemaTypesGeneratorConfig, ctx: Context<'a>) -> Self {
        Self { config, ctx }
    }

    fn generate<T: Write>(&mut self, writer: &mut T, schema: &Valid<Schema>) -> Result<()> {
        for schema_type in schema.types.values() {
            if schema_type.is_built_in() {
                continue;
            }

            match schema_type {
                ExtendedType::Interface(node) => self.generate_interface(writer, node)?,
                ExtendedType::Enum(node) => self.generate_enum(writer, node)?,
                ExtendedType::Union(node) => self.generate_union(writer, node)?,
                ExtendedType::Object(node) => self.generate_object(writer, node)?,
                ExtendedType::InputObject(node) => self.generate_input_object(writer, node)?,
                _ => {}
            }
        }

        Ok(())
    }

    fn generate_interface<T: Write>(
        &mut self,
        writer: &mut T,
        node: &Node<InterfaceType>,
    ) -> Result<()> {
        let readonly = self.config.readonly.unwrap_or(false);

        writeln!(writer, "\nexport interface {} {{", node.name)?;
        self.ctx.formatter.inc_indent();

        for (name, field) in &node.fields {
            self.ctx
                .formatter
                .empty()
                .indent()
                .append_if(readonly, "readonly ")
                .append(name)
                .append(": ")
                .append(&self.render_type(&field.ty))
                .semi()
                .writeln(writer)?;
        }

        self.ctx.formatter.dec_indent();
        writeln!(writer, "}}")?;

        Ok(())
    }

    fn generate_enum<T: Write>(&self, writer: &mut T, node: &Node<EnumType>) -> Result<()> {
        let use_native_enums = self.config.use_native_enums.unwrap_or(false);
        let future_proof_enums = self.config.future_proof_enums.unwrap_or(false);

        if use_native_enums {
            writeln!(writer, "\nexport enum {} {{", node.name)?;

            for (name, value) in &node.values {
                writeln!(writer, "  {} = \"{}\",", name, value.value)?;
            }

            writeln!(writer, "}}")?;
            return Ok(());
        }

        write!(writer, "\nexport type {} = ", node.name)?;

        let values = node.values.values();
        let values_count = values.len();

        for (i, value) in values.enumerate() {
            write!(writer, " \"{}\"", value.value)?;

            if i < values_count - 1 {
                write!(writer, " |")?;
            }
        }

        if future_proof_enums {
            write!(writer, " | \"%future added value\"")?;
        }

        write!(writer, ";")?;
        writeln!(writer)?;

        Ok(())
    }

    fn generate_union<T: Write>(&self, writer: &mut T, node: &Node<UnionType>) -> Result<()> {
        write!(writer, "\nexport type {} = ", node.name)?;

        let members_count = node.members.len();

        for (i, value) in node.members.iter().enumerate() {
            write!(writer, " {}", value.name)?;

            if i < members_count - 1 {
                write!(writer, " |")?;
            }
        }

        writeln!(writer, ";")?;

        Ok(())
    }

    fn generate_object<T: Write>(&mut self, writer: &mut T, node: &Node<ObjectType>) -> Result<()> {
        let readonly = self.config.readonly.unwrap_or(false);

        writeln!(writer)?;

        self.render_description(writer, &node.description)?;

        write!(writer, "export interface {}", node.name)?;

        let interfaces_count = node.implements_interfaces.len();

        if interfaces_count > 0 {
            write!(writer, " extends")?;
        }

        for (i, interface) in node.implements_interfaces.iter().enumerate() {
            write!(writer, " {interface}")?;

            if i < interfaces_count - 1 {
                write!(writer, ",")?;
            }
        }

        writeln!(writer, " {{")?;

        self.ctx.formatter.inc_indent();

        let fmtd_type_name = self.ctx.formatter.format(&node.name).quote().to_string();

        self.ctx
            .formatter
            .empty()
            .indent()
            .append_if(readonly, "readonly ")
            .append("__typename: ")
            .append(&fmtd_type_name)
            .semi()
            .writeln(writer)?;

        for (name, field) in &node.fields {
            if field.description.is_some() {
                self.render_description(writer, &field.description)?;
            }

            self.ctx
                .formatter
                .empty()
                .indent()
                .append_if(readonly, "readonly ")
                .append(name)
                .append(": ")
                .append(&self.render_type(&field.ty))
                .semi()
                .writeln(writer)?;

            // writeln!(
            //     writer,
            //     "{}",
            //     self.formatter.indent_with_semicolon(&format!(
            //         "{}{}: {}",
            //         prefix,
            //         name,
            //         self.render_type(&field.ty)
            //     ))
            // )?;
        }

        self.ctx.formatter.dec_indent();
        writeln!(writer, "}}")?;

        Ok(())
    }

    fn generate_input_object<T: Write>(
        &mut self,
        writer: &mut T,
        node: &Node<InputObjectType>,
    ) -> Result<()> {
        let readonly = self.config.readonly.unwrap_or(false);

        writeln!(writer, "\nexport interface {} {{", node.name)?;
        self.ctx.formatter.inc_indent();

        let prefix = if readonly { "readonly " } else { "" };
        writeln!(
            writer,
            "{}",
            self.ctx
                .formatter
                .indent_with_semicolon(&format!("{}__typename: \"{}\"", prefix, node.name))
        )?;

        for (name, field) in &node.fields {
            if field.description.is_some() {
                self.render_description(writer, &field.description)?;
            }

            writeln!(
                writer,
                "{}",
                self.ctx.formatter.indent_with_semicolon(&format!(
                    "{}{}: {}",
                    prefix,
                    name,
                    self.render_type(&field.ty)
                ))
            )?;
        }

        self.ctx.formatter.dec_indent();
        writeln!(writer, "}}")?;

        Ok(())
    }

    fn render_type(&self, ty: &Type) -> String {
        match ty {
            Type::Named(name) => format!("{} | null | undefined", self.render_scalar_type(name)),
            Type::NonNullNamed(name) => self.render_scalar_type(name).to_string(),
            Type::List(inner) => {
                format!("Array<{}> | null | undefined", self.render_type(inner))
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

    fn render_description<T: Write>(
        &self,
        writer: &mut T,
        description: &Option<Node<str>>,
    ) -> Result<()> {
        if let Some(description) = description {
            writeln!(writer, "{}", self.ctx.formatter.indent("/**"))?;

            for line in description.lines() {
                writeln!(
                    writer,
                    "{}",
                    self.ctx.formatter.indent(&format!(" * {line}"))
                )?;
            }

            writeln!(writer, "{}", self.ctx.formatter.indent(" */"))?;
        }

        Ok(())
    }
}

pub fn generate_ts_schema_types(
    writer: &mut impl Write,
    schema: &Valid<Schema>,
    operations: &IndexMap<Name, OperationResult>,
    fragments: &IndexMap<Name, FragmentResult>,
    config: &TsSchemaTypesGeneratorConfig,
    logger: &Logger,
) -> Result<()> {
    logger.debug("Running ts_schema_types generator...");
    let formatter_config = config.formatting.unwrap_or_default();

    let ctx = Context::new(
        schema,
        operations,
        fragments,
        Formatter::from_config(formatter_config),
        logger,
    );

    let mut generator = TsSchemaTypesGenerator::new(config, ctx);
    generator.generate(writer, schema)?;
    Ok(())
}
