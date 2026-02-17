use apollo_compiler::ast::Type;

use crate::Result;
use crate::generators::GeneratorContext;
use crate::generators::common::helpers::{
    NullableLocation, ScalarDirection, get_array_type, get_readonly_kw, indent,
    render_nullable_closing, render_type,
};
use crate::generators::common::list::{render_list_closing, render_list_opening};
use crate::generators::operation_types::selection::{
    NormalizedSelection, render_normalized, render_variants,
};

pub(crate) fn render_field(
    ctx: &mut GeneratorContext,
    response_type: &str,
    field: &NormalizedSelection,
    depth: usize,
) -> Result<()> {
    let readonly = get_readonly_kw(ctx);
    let optional = get_optional_prop_modifier(field);

    indent(ctx, depth)?;
    write!(ctx.writer, "{readonly}{response_type}{optional}: ")?;

    // union/interface variant rendering
    if !field.children.variants.is_empty() {
        render_list_opening(ctx, &field.field_type)?;
        writeln!(ctx.writer)?;

        render_variants(ctx, &field.children, depth + 1)?;

        let element = inner_element_type(&field.field_type);
        if !element.is_non_null() {
            writeln!(ctx.writer)?;
            indent(ctx, depth + 1)?;
            render_nullable_closing(ctx, NullableLocation::List)?;
        }

        // indent(ctx, depth)?;

        render_list_closing(ctx, &field.field_type)?;
        writeln!(ctx.writer, ";")?;
        return Ok(());
    }

    // object type rendering
    if !field.children.fields.is_empty() {
        render_list_opening(ctx, &field.field_type)?;
        writeln!(ctx.writer, "{{")?;

        render_normalized(ctx, &field.children, depth)?;

        write!(ctx.writer, "}}")?;

        let element = inner_element_type(&field.field_type);
        if !element.is_non_null() {
            render_nullable_closing(ctx, NullableLocation::List)?;
        }

        render_list_closing(ctx, &field.field_type)?;
        writeln!(ctx.writer, ";")?;
        return Ok(());
    }

    // scalar type rendering
    render_type(ctx, &field.field_type, ScalarDirection::Output)?;
    writeln!(ctx.writer, ";")?;

    Ok(())
}

/// Recursively write `Array<` (or `ReadonlyArray<`) for each list layer.
/// Unwrap all list layers to get the innermost element type.
fn inner_element_type(ty: &Type) -> &Type {
    match ty {
        Type::List(inner) | Type::NonNullList(inner) => inner_element_type(inner),
        _ => ty,
    }
}

fn get_optional_prop_modifier(field: &NormalizedSelection) -> &'static str {
    // TODO: null type option?
    let is_nullable = !field.field_type.is_non_null() || field.has_conditional;
    if is_nullable { "?" } else { "" }
}
