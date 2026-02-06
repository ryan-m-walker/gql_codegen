use apollo_compiler::Node;
use apollo_compiler::collections::IndexSet;
use apollo_compiler::schema::ComponentName;

use crate::generators::GeneratorContext;
use crate::{DeclarationKind, Result};

/// Returns the export keyword based if exports are enabled.
pub(crate) fn get_export_kw(ctx: &GeneratorContext) -> &'static str {
    if ctx.options.no_export { "" } else { "export " }
}

/// Returns the readonly keyword based on immutability configuration.
pub(crate) fn get_readonly_kw(ctx: &GeneratorContext) -> &'static str {
    if ctx.options.immutable_types {
        "readonly "
    } else {
        ""
    }
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

/// TODO: make this less strict, allowing strings which we parse or fallback to default
pub(crate) fn get_decl_kind_kw(ctx: &GeneratorContext) -> &'static str {
    match ctx.options.declaration_kind {
        Some(DeclarationKind::Type) | None => "type",
        Some(DeclarationKind::Interface) => "interface",
        Some(DeclarationKind::Class) => "class",
        Some(DeclarationKind::AbstractClass) => "abstract class",
    }
}

/// Renders the opening of a GraphQL object type declaration.
pub(crate) fn render_decl_opening(
    ctx: &mut GeneratorContext,
    name: &str,
    implements_interfaces: Option<&IndexSet<ComponentName>>,
) -> Result<()> {
    let export = get_export_kw(ctx);
    let decl_kind = get_decl_kind_kw(ctx);

    let separator = match ctx.options.declaration_kind {
        Some(DeclarationKind::Type) | None => " = ",
        _ => " ",
    };

    write!(ctx.writer, "{export}{decl_kind} {name}{separator}")?;

    if let Some(interfaces) = implements_interfaces {
        if !interfaces.is_empty() {
            match ctx.options.declaration_kind {
                Some(DeclarationKind::Type) | None => {
                    for interface in interfaces {
                        write!(ctx.writer, "{interface}")?;
                        write!(ctx.writer, " & ")?;
                    }
                }
                _ => {
                    write!(ctx.writer, "implements ")?;

                    for (i, interface) in interfaces.iter().enumerate() {
                        write!(ctx.writer, "{interface}")?;
                        if i < interfaces.len() - 1 {
                            write!(ctx.writer, ", ")?;
                        }
                    }

                    write!(ctx.writer, " ")?;
                }
            }
        }
    }

    writeln!(ctx.writer, "{{")?;

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

/// Convert a GraphQL description to a TypeScript doc comment.
///
/// **Example:**
/// ``` graphql
/// """
/// This is a description
/// """
///
/// ```
/// **Output:**
/// ``` typescript
/// /** This is a description */
/// ```
pub(crate) fn render_description(
    ctx: &mut GeneratorContext,
    description: &Option<Node<str>>,
    indent_level: usize,
) -> Result<()> {
    if ctx.options.disable_descriptions {
        return Ok(());
    }

    let Some(description) = description else {
        return Ok(());
    };

    if description.is_empty() {
        return Ok(());
    }

    let indent = if indent_level > 0 {
        " ".repeat(indent_level * 2)
    } else {
        "".to_string()
    };

    if description.lines().count() > 1 {
        writeln!(ctx.writer, "{indent}/**")?;
        for line in description.lines() {
            writeln!(ctx.writer, "{indent} * {line}")?;
        }
        writeln!(ctx.writer, "{indent} */")?;

        return Ok(());
    }

    writeln!(ctx.writer, "{indent}/** {description} */")?;

    Ok(())
}
