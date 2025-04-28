use std::io::{Result, Write};

use super::TsSchemaTypesGeneratorConfig;

struct Scalar {
    name: &'static str,
    default_type: &'static str,
}

const DEFAULT_SCALARS: [Scalar; 5] = [
    Scalar {
        name: "ID",
        default_type: "string",
    },
    // TODO: handle this better
    Scalar {
        name: "String",
        default_type: "string",
    },
    Scalar {
        name: "Int",
        default_type: "number",
    },
    Scalar {
        name: "Float",
        default_type: "number",
    },
    Scalar {
        name: "Boolean",
        default_type: "boolean",
    },
];

pub(crate) fn generate_default_scalars<T: Write>(
    writer: &mut T,
    config: &TsSchemaTypesGeneratorConfig,
) -> Result<()> {
    for Scalar { name, default_type } in DEFAULT_SCALARS {
        if let Some(scalars) = &config.scalars {
            if let Some(custom_type) = scalars.get(name) {
                writeln!(writer, "export type {name} = {custom_type};",).unwrap();
                continue;
            }
        }

        writeln!(writer, "export type {name} = {default_type};").unwrap();
    }

    Ok(())
}
