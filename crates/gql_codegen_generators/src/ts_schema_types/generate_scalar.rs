use std::io::{Result, Write};

use apollo_compiler::{Node, schema::ScalarType};

use super::TsSchemaTypesGeneratorConfig;

pub(crate) fn generate_scalar<T: Write>(
    writer: &mut T,
    node: &Node<ScalarType>,
    config: &TsSchemaTypesGeneratorConfig,
) -> Result<()> {
    if let Some(scalars) = &config.scalars {
        if let Some(custom_type) = scalars.get(node.name.as_str()) {
            return writeln!(writer, "export type {} = {};", node.name, custom_type);
        }
    }

    writeln!(writer, "export type {} = unknown;", node.name)
}
