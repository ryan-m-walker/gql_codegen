use apollo_compiler::Name;
use apollo_compiler::ast::OperationDefinition;

use crate::Result;
use crate::generators::GeneratorContext;
use crate::generators::common::helpers::{render_decl_closing, render_decl_prefix};
use crate::generators::operation_types::selection::{
    NormalizedSelectionSet, collect_selection_set, render_normalized,
};
use crate::generators::operation_types::variables::render_variables;

/// Render a GraphQL operations (`query`, `mutation` or `subscription`) as a TypeScript type
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

    let name = format!("{name}{root_type}");
    let name = ctx.transform_type_name(&name);

    render_decl_prefix(ctx, &name, None)?;
    writeln!(ctx.writer, "{{")?;
    render_normalized(ctx, &normalized, 0)?;
    render_decl_closing(ctx)?;
    writeln!(ctx.writer)?;

    render_variables(ctx, &name, operation)?;

    Ok(())
}
