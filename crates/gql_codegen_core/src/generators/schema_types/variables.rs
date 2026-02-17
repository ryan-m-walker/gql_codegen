use apollo_compiler::schema::ObjectType;

use crate::generators::GeneratorContext;
use crate::generators::common::helpers::{FieldType, render_decl_closing, render_decl_opening};
use crate::generators::schema_types::field::render_field;
use crate::{NamingCase, Result};

pub fn render_variables(ctx: &mut GeneratorContext, name: &str, object: &ObjectType) -> Result<()> {
    for (field_name, field) in object.fields.iter() {
        if field.arguments.is_empty() {
            continue;
        }

        // PascalCase the field name, compose, then apply typeNames casing
        let pascal_field = NamingCase::PascalCase.apply(field_name.as_str(), true);
        let composed = format!("{name}{pascal_field}Args");
        let args_name = ctx.transform_type_name(&composed);
        render_decl_opening(ctx, &args_name, None)?;

        for field in field.arguments.iter() {
            render_field(ctx, &field.name, &FieldType::InputObject(field.as_ref()))?;
        }

        render_decl_closing(ctx)?;
        writeln!(ctx.writer)?;
    }

    Ok(())
}
