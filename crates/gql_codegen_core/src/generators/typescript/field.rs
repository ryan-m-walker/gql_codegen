use apollo_compiler::Name;
use apollo_compiler::schema::Type;

use crate::Result;
use crate::generators::GeneratorContext;
use crate::generators::common::helpers::{
    FieldType, get_array_type, get_optional_prop_modifier, get_readonly_kw, render_field_type,
    render_type, wrap_maybe,
};
use crate::generators::typescript::helpers::render_description;

pub(crate) fn render_field(
    ctx: &mut GeneratorContext,
    name: &Name,
    field_type: &FieldType,
) -> Result<()> {
    let readonly = get_readonly_kw(ctx);
    let optional_field = get_optional_prop_modifier(field_type);
    let array_type = get_array_type(ctx);
    let dir = field_type.direction();

    let (description, ty) = match field_type {
        FieldType::InputObject(field) => (&field.description, field.ty.as_ref()),
        FieldType::Object(field) => (&field.description, &field.ty),
    };

    render_description(ctx, description, 1)?;
    write!(ctx.writer, "  {readonly}{name}")?;

    match &ty {
        Type::Named(name) => {
            let field = render_field_type(ctx, name, dir);
            let maybe_value = wrap_maybe(field.as_ref());
            writeln!(ctx.writer, "{optional_field}: {maybe_value};")?;
        }
        Type::NonNullNamed(name) => {
            let field_type = render_field_type(ctx, name, dir);
            writeln!(ctx.writer, "{optional_field}: {field_type};")?;
        }
        Type::List(inner) => {
            let inner_type = render_type(ctx, inner.as_ref(), dir);
            let array = wrap_maybe(&format!("{array_type}<{inner_type}>"));
            writeln!(ctx.writer, "{optional_field}: {array};")?;
        }
        Type::NonNullList(inner) => {
            let inner_type = render_type(ctx, inner.as_ref(), dir);
            writeln!(ctx.writer, "{optional_field}: {array_type}<{inner_type}>;")?;
        }
    }

    Ok(())
}
