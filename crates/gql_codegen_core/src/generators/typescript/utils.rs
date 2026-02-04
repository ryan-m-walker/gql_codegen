use std::io::Write;

use crate::Result;
use crate::{DeclarationKind, generators::GeneratorContext};

pub(crate) fn get_export_kw(ctx: &GeneratorContext) -> &'static str {
    if ctx.options.no_export { "" } else { "export" }
}

pub(crate) fn get_decl_kind_kw(ctx: &GeneratorContext) -> &'static str {
    match ctx.options.declaration_kind {
        Some(DeclarationKind::Type) | None => "type",
        Some(DeclarationKind::Interface) => "interface",
        Some(DeclarationKind::Class) => "class",
        Some(DeclarationKind::AbstractClass) => "abstract class",
    }
}

pub(crate) fn generate_decl_opening(
    name: &str,
    ctx: &GeneratorContext,
    writer: &mut dyn Write,
) -> Result<()> {
    let export = get_export_kw(ctx);
    let decl_kind = get_decl_kind_kw(ctx);

    let separator = match ctx.options.declaration_kind {
        Some(DeclarationKind::Type) | None => " = ",
        _ => " ",
    };

    writeln!(writer, "{export} {decl_kind} {name}{separator}{{")?;
    Ok(())
}

pub(crate) fn generate_decl_closing(ctx: &GeneratorContext, writer: &mut dyn Write) -> Result<()> {
    let semi = match ctx.options.declaration_kind {
        Some(DeclarationKind::Type) | None => ";",
        _ => "",
    };

    writeln!(writer, "}}{semi}")?;
    Ok(())
}
