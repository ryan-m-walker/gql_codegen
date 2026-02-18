use apollo_compiler::Name;
use apollo_compiler::ast::OperationDefinition;
use apollo_compiler::schema::ExtendedType;

use crate::generators::GeneratorContext;
use crate::generators::operation_types::fragment::render_fragment;
use crate::generators::operation_types::operation::render_operation;
use crate::generators::schema_types::r#enum::render_enum;
use crate::generators::schema_types::union::render_union;
use crate::{ParsedFragment, Result};

mod field;
mod fragment;
mod operation;
mod selection;
mod typename;
mod variables;

/// Item to generate - either a fragment or operation
enum GenerateItem<'a> {
    Fragment(&'a Name, &'a ParsedFragment<'a>),
    Operation(&'a Name, &'a OperationDefinition),
}

/// Generate TypeScript types for GraphQL operations
///
/// **Example**
/// ``` graphql
/// query Users {
///   users {
///     id
///     name
///   }
/// }
/// ```
///
/// ```ts
/// type UsersQuery = {
///   users: Array<{
///     __typename: 'User';
///     id: string;
///     name: string;
///   }>;
/// }
/// ```
pub fn generate_typescript_operations(ctx: &mut GeneratorContext) -> Result<()> {
    let mut items: Vec<GenerateItem> =
        Vec::with_capacity(ctx.fragments.len() + ctx.operations.len());

    let schema_types_generator = ctx.generators.iter().find(|g| g.name() == "schema-types");

    // need to render dependencies if schema types plugin is not available
    if schema_types_generator.is_none() {
        for (name, ty) in &ctx.schema.types {
            if name.starts_with("__") {
                continue;
            }

            match ty {
                ExtendedType::Enum(en) => render_enum(ctx, en)?,
                ExtendedType::Union(union) => render_union(ctx, name, union)?,
                _ => {}
            }
        }
    }

    for (name, fragment) in ctx.fragments.iter() {
        items.push(GenerateItem::Fragment(name, fragment));
    }

    for (name, operation) in ctx.operations.iter() {
        items.push(GenerateItem::Operation(name, &operation.definition));
    }

    // Sort for deterministic output
    items.sort_by_key(|item| match item {
        GenerateItem::Fragment(name, _) => name.as_str(),
        GenerateItem::Operation(name, _) => name.as_str(),
    });

    for item in items {
        match item {
            GenerateItem::Fragment(name, fragment) => {
                render_fragment(ctx, name, fragment)?;
            }
            GenerateItem::Operation(name, operation) => {
                render_operation(ctx, name, operation)?;
            }
        }
    }

    Ok(())
}
