use apollo_compiler::ast::Type;

use crate::Result;
use crate::generators::GeneratorContext;
use crate::generators::common::helpers::{
    NullableLocation, get_array_type, render_nullable_closing,
};

pub(crate) fn render_list_opening(ctx: &mut GeneratorContext, ty: &Type) -> Result<()> {
    if let Type::List(inner) | Type::NonNullList(inner) = ty {
        let array_type = get_array_type(ctx);
        write!(ctx.writer, "{array_type}<")?;
        render_list_opening(ctx, inner)?;
    }

    Ok(())
}

/// Recursively close list layers from inside out, appending `| null` for nullable layers.
pub(crate) fn render_list_closing(ctx: &mut GeneratorContext, ty: &Type) -> Result<()> {
    if let Type::List(inner) | Type::NonNullList(inner) = ty {
        render_list_closing(ctx, inner)?;
        write!(ctx.writer, ">")?;

        if matches!(ty, Type::List(_)) {
            render_nullable_closing(ctx, NullableLocation::List)?;
        }
    }

    Ok(())
}
