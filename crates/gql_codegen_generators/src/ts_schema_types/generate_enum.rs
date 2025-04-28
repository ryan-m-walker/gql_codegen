use std::io::{Result, Write};

use apollo_compiler::{Node, schema::EnumType};

use super::TsSchemaTypesGeneratorConfig;

pub(crate) fn generate_enum<T: Write>(
    writer: &mut T,
    node: &Node<EnumType>,
    config: &TsSchemaTypesGeneratorConfig,
) -> Result<()> {
    let use_native_enums = config.use_native_enums.unwrap_or(false);
    let future_proof_enums = config.future_proof_enums.unwrap_or(false);

    if use_native_enums {
        writeln!(writer, "\nexport enum {} {{", node.name)?;

        for (name, value) in &node.values {
            writeln!(writer, "  {} = \"{}\",", name, value.value)?;
        }

        writeln!(writer, "}}")?;
        return Ok(());
    }

    write!(writer, "\nexport type {} = ", node.name)?;

    let values = node.values.values();
    let values_count = values.len();

    for (i, value) in values.enumerate() {
        write!(writer, " \"{}\"", value.value)?;

        if i < values_count - 1 {
            write!(writer, " |")?;
        }
    }

    if future_proof_enums {
        write!(writer, " | \"%future added value\"")?;
    }

    write!(writer, ";")?;
    writeln!(writer)?;

    Ok(())
}
