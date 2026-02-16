use apollo_compiler::ast::Type;

use crate::Result;
use crate::generators::GeneratorContext;
use crate::generators::common::helpers::{
    ScalarDirection, get_array_type, get_readonly_kw, indent, render_type,
};
use crate::generators::typescript_operations::selection::{
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
            write!(ctx.writer, "| null | undefined")?;
        }

        writeln!(ctx.writer)?;
        indent(ctx, depth)?;

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
            write!(ctx.writer, " | null | undefined")?;
        }

        render_list_closing(ctx, &field.field_type)?;
        writeln!(ctx.writer, ";")?;
        return Ok(());
    }

    // scalar type rendering
    let ts_type = render_type(ctx, &field.field_type, ScalarDirection::Output);
    writeln!(ctx.writer, "{ts_type};")?;
    Ok(())
}

/// Recursively write `Array<` (or `ReadonlyArray<`) for each list layer.
fn render_list_opening(ctx: &mut GeneratorContext, ty: &Type) -> Result<()> {
    if let Type::List(inner) | Type::NonNullList(inner) = ty {
        let array_type = get_array_type(ctx);
        write!(ctx.writer, "{array_type}<")?;
        render_list_opening(ctx, inner)?;
    }
    Ok(())
}

/// Recursively close list layers from inside out, appending `| null` for nullable layers.
fn render_list_closing(ctx: &mut GeneratorContext, ty: &Type) -> Result<()> {
    if let Type::List(inner) | Type::NonNullList(inner) = ty {
        render_list_closing(ctx, inner)?;
        write!(ctx.writer, ">")?;
        if matches!(ty, Type::List(_)) {
            write!(ctx.writer, " | null | undefined")?;
        }
    }
    Ok(())
}

/// Unwrap all list layers to get the innermost element type.
fn inner_element_type(ty: &Type) -> &Type {
    match ty {
        Type::List(inner) | Type::NonNullList(inner) => inner_element_type(inner),
        _ => ty,
    }
}

fn get_optional_prop_modifier(field: &NormalizedSelection) -> &'static str {
    // TODO: null type?
    let is_nullable = !field.field_type.is_non_null() || field.has_conditional;
    if is_nullable { "?" } else { "" }
}
