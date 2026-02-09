use apollo_compiler::Name;
use apollo_compiler::ast::Field;

use crate::Result;
use crate::generators::GeneratorContext;
use crate::generators::common::helpers::indent;
use crate::generators::typescript_operations::selection::NormalizedSelectionSet;

pub(crate) fn render_field(
    ctx: &mut GeneratorContext,
    field: &Field,
    parent_type: &Name,
    normalized: &mut NormalizedSelectionSet,
    depth: usize,
) -> Result<()> {
    indent(ctx, depth)?;

    let response_name = field.alias.as_ref().unwrap_or(&field.name);

    let Ok(field_def) = ctx.schema.type_field(parent_type, &field.name) else {
        return Ok(());
    };

    write!(ctx.writer, "{response_name}: ")?;
    writeln!(ctx.writer, "TODO;")?;

    Ok(())
}
