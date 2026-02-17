use apollo_compiler::Node;
use apollo_compiler::schema::ObjectType;

use crate::Result;
use crate::generators::GeneratorContext;
use crate::generators::common::helpers::{FieldType, render_decl_closing, render_decl_opening};
use crate::generators::common::typename::render_typename;
use crate::generators::schema_types::field::render_field;
use crate::generators::schema_types::helpers::render_description;
use crate::generators::schema_types::variables::render_variables;

/// Render a GraphQL object type as TypeScript type to the current writer.
///
/// **Example Input:**
/// ``` graphql
/// type Query {
///   user(id: ID!): User
///   users: [User!]!
/// }
/// ```
///
/// **Output:**
/// ``` typescript
/// export type Query = {
///   __typename?: 'Query';
///   user?: User | null;
///   users: Array<User>;
/// };
///
/// export type QueryUserArgs = {
///   id: string;
/// };
/// ```
pub(crate) fn render_object(ctx: &mut GeneratorContext, object: &Node<ObjectType>) -> Result<()> {
    let raw_name = object.name.as_str();
    let type_name = ctx.transform_type_name(raw_name);

    render_description(ctx, &object.description, 0)?;
    render_decl_opening(ctx, &type_name, Some(&object.implements_interfaces))?;
    render_typename(ctx, raw_name)?;

    for (field_name, field) in object.fields.iter() {
        render_field(ctx, field_name, &FieldType::Object(field))?;
    }

    render_decl_closing(ctx)?;
    writeln!(ctx.writer)?;

    render_variables(ctx, raw_name, object)?;

    Ok(())
}
