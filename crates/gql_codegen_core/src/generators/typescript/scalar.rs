use std::collections::HashSet;

use apollo_compiler::Node;
use apollo_compiler::schema::{ExtendedType, ScalarType};

use crate::config::ScalarConfig;
use crate::generators::GeneratorContext;
use crate::generators::typescript::helpers::{
    get_export_kw, render_decl_closing, render_decl_opening, render_description,
};
use crate::{Error, Result};

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
    // If using utility types, scalars are rendered as references
    // to the top level Scalars type so we skip rendering them here.
    if ctx.options.use_utility_types {
        return Ok(());
    }

    let name = scalar.name.as_str();
    let export = get_export_kw(ctx);

    if is_builtin_scalar(name) {
        return Ok(());
    }

    let custom_type = ctx.options.scalars.get(scalar.name.as_str());

    match custom_type {
        Some(ScalarConfig::Simple(ts_type)) => {
            render_description(ctx, &scalar.description, 0)?;
            writeln!(ctx.writer, "{export}type {name} = {ts_type};")?;
            writeln!(ctx.writer)?;
        }
        Some(ScalarConfig::Detailed { input, output }) => {
            render_description(ctx, &scalar.description, 0)?;
            writeln!(
                ctx.writer,
                "{export}type {name} = {{\n  input: {input};\n  output: {output};\n}};"
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
            writeln!(ctx.writer, "{export}type {name} = {default_type};")?;
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
                return Err(Error::UnknownScalar(name.to_string()));
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
