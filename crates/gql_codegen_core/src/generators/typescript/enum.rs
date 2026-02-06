use std::borrow::Cow;

use apollo_compiler::Node;
use apollo_compiler::schema::EnumType;

use crate::Result;
use crate::config::{NamingCase, NamingConvention, PluginOptions};
use crate::generators::GeneratorContext;
use crate::generators::typescript::helpers::{get_export_kw, render_description};

/// TODO: js lib is not persisting enum key for ts enums

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

    if ctx.options.enums_as_types.unwrap_or(true) {
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

    writeln!(ctx.writer, "{export}type {enum_name} =")?;

    let values: Vec<_> = enum_type.values.keys().collect();
    for (i, value) in values.iter().enumerate() {
        let transformed = transform_enum_value(value.as_str(), ctx.options);

        let semi = if i == values.len() - 1 && !ctx.options.future_proof_enums {
            ";"
        } else {
            ""
        };

        writeln!(ctx.writer, "  | '{transformed}'{semi}")?;
    }

    if ctx.options.future_proof_enums {
        writeln!(ctx.writer, "  | '%future added value';")?;
    }

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

    // TODO: warn of config conflict
    let const_kw = if ctx.options.const_enums && !ctx.options.numeric_enums {
        "const "
    } else {
        ""
    };

    writeln!(ctx.writer, "{export}{const_kw}enum {enum_name} {{")?;

    for (i, key) in enum_type.values.keys().enumerate() {
        let transformed = transform_enum_value(key.as_str(), ctx.options);

        let value = if ctx.options.numeric_enums {
            i.to_string()
        } else {
            format!("'{key}'")
        };

        // Enum member name is transformed, value stays original
        writeln!(ctx.writer, "  {transformed} = {value},")?;
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
