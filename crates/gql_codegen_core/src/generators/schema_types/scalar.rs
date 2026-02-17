use std::collections::HashSet;

use apollo_compiler::Node;
use apollo_compiler::schema::{ExtendedType, ScalarType};

use crate::Result;
use crate::config::ScalarConfig;
use crate::diagnostic::{Diagnostic, DiagnosticCategory};
use crate::generators::GeneratorContext;
use crate::generators::common::helpers::{get_export_kw, render_decl_closing, render_decl_opening};
use crate::generators::schema_types::helpers::render_description;

const DEFAULT_SCALARS: [(&str, &str); 5] = [
    ("ID", "string"),
    ("String", "string"),
    ("Boolean", "boolean"),
    ("Int", "number"),
    ("Float", "number"),
];

fn is_builtin_scalar(name: &str) -> bool {
    DEFAULT_SCALARS.iter().any(|(n, _)| *n == name)
}

pub(crate) fn render_scalar(ctx: &mut GeneratorContext, scalar: &Node<ScalarType>) -> Result<()> {
    let raw_name = scalar.name.as_str();
    let export = get_export_kw(ctx);

    if is_builtin_scalar(raw_name) {
        return Ok(());
    }

    let type_name = ctx.transform_type_name(raw_name);
    let custom_type = ctx.options.scalars.get(raw_name);

    match custom_type {
        Some(ScalarConfig::Simple(ts_type)) => {
            render_description(ctx, &scalar.description, 0)?;
            writeln!(ctx.writer, "{export}type {type_name} = {ts_type};")?;
            writeln!(ctx.writer)?;
        }
        Some(ScalarConfig::Detailed { input, output }) => {
            render_description(ctx, &scalar.description, 0)?;
            writeln!(
                ctx.writer,
                "{export}type {type_name} = {{\n  input: {input};\n  output: {output};\n}};"
            )?;
            writeln!(ctx.writer)?;
        }
        None => {
            let default_type = ctx
                .options
                .default_scalar_type
                .as_deref()
                .unwrap_or("unknown");

            render_description(ctx, &scalar.description, 0)?;
            writeln!(ctx.writer, "{export}type {type_name} = {default_type};")?;
            writeln!(ctx.writer)?;
        }
    }

    Ok(())
}

/// Renders the scalars section at the top of the generated file.
/// This is mainly to support backwards compatibility with graphql-codegen
/// as this is how they handle scalars. Will not render for `sgc` preset.
pub(crate) fn render_scalars(ctx: &mut GeneratorContext) -> Result<()> {
    writeln!(
        ctx.writer,
        "/** All built-in and custom scalars, mapped to their actual values */"
    )?;

    render_decl_opening(ctx, "Scalars", None)?;

    let mut rendered = HashSet::new();

    for (name, default_type) in DEFAULT_SCALARS {
        let custom = ctx.options.scalars.get(name);

        if rendered.contains(name) {
            continue;
        }

        let (input, output) = match custom {
            Some(ScalarConfig::Simple(value)) => (value.as_str(), value.as_str()),
            Some(ScalarConfig::Detailed { input, output }) => (input.as_str(), output.as_str()),
            None => (default_type, default_type),
        };

        writeln!(
            ctx.writer,
            "  {name}: {{ input: {input}; output: {output}; }}"
        )?;

        rendered.insert(name);
    }

    for (_, ty) in ctx.schema.types.iter() {
        if let ExtendedType::Scalar(scalar) = ty {
            let name = scalar.name.as_str();

            if rendered.contains(name) {
                continue;
            }

            let custom_type = ctx.options.scalars.get(name);

            if ctx.options.strict_scalars && custom_type.is_none() {
                return Err(Diagnostic::error(
                    DiagnosticCategory::Generation,
                    format!("Unknown scalar type '{name}'. Please override it using the \"scalars\" configuration field!"),
                ).into());
            }

            let default_type = ctx
                .options
                .default_scalar_type
                .as_deref()
                .unwrap_or("unknown");

            let (input, output) = match custom_type {
                Some(ScalarConfig::Simple(value)) => (value.as_str(), value.as_str()),
                Some(ScalarConfig::Detailed { input, output }) => (input.as_str(), output.as_str()),
                None => (default_type, default_type),
            };

            writeln!(
                ctx.writer,
                "  {name}: {{ input: {input}; output: {output}; }}"
            )?;

            rendered.insert(name);
        }
    }

    render_decl_closing(ctx)?;
    writeln!(ctx.writer)?;

    Ok(())
}
