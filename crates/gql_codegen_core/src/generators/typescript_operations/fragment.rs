use std::io::Write;

use apollo_compiler::Name;

use crate::generators::GeneratorContext;
use crate::{ParsedFragment, Result};

pub(crate) fn render_fragment<'a>(
    ctx: &mut GeneratorContext,
    name: &Name,
    fragment: &ParsedFragment<'a>,
) -> Result<()> {
    Ok(())
}
