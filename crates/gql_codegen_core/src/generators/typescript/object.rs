use apollo_compiler::Node;
use apollo_compiler::schema::ObjectType;

use crate::Result;
use crate::config::NamingCase;
use crate::generators::GeneratorContext;
use crate::generators::common::helpers::{FieldType, render_decl_closing};
use crate::generators::common::typename::render_typename;
use crate::generators::typescript::field::render_field;
use crate::generators::common::helpers::render_decl_opening;
use crate::generators::typescript::helpers::render_description;

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
    if ctx.options.only_enums {
        return Ok(());
    }

    let raw_name = object.name.as_str();
    let type_name = ctx.transform_type_name(raw_name);

    render_description(ctx, &object.description, 0)?;
    render_decl_opening(ctx, &type_name, Some(&object.implements_interfaces))?;
    // __typename uses the raw schema name, not the cased one
    render_typename(ctx, raw_name)?;

    for (field_name, field) in object.fields.iter() {
        render_field(ctx, field_name, &FieldType::Object(field))?;
    }

    render_decl_closing(ctx)?;
    writeln!(ctx.writer)?;

    for (field_name, field) in object.fields.iter() {
        if field.arguments.is_empty() {
            continue;
        }

        // PascalCase the field name, compose, then apply typeNames casing
        let pascal_field = NamingCase::PascalCase.apply(field_name.as_str(), true);
        let composed = format!("{raw_name}{pascal_field}Args");
        let args_name = ctx.transform_type_name(&composed);
        render_decl_opening(ctx, &args_name, None)?;

        for field in field.arguments.iter() {
            render_field(ctx, &field.name, &FieldType::InputObject(field.as_ref()))?;
        }

        render_decl_closing(ctx)?;
        writeln!(ctx.writer)?;
    }

    Ok(())
}
