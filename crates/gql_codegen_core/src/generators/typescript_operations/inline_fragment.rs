use apollo_compiler::Name;
use apollo_compiler::ast::InlineFragment;

use crate::Result;
use crate::generators::GeneratorContext;
use crate::generators::typescript_operations::selection::NormalizedSelectionSet;

pub(crate) fn render_inline_fragment(
    ctx: &mut GeneratorContext,
    inline: &InlineFragment,
    parent_type: &Name,
    normalized: &mut NormalizedSelectionSet,
    depth: usize,
) -> Result<()> {
    Ok(())
}
