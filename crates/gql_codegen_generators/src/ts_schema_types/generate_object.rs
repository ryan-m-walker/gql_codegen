use std::io::{Result, Write};

use apollo_compiler::{Node, schema::ObjectType};

use super::{
    TsSchemaTypesGeneratorConfig,
    common::{render_description, render_type},
};

pub(crate) fn generate_object<T: Write>(
    writer: &mut T,
    node: &Node<ObjectType>,
    config: &TsSchemaTypesGeneratorConfig,
) -> Result<()> {
    let readonly = config.readonly.unwrap_or(false);

    writeln!(writer)?;

    render_description(writer, &node.description, "")?;

    write!(writer, "export interface {}", node.name)?;

    let interfaces_count = node.implements_interfaces.len();

    if interfaces_count > 0 {
        write!(writer, " extends")?;
    }

    for (i, interface) in node.implements_interfaces.iter().enumerate() {
        write!(writer, " {interface}")?;

        if i < interfaces_count - 1 {
            write!(writer, ",")?;
        }
    }

    writeln!(writer, " {{")?;

    write!(writer, "  ")?;
    if readonly {
        write!(writer, "readonly ")?;
    }
    writeln!(writer, "__typename: \"{}\";", node.name)?;

    for (name, field) in &node.fields {
        if field.description.is_some() {
            render_description(writer, &field.description, "  ")?;
        }

        write!(writer, "  ")?;

        if readonly {
            write!(writer, "readonly ")?;
        }

        writeln!(writer, "{}: {};", name, render_type(&field.ty))?;
    }

    writeln!(writer, "}}")?;

    Ok(())
}
