use apollo_compiler::Name;

use crate::Result;
use crate::generators::GeneratorContext;
use crate::generators::common::helpers::{get_readonly_kw, indent};

pub(crate) fn render_op_typename(
    ctx: &mut GeneratorContext,
    response_name: &str,
    parent_type: &Name,
    depth: usize,
) -> Result<bool> {
    let policy = ctx.options.resolved_typename_policy();

    // TODO: optional ? or not

    let readonly = get_readonly_kw(ctx);

    indent(ctx, depth)?;
    writeln!(ctx.writer, "{readonly}{response_name}: '{parent_type}';",)?;

    Ok(true)
}
