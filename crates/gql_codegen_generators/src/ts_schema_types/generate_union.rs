use std::io::{Result, Write};

use apollo_compiler::{Node, schema::UnionType};

use super::TsSchemaTypesGeneratorConfig;

pub(crate) fn generate_union<T: Write>(
    writer: &mut T,
    node: &Node<UnionType>,
    _config: &TsSchemaTypesGeneratorConfig,
) -> Result<()> {
    write!(writer, "\nexport type {} = ", node.name)?;

    let members_count = node.members.len();

    for (i, value) in node.members.iter().enumerate() {
        write!(writer, " {}", value.name)?;

        if i < members_count - 1 {
            write!(writer, " |")?;
        }
    }

    writeln!(writer, ";")?;

    Ok(())
}
