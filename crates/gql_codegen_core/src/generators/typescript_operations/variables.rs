use apollo_compiler::ast::OperationDefinition;

use crate::Result;
use crate::generators::GeneratorContext;
use crate::generators::common::helpers::{
    ScalarDirection, get_readonly_kw, indent, render_decl_closing, render_decl_opening, render_type,
};

pub(crate) fn render_variables(
    ctx: &mut GeneratorContext,
    op_name: &str,
    operation: &OperationDefinition,
) -> Result<()> {
    if operation.variables.is_empty() {
        return Ok(());
    }

    let readonly = get_readonly_kw(ctx);
    let raw_name = format!("{op_name}Variables");
    let name = ctx.transform_type_name(&raw_name);

    render_decl_opening(ctx, &name, None)?;

    for var in &operation.variables {
        indent(ctx, 1)?;
        let var_type = render_type(ctx, &var.ty, ScalarDirection::Input);
        let name = var.name.as_str();
        writeln!(ctx.writer, "{readonly}{name}: {var_type};")?;
    }

    render_decl_closing(ctx)?;
    writeln!(ctx.writer)?;

    Ok(())
}
