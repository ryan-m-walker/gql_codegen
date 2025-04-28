use std::io::{Result, Write};

use apollo_compiler::{Node, schema::InterfaceType};

use super::{TsSchemaTypesGeneratorConfig, common::render_type};

pub(crate) fn generate_interface<T: Write>(
    writer: &mut T,
    node: &Node<InterfaceType>,
    config: &TsSchemaTypesGeneratorConfig,
) -> Result<()> {
    let readonly = config.readonly.unwrap_or(false);

    writeln!(writer, "\nexport interface {} {{", node.name)?;

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
