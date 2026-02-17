use apollo_compiler::Node;
use apollo_compiler::schema::InputObjectType;

use crate::Result;
use crate::generators::GeneratorContext;
use crate::generators::common::helpers::{FieldType, render_decl_closing, render_decl_opening};
use crate::generators::schema_types::field::render_field;
use crate::generators::schema_types::helpers::render_description;

/// Render a GraphQL input object type as TypeScript type to the current writer.
///
/// **Example Input:**
/// ``` graphql
/// input UserInput {
///   name: String!
///   email: String!
/// }
/// ```
///
/// **Output:**
/// ``` typescript
/// interface UserInput {
///   name: string;
///   email: string;
/// }
/// ```
pub(crate) fn render_input(
    ctx: &mut GeneratorContext,
    input: &Node<InputObjectType>,
) -> Result<()> {
    let type_name = ctx.transform_type_name(input.name.as_str());

    render_description(ctx, &input.description, 0)?;
    render_decl_opening(ctx, &type_name, None)?;

    for (field_name, field) in input.fields.iter() {
        render_field(ctx, field_name, &FieldType::InputObject(field.as_ref()))?;
    }

    render_decl_closing(ctx)?;
    writeln!(ctx.writer)?;

    Ok(())
}
