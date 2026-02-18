use apollo_compiler::schema::UnionType;
use apollo_compiler::{Name, Node};

use crate::Result;
use crate::generators::GeneratorContext;
use crate::generators::common::helpers::{get_export_kw, get_readonly_kw};

pub(crate) fn render_union(
    ctx: &mut GeneratorContext,
    name: &Name,
    union: &Node<UnionType>,
) -> Result<()> {
    let export = get_export_kw(ctx);
    let readonly = get_readonly_kw(ctx);
    let type_name = ctx.transform_type_name(name.as_str());

    writeln!(ctx.writer, "{export}type {type_name} = ")?;

    for (i, member) in union.members.iter().enumerate() {
        let member_type_name = ctx.transform_type_name(member.name.as_str());

        write!(ctx.writer, "  | {member_type_name}",)?;

        if i == union.members.len() - 1 && !ctx.options.future_proof_unions() {
            write!(ctx.writer, ";")?;
        }

        writeln!(ctx.writer)?;
    }

    if ctx.options.future_proof_unions() {
        writeln!(ctx.writer, "  | {{ {readonly}__typename?: '%other' }};")?;
    }

    writeln!(ctx.writer)?;

    Ok(())
}
