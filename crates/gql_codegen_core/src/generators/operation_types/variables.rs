use apollo_compiler::ast::OperationDefinition;

use crate::Result;
use crate::generators::GeneratorContext;
use crate::generators::common::helpers::{
    FieldType, ScalarDirection, get_optional_prop_modifier, get_readonly_kw, indent,
    render_decl_closing, render_decl_opening, render_type,
};

/// Renders variable for operations as a distinct type.
///
/// **Example:**
///
/// ``` typescript
/// export interface GetUserQueryVariables {
///   readonly id: string;
/// }
/// ```
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
        let optional = get_optional_prop_modifier(&FieldType::Variable(var));
        let name = var.name.as_str();

        indent(ctx, 1)?;
        write!(ctx.writer, "{readonly}{name}{optional}: ")?;
        render_type(ctx, &var.ty, ScalarDirection::Input)?;
        writeln!(ctx.writer, ";")?;
    }

    render_decl_closing(ctx)?;
    writeln!(ctx.writer)?;

    Ok(())
}
