use apollo_compiler::Name;

use crate::generators::GeneratorContext;
use crate::generators::common::helpers::render_decl_prefix;
use crate::generators::operation_types::selection::{
    NormalizedSelectionSet, collect_selection_set, render_normalized,
};
use crate::{ParsedFragment, Result};

pub(crate) fn render_fragment<'a>(
    ctx: &mut GeneratorContext,
    name: &Name,
    fragment: &ParsedFragment<'a>,
) -> Result<()> {
    let condition = &fragment.definition.type_condition;

    let mut normalized = NormalizedSelectionSet::new();

    collect_selection_set(
        ctx,
        &fragment.definition.selection_set,
        condition,
        &mut normalized,
    )?;

    let name = format!("{name}Fragment");
    let name = ctx.transform_type_name(&name);

    render_decl_prefix(ctx, &name, None)?;
    writeln!(ctx.writer, "{{")?;

    render_normalized(ctx, &normalized, 0)?;

    writeln!(ctx.writer, "}}")?;

    Ok(())
}
