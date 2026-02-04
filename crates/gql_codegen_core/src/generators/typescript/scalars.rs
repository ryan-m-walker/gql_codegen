use std::{collections::HashSet, io::Write};

use apollo_compiler::schema::ExtendedType;

use crate::{
    Error, Result,
    generators::{
        GeneratorContext,
        typescript::utils::{
            generate_decl_closing, generate_decl_opening, get_decl_kind_kw, get_export_kw,
        },
    },
};

const DEFAULT_SCALARS: [(&str, &str); 5] = [
    ("ID", "string"),
    ("String", "string"),
    ("Boolean", "boolean"),
    ("Int", "number"),
    ("Float", "number"),
];

pub(crate) fn generate_scalars(ctx: &GeneratorContext, writer: &mut dyn Write) -> Result<()> {
    writeln!(
        writer,
        "/** All built-in and custom scalars, mapped to their actual values */"
    )?;

    generate_decl_opening("Scalars", ctx, writer)?;

    let mut rendered = HashSet::new();

    for (name, default_type) in DEFAULT_SCALARS {
        let ts_type = if let Some(ref custom_type) = ctx.options.scalars.get(name) {
            custom_type
        } else {
            default_type
        };

        writeln!(
            writer,
            "  {name}: {{ input: {ts_type}; output: {ts_type}; }}"
        )?;

        rendered.insert(name);
    }

    for (_, ty) in ctx.schema.types.iter() {
        if let ExtendedType::Scalar(scalar) = ty {
            let name = scalar.name.as_str();

            if rendered.contains(name) {
                continue;
            }

            let custom_type = ctx.options.scalars.get(name);

            if ctx.options.strict_scalars && custom_type.is_none() {
                return Err(Error::UnknownScalar(name.to_string()));
            }

            let ts_type = if let Some(ref custom_type) = custom_type {
                custom_type
            } else if let Some(ref default_type) = ctx.options.default_scalar_type {
                default_type
            } else {
                "unknown"
            };

            writeln!(
                writer,
                "  {name}: {{ input: {ts_type}; output: {ts_type}; }};"
            )?;

            rendered.insert(name);
        }
    }

    generate_decl_closing(ctx, writer)?;
    writeln!(writer)?;

    Ok(())
}
