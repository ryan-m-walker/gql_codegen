use crate::Result;
use crate::generators::GeneratorContext;
use crate::generators::common::helpers::get_readonly_kw;

pub(crate) fn render_typename(ctx: &mut GeneratorContext, type_name: &str) -> Result<()> {
    if ctx.options.skip_typename {
        return Ok(());
    }

    let readonly = get_readonly_kw(ctx);
    writeln!(ctx.writer, "  {readonly}__typename?: '{type_name}';")?;

    Ok(())
}
