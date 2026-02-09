use apollo_compiler::Name;
use apollo_compiler::ast::OperationDefinition;

use crate::Result;
use crate::generators::GeneratorContext;
use crate::generators::common::helpers::get_export_kw;
use crate::generators::typescript_operations::selection::{
    NormalizedSelectionSet, collect_selection_set, render_normalized,
};
use crate::generators::typescript_operations::variables::render_variables;

pub(crate) fn render_operation(
    ctx: &mut GeneratorContext,
    name: &Name,
    operation: &OperationDefinition,
) -> Result<()> {
    let Some(root_type) = ctx.schema.root_operation(operation.operation_type) else {
        return Ok(());
    };

    let mut normalized = NormalizedSelectionSet::new();
    collect_selection_set(ctx, &operation.selection_set, root_type, &mut normalized)?;

    let export = get_export_kw(ctx);
    let name = format!("{name}{root_type}");
    let name = ctx.transform_type_name(&name);

    // TODO: interface options?
    write!(ctx.writer, "{export}type {name} = ")?;
    render_normalized(ctx, &normalized, 0)?;
    writeln!(ctx.writer)?;

    render_variables(ctx, &name, operation)?;

    Ok(())
}
