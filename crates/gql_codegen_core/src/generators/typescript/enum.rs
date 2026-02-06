use std::borrow::Cow;

use apollo_compiler::Node;
use apollo_compiler::schema::EnumType;

use crate::Result;
use crate::config::{NamingCase, NamingConvention, PluginOptions};
use crate::generators::GeneratorContext;
use crate::generators::typescript::helpers::{get_export_kw, render_description};

/// Render a GraphQL enum type as TypeScript type to the current writer.
///
/// **Example Input:**
/// ``` graphql
/// enum Status {
///   ACTIVE
///   INACTIVE
/// }
/// ```
///
/// **Output:**
/// ``` typescript
/// type Status = 'ACTIVE' | 'INACTIVE';
/// ```
pub(crate) fn render_enum(ctx: &mut GeneratorContext, enum_type: &Node<EnumType>) -> Result<()> {
    render_description(ctx, &enum_type.description, 0)?;
    let cased_name = ctx.transform_type_name(enum_type.name.as_str());
    let enum_name = apply_enum_affixes(&cased_name, ctx.options);

    if ctx.options.enums_as_types {
        render_as_type_union(ctx, &enum_name, enum_type)?;
    } else {
        render_as_ts_enum(ctx, &enum_name, enum_type)?;
    }

    writeln!(ctx.writer)?;
    Ok(())
}

/// Render enum as TypeScript type union.
///
/// **Example:**
/// ``` typescript
/// type Status = 'ACTIVE' | 'INACTIVE';
/// ```
fn render_as_type_union(
    ctx: &mut GeneratorContext,
    enum_name: &str,
    enum_type: &Node<EnumType>,
) -> Result<()> {
    let export = get_export_kw(ctx);

    write!(ctx.writer, "{export}type {enum_name} =")?;

    let values: Vec<_> = enum_type.values.keys().collect();
    for (i, value) in values.iter().enumerate() {
        let transformed = transform_enum_value(value.as_str(), ctx.options);
        if i == 0 {
            write!(ctx.writer, " '{transformed}'")?;
        } else {
            write!(ctx.writer, " | '{transformed}'")?;
        }
    }

    if ctx.options.future_proof_enums {
        write!(ctx.writer, " | '%future added value'")?;
    }

    writeln!(ctx.writer, ";")?;
    Ok(())
}

/// Render as actual TypeScript enum.
///
/// **Example:**
/// ``` typescript
/// enum Status {
///     ACTIVE = 'ACTIVE',
///     INACTIVE = 'INACTIVE',
/// }
/// ```
fn render_as_ts_enum(
    ctx: &mut GeneratorContext,
    enum_name: &str,
    enum_type: &Node<EnumType>,
) -> Result<()> {
    let export = get_export_kw(ctx);

    let const_kw = if ctx.options.const_enums {
        "const "
    } else {
        ""
    };

    writeln!(ctx.writer, "{export}{const_kw}enum {enum_name} {{")?;

    for value in enum_type.values.keys() {
        let transformed = transform_enum_value(value.as_str(), ctx.options);
        // Enum member name is transformed, value stays original
        writeln!(ctx.writer, "  {transformed} = '{value}',")?;
    }

    writeln!(ctx.writer, "}}")?;
    Ok(())
}

/// Apply enum prefix and suffix to a type name based on user configuration
fn apply_enum_affixes<'a>(type_name: &'a str, options: &PluginOptions) -> Cow<'a, str> {
    match (&options.enum_prefix, &options.enum_suffix) {
        (None, None) => Cow::Borrowed(type_name),
        (prefix, suffix) => {
            let prefix = prefix.as_deref().unwrap_or("");
            let suffix = suffix.as_deref().unwrap_or("");
            Cow::Owned(format!("{prefix}{type_name}{suffix}"))
        }
    }
}

/// Apply naming convention to an enum value
fn transform_enum_value<'a>(value: &'a str, options: &PluginOptions) -> Cow<'a, str> {
    let (case, transform_underscore) = match &options.naming_convention {
        None => (NamingCase::Keep, false),
        Some(NamingConvention::Simple(case)) => (*case, false),
        Some(NamingConvention::Detailed(config)) => {
            let case = config.enum_values.unwrap_or(NamingCase::Keep);
            (case, config.transform_underscore)
        }
    };

    case.apply(value, transform_underscore)
}
