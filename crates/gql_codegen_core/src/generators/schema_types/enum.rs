use std::borrow::Cow;

use apollo_compiler::Node;
use apollo_compiler::schema::EnumType;

use crate::Result;
use crate::config::{GeneratorOptions, NamingCase, NamingConvention};
use crate::generators::GeneratorContext;
use crate::generators::common::helpers::get_export_kw;
use crate::generators::schema_types::helpers::render_description;

// TODO: js lib is not persisting enum key for ts enums
// TODO: maybe move this to common since it's shared in operation_types

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
    let name = ctx.transform_type_name(enum_type.name.as_str());

    // TODO: maybe as const enum is ok?
    render_as_type_union(ctx, &name, enum_type)?;

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

    let len = enum_type.values.len();

    for (i, value) in enum_type.values.values().enumerate() {
        let transformed = transform_enum_value(value.value.as_str(), ctx.options);

        let semi = if i == len - 1 && !ctx.options.future_proof_enums() {
            ";"
        } else {
            ""
        };

        render_description(ctx, &value.description, 1)?;
        writeln!(ctx.writer, "  | '{transformed}'{semi}")?;
    }

    if ctx.options.future_proof_enums() {
        writeln!(ctx.writer, "  | '%future added value';")?;
    }

    Ok(())
}

/// Render enum as a `const` object with a derived type.
///
/// **Example:**
/// ``` typescript
/// export const Status = {
///   ACTIVE: 'ACTIVE',
///   INACTIVE: 'INACTIVE',
/// } as const;
///
/// export type Status = typeof Status[keyof typeof Status];
/// ```
fn _render_as_const_object(
    ctx: &mut GeneratorContext,
    enum_name: &str,
    enum_type: &Node<EnumType>,
) -> Result<()> {
    let export = get_export_kw(ctx);

    writeln!(ctx.writer, "{export}const {enum_name} = {{")?;

    for key in enum_type.values.keys() {
        let transformed = transform_enum_value(key.as_str(), ctx.options);
        writeln!(ctx.writer, "  {transformed}: '{key}',")?;
    }

    writeln!(ctx.writer, "}} as const;")?;
    writeln!(ctx.writer)?;
    writeln!(
        ctx.writer,
        "{export}type {enum_name} = typeof {enum_name}[keyof typeof {enum_name}];"
    )?;

    Ok(())
}

/// Apply naming convention to an enum value
fn transform_enum_value<'a>(value: &'a str, options: &GeneratorOptions) -> Cow<'a, str> {
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
