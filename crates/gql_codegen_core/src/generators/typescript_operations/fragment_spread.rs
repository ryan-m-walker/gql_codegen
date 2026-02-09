use apollo_compiler::Name;
use apollo_compiler::ast::FragmentSpread;

use crate::Result;
use crate::generators::GeneratorContext;
use crate::generators::typescript_operations::selection::NormalizedSelectionSet;

pub(crate) fn render_fragment_spread(
    ctx: &mut GeneratorContext,
    spread: &FragmentSpread,
    parent_type: &Name,
    normalized: &mut NormalizedSelectionSet,
    depth: usize,
) -> Result<()> {
    Ok(())
}
