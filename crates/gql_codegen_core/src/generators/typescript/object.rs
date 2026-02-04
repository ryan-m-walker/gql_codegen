use std::io::Write;

use apollo_compiler::{Node, schema::ObjectType};

use crate::{
    Result,
    generators::{
        GeneratorContext,
        typescript::{
            field::render_field,
            helpers::{
                get_optional_prop_modifier, get_readonly_kw, render_decl_closing,
                render_decl_opening, render_description,
            },
        },
    },
};

pub(crate) fn render_object(
    object: &Node<ObjectType>,
    ctx: &GeneratorContext,
    writer: &mut dyn Write,
) -> Result<()> {
    let readonly = get_readonly_kw(ctx);

    render_description(&object.description, 0, writer)?;
    render_decl_opening(&object.name, ctx, writer)?;

    // TODO: casing!
    let type_name = object.name.as_str();

    if !ctx.options.skip_typename {
        let optional = get_optional_prop_modifier(ctx);
        writeln!(writer, "  {readonly}__typename{optional}: '{type_name}';")?;
    }

    for (field_name, field) in object.fields.iter() {
        render_field(field_name, field, ctx, writer)?;
    }

    render_decl_closing(ctx, writer)?;

    Ok(())
}
