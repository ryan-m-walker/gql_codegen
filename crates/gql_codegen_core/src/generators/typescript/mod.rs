//! TypeScript schema types generator
//!
//! Generates TypeScript types for GraphQL schema types (objects, interfaces,
//! enums, unions, input objects).

use apollo_compiler::schema::ExtendedType;

use super::GeneratorContext;
use crate::Result;
use crate::generators::typescript::r#enum::render_enum;
use crate::generators::typescript::input::render_input;
use crate::generators::typescript::interface::render_interface;
use crate::generators::typescript::object::render_object;
use crate::generators::typescript::operation_types::collect_operation_types;
use crate::generators::typescript::scalar::render_scalar;
use crate::generators::typescript::union::render_union;

mod r#enum;
mod field;
mod helpers;
mod input;
mod interface;
mod object;
mod operation_types;
mod scalar;
mod union;

/// Main entry point for the TypeScript generator.
/// Generates TypeScript types from the GraphQL schema.
pub fn generate_typescript(ctx: &mut GeneratorContext) -> Result<()> {
    // TODO: only usage op types
    let referenced_types = if ctx.options.only_referenced_types {
        Some(collect_operation_types(ctx))
    } else {
        None
    };

    // Sort for deterministic output
    let mut type_names: Vec<_> = ctx.schema.types.keys().collect();
    type_names.sort();

    for name in type_names {
        // Skip built-in types
        // TODO: relay types? like __id? maybe a config option?
        if name.as_str().starts_with("__") {
            continue;
        }

        // Skip types not referenced in operations (if only_operation_types is enabled)
        if let Some(ref referenced) = referenced_types
            && referenced.contains(name.as_str())
        {
            continue;
        }

        let Some(ty) = &ctx.schema.types.get(name) else {
            // TODO: warn?
            continue;
        };

        match ty {
            ExtendedType::Object(obj) => render_object(ctx, obj)?,
            ExtendedType::Enum(en) => render_enum(ctx, en)?,
            ExtendedType::Interface(iface) => render_interface(ctx, iface)?,
            ExtendedType::Union(union) => render_union(ctx, name, union)?,
            ExtendedType::InputObject(input) => render_input(ctx, input)?,
            ExtendedType::Scalar(scalar) => render_scalar(ctx, scalar)?,
        }
    }

    Ok(())
}
