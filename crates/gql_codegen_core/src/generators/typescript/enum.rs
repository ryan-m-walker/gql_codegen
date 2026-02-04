use std::borrow::Cow;
use std::io::Write;

use apollo_compiler::{Node, schema::EnumType};

use crate::config::{NamingCase, NamingConvention};
use crate::{
    Result,
    config::PluginOptions,
    generators::{GeneratorContext, typescript::helpers::get_export_kw},
};

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
pub(crate) fn render_enum(
    enum_type: &Node<EnumType>,
    ctx: &GeneratorContext,
    writer: &mut dyn Write,
) -> Result<()> {
    let enum_name = apply_enum_affixes(enum_type.name.as_str(), ctx.options);

    if ctx.options.enums_as_types {
        render_as_type_union(&enum_name, enum_type, ctx, writer)?;
    } else {
        render_as_ts_enum(&enum_name, enum_type, ctx, writer)?;
    }

    writeln!(writer)?;
    Ok(())
}

/// Render enum as TypeScript type union.
///
/// **Example:**
/// ``` typescript
/// type Status = 'ACTIVE' | 'INACTIVE';
/// ```
fn render_as_type_union(
    enum_name: &str,
    enum_type: &Node<EnumType>,
    ctx: &GeneratorContext,
    writer: &mut dyn Write,
) -> Result<()> {
    let export = get_export_kw(ctx);

    write!(writer, "{export}type {enum_name} =")?;

    let values: Vec<_> = enum_type.values.keys().collect();
    for (i, value) in values.iter().enumerate() {
        let transformed = transform_enum_value(value.as_str(), ctx.options);
        if i == 0 {
            write!(writer, " '{transformed}'")?;
        } else {
            write!(writer, " | '{transformed}'")?;
        }
    }

    if ctx.options.future_proof_enums {
        write!(writer, " | '%future added value'")?;
    }

    writeln!(writer, ";")?;
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
    enum_name: &str,
    enum_type: &Node<EnumType>,
    ctx: &GeneratorContext,
    writer: &mut dyn Write,
) -> Result<()> {
    let export = get_export_kw(ctx);

    let const_kw = if ctx.options.const_enums {
        "const "
    } else {
        ""
    };

    writeln!(writer, "{export}{const_kw}enum {enum_name} {{")?;

    for value in enum_type.values.keys() {
        let transformed = transform_enum_value(value.as_str(), ctx.options);
        // Enum member name is transformed, value stays original
        writeln!(writer, "  {transformed} = '{value}',")?;
    }

    writeln!(writer, "}}")?;
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
