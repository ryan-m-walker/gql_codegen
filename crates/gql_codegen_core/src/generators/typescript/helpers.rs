use apollo_compiler::Node;
use apollo_compiler::collections::IndexSet;
use apollo_compiler::schema::ComponentName;

use crate::generators::GeneratorContext;
use crate::{DeclarationKind, Result};

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
    ctx: &mut GeneratorContext,
    name: &str,
    _implements_interfaces: Option<&IndexSet<ComponentName>>,
) -> Result<()> {
    let export = get_export_kw(ctx);
    let decl_kind = get_decl_kind_kw(ctx);

    let separator = match ctx.options.declaration_kind {
        Some(DeclarationKind::Type) | None => " = ",
        _ => " ",
    };

    writeln!(ctx.writer, "{export}{decl_kind} {name}{separator}{{")?;
    Ok(())
}

pub(crate) fn render_decl_closing(ctx: &mut GeneratorContext) -> Result<()> {
    let semi = match ctx.options.declaration_kind {
        Some(DeclarationKind::Type) | None => ";",
        _ => "",
    };

    writeln!(ctx.writer, "}}{semi}")?;
    Ok(())
}

pub(crate) fn render_description(
    ctx: &mut GeneratorContext,
    description: &Option<Node<str>>,
    indent_level: usize,
) -> Result<()> {
    let indent = if indent_level > 0 {
        " ".repeat(indent_level * 2)
    } else {
        "".to_string()
    };

    if let Some(description) = description {
        if description.is_empty() {
            return Ok(());
        } else if description.lines().count() > 1 {
            writeln!(ctx.writer, "{indent}/**")?;
            for line in description.lines() {
                writeln!(ctx.writer, "{indent} * {line}")?;
            }
            writeln!(ctx.writer, "{indent} */")?;

            return Ok(());
        } else {
            writeln!(ctx.writer, "{indent}/** {description} */")?;
        }
    }

    Ok(())
}
