use crate::Result;
use crate::config::TypenamePolicy;
use crate::generators::GeneratorContext;
use crate::generators::common::helpers::get_readonly_kw;

pub(crate) fn render_typename(ctx: &mut GeneratorContext, type_name: &str) -> Result<()> {
    match ctx.options.resolved_typename_policy() {
        TypenamePolicy::Skip | TypenamePolicy::AsSelected => return Ok(()),
        TypenamePolicy::Always => {}
    }

    let readonly = get_readonly_kw(ctx);
    // TODO: optional ? or not
    writeln!(ctx.writer, "  {readonly}__typename: '{type_name}';")?;

    Ok(())
}
