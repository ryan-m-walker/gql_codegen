use apollo_compiler::Name;

use crate::generators::GeneratorContext;
use crate::generators::common::helpers::{get_readonly_kw, indent};
use crate::{Result, TypenamePolicy};

pub(crate) fn render_op_typename(
    ctx: &mut GeneratorContext,
    response_name: &str,
    parent_type: &Name,
    depth: usize,
) -> Result<bool> {
    let policy = ctx.options.resolved_typename_policy();

    let optional = match policy {
        TypenamePolicy::AsSelected => "",
        TypenamePolicy::Skip => return Ok(false),
        TypenamePolicy::Always => {
            if ctx.options.non_optional_typename {
                ""
            } else {
                "?"
            }
        }
    };

    let readonly = get_readonly_kw(ctx);

    indent(ctx, depth)?;
    writeln!(
        ctx.writer,
        "{readonly}{response_name}{optional}: '{parent_type}';",
    )?;

    Ok(true)
}

