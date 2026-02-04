use std::io::Write;

use apollo_compiler::Node;

use crate::Result;
use crate::{DeclarationKind, generators::GeneratorContext};

pub(crate) fn get_export_kw(ctx: &GeneratorContext) -> &'static str {
    if ctx.options.no_export { "" } else { "export " }
}

/// Map built-in GraphQL scalar types to TypeScript types.
/// Returns None for custom scalars that need separate handling.
pub(crate) fn gql_scalar_to_ts(name: &str) -> Option<&'static str> {
    match name {
        "String" | "ID" => Some("string"),
        "Int" | "Float" => Some("number"),
        "Boolean" => Some("boolean"),
        _ => None,
    }
}

pub(crate) fn get_decl_kind_kw(ctx: &GeneratorContext) -> &'static str {
    match ctx.options.declaration_kind {
        Some(DeclarationKind::Type) | None => "type",
        Some(DeclarationKind::Interface) => "interface",
        Some(DeclarationKind::Class) => "class",
        Some(DeclarationKind::AbstractClass) => "abstract class",
    }
}

pub(crate) fn get_readonly_kw(ctx: &GeneratorContext) -> &'static str {
    if ctx.options.immutable_types {
        "readonly "
    } else {
        ""
    }
}

pub(crate) fn get_optional_prop_modifier(ctx: &GeneratorContext) -> &'static str {
    if ctx.options.avoid_optionals { "" } else { "?" }
}

pub(crate) fn render_decl_opening(
    name: &str,
    ctx: &GeneratorContext,
    writer: &mut dyn Write,
) -> Result<()> {
    let export = get_export_kw(ctx);
    let decl_kind = get_decl_kind_kw(ctx);

    let separator = match ctx.options.declaration_kind {
        Some(DeclarationKind::Type) | None => " = ",
        _ => " ",
    };

    writeln!(writer, "{export}{decl_kind} {name}{separator}{{")?;
    Ok(())
}

pub(crate) fn render_decl_closing(ctx: &GeneratorContext, writer: &mut dyn Write) -> Result<()> {
    let semi = match ctx.options.declaration_kind {
        Some(DeclarationKind::Type) | None => ";",
        _ => "",
    };

    writeln!(writer, "}}{semi}")?;
    Ok(())
}

pub(crate) fn render_description(
    description: &Option<Node<str>>,
    indent: usize,
    writer: &mut dyn Write,
) -> Result<()> {
    let indent = if indent > 0 {
        " ".repeat(indent * 2)
    } else {
        "".to_string()
    };

    if let Some(description) = description {
        if description.is_empty() {
            return Ok(());
        } else if description.lines().count() > 1 {
            writeln!(writer, "{indent}/**")?;
            for line in description.lines() {
                writeln!(writer, "{indent} * {line}")?;
            }
            writeln!(writer, "{indent} */")?;

            return Ok(());
        } else {
            writeln!(writer, "{indent}/** {description} */")?;
        }
    }

    Ok(())
}
