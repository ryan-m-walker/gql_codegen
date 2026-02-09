use apollo_compiler::Name;
use apollo_compiler::ast::OperationDefinition;

use crate::generators::GeneratorContext;
use crate::generators::typescript_operations::fragment::render_fragment;
use crate::generators::typescript_operations::operation::render_operation;
use crate::{ParsedFragment, Result};

mod field;
mod fragment;
mod fragment_spread;
mod inline_fragment;
mod operation;
mod selection;
mod typename;
mod variables;

/// Item to generate - either a fragment or operation
enum GenerateItem<'a> {
    Fragment(&'a Name, &'a ParsedFragment<'a>),
    Operation(&'a Name, &'a OperationDefinition),
}

pub fn generate_typescript_operations(ctx: &mut GeneratorContext) -> Result<()> {
    let mut items: Vec<GenerateItem> =
        Vec::with_capacity(ctx.fragments.len() + ctx.operations.len());

    for (name, fragment) in ctx.fragments.iter() {
        items.push(GenerateItem::Fragment(name, fragment));
    }
    for (name, operation) in ctx.operations.iter() {
        items.push(GenerateItem::Operation(name, &operation.definition));
    }

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
