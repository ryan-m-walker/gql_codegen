use apollo_compiler::Node;
use apollo_compiler::schema::InterfaceType;

use crate::Result;
use crate::generators::GeneratorContext;
use crate::generators::common::helpers::{FieldType, render_decl_closing, render_decl_opening};
use crate::generators::schema_types::field::render_field;
use crate::generators::schema_types::helpers::render_description;

/// Render a GraphQL interface type as TypeScript type to the current writer.
///
/// **Example Input:**
/// ``` graphql
/// interface Node {
///   id: ID!
/// }
/// ```
///
/// **Output:**
/// ``` typescript
/// interface Node {
///   id: string;
/// }
///
/// interface User implements Node {
///   id: string;
///   name: string;
/// }
/// ```
pub(crate) fn render_interface(
    ctx: &mut GeneratorContext,
    interface: &Node<InterfaceType>,
) -> Result<()> {
    // TODO: typename prefix and suffix
    let type_name = ctx.transform_type_name(interface.name.as_str());

    render_description(ctx, &interface.description, 0)?;
    render_decl_opening(ctx, &type_name, None)?;

    for (field_name, field) in interface.fields.iter() {
        render_field(ctx, field_name, &FieldType::Object(field))?;
    }

    render_decl_closing(ctx)?;
    writeln!(ctx.writer)?;

    Ok(())
}
