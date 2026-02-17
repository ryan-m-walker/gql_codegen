use apollo_compiler::Name;

use crate::Result;
use crate::generators::GeneratorContext;
use crate::generators::common::helpers::{
    FieldType, get_optional_prop_modifier, get_readonly_kw, render_type,
};
use crate::generators::schema_types::helpers::render_description;

pub(crate) fn render_field(
    ctx: &mut GeneratorContext,
    name: &Name,
    field_type: &FieldType,
) -> Result<()> {
    let readonly = get_readonly_kw(ctx);
    let optional = get_optional_prop_modifier(field_type);
    let dir = field_type.direction();

    let (description, ty) = match field_type {
        FieldType::InputObject(field) => (&field.description, field.ty.as_ref()),
        FieldType::Object(field) => (&field.description, &field.ty),
        FieldType::Variable(field) => (&None, field.ty.as_ref()),
    };

    render_description(ctx, description, 1)?;
    write!(ctx.writer, "  {readonly}{name}{optional}: ")?;

    render_type(ctx, ty, dir)?;
    writeln!(ctx.writer, ";")?;

    Ok(())
}
