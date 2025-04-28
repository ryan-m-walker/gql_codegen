use std::io::{Result, Write};

use apollo_compiler::{
    Node,
    schema::{InputObjectType, ObjectType},
};

use super::{TsSchemaTypesGeneratorConfig, common::render_type};

pub(crate) fn generate_input_object<T: Write>(
    writer: &mut T,
    node: &Node<InputObjectType>,
    config: &TsSchemaTypesGeneratorConfig,
) -> Result<()> {
    let readonly = config.readonly.unwrap_or(false);

    writeln!(writer, "\nexport interface {} {{", node.name)?;

    write!(writer, "  ")?;
    if readonly {
        write!(writer, "readonly ")?;
    }
    writeln!(writer, "__typename: \"{}\";", node.name)?;

    for (name, field) in &node.fields {
        write!(writer, "  ")?;

        if readonly {
            write!(writer, "readonly ")?;
        }

        writeln!(writer, "{}: {};", name, render_type(&field.ty))?;
    }

    writeln!(writer, "}}")?;

    Ok(())
}
