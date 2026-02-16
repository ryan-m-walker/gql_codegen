//! GraphQL document constants generator
//!
//! Generates TypeScript/JavaScript constants containing GraphQL documents.
//! These can be used with Apollo Client, urql, or other GraphQL clients.

use super::GeneratorContext;
use super::document_transform::{TransformOptions, write_transformed_operation};
use crate::Result;
use crate::config::GraphqlTag;

/// Generate document constants
pub fn generate_documents(ctx: &mut GeneratorContext) -> Result<()> {
    // TODO:
    //
    // let options = ctx.options;
    // let needs_transform = options.inline_fragments || options.dedupe_selections;

    // Add import for graphql tag if needed
    // match options.graphql_tag {
    //     Some(GraphqlTag::Gql) => {
    //         writeln!(ctx.writer, "import {{ gql }} from 'graphql-tag';")?;
    //         writeln!(ctx.writer)?;
    //     }
    //     Some(GraphqlTag::Graphql) => {
    //         writeln!(ctx.writer, "import {{ graphql }} from 'graphql-tag';")?;
    //         writeln!(ctx.writer)?;
    //     }
    //     Some(GraphqlTag::None) | None => {
    //         // No import needed - will output raw strings
    //     }
    // }

    // Collect all documents and sort alphabetically for deterministic output
    // enum DocItem<'a> {
    //     Fragment(&'a str, &'a str), // name, text
    //     Operation(&'a str, &'a crate::documents::ParsedOperation<'a>),
    // }
    //
    // let mut items: Vec<DocItem> = Vec::new();
    //
    // // Add fragments (unless inlining)
    // if !options.inline_fragments {
    //     for (name, fragment) in ctx.fragments.iter() {
    //         items.push(DocItem::Fragment(name.as_str(), fragment.text));
    //     }
    // }
    //
    // // Add operations
    // for (name, operation) in ctx.operations.iter() {
    //     items.push(DocItem::Operation(name.as_str(), operation));
    // }
    //
    // // Sort all items alphabetically
    // items.sort_by_key(|item| match item {
    //     DocItem::Fragment(name, _) => *name,
    //     DocItem::Operation(name, _) => *name,
    // });
    //
    // // Generate in sorted order
    // for item in items {
    //     match item {
    //         DocItem::Fragment(name, text) => {
    //             write_document(ctx, name, text, options.graphql_tag)?;
    //         }
    //         DocItem::Operation(name, operation) => {
    //             if needs_transform {
    //                 let transform_opts = TransformOptions {
    //                     inline_fragments: options.inline_fragments,
    //                     dedupe_selections: options.dedupe_selections,
    //                 };
    //                 let mut buffer = Vec::new();
    //                 write_transformed_operation(
    //                     &mut buffer,
    //                     &operation.definition,
    //                     ctx.fragments,
    //                     &transform_opts,
    //                 )?;
    //                 let text =
    //                     String::from_utf8(buffer).expect("transform output should be valid UTF-8");
    //                 write_document(ctx, name, &text, options.graphql_tag)?;
    //             } else {
    //                 write_document(ctx, name, operation.text, options.graphql_tag)?;
    //             }
    //         }
    //     }
    // }

    Ok(())
}

fn write_document(
    ctx: &mut GeneratorContext,
    name: &str,
    text: &str,
    tag: Option<GraphqlTag>,
) -> Result<()> {
    let doc_name = format!("{name}Document");

    match tag {
        Some(GraphqlTag::Gql) => {
            writeln!(ctx.writer, "export const {doc_name} = gql`")?;
            write_indented_graphql(ctx, text)?;
            writeln!(ctx.writer, "`;")?;
        }
        Some(GraphqlTag::Graphql) => {
            writeln!(ctx.writer, "export const {doc_name} = graphql`")?;
            write_indented_graphql(ctx, text)?;
            writeln!(ctx.writer, "`;")?;
        }
        Some(GraphqlTag::None) | None => {
            writeln!(ctx.writer, "export const {doc_name} = `")?;
            write_indented_graphql(ctx, text)?;
            writeln!(ctx.writer, "`;")?;
        }
    }

    writeln!(ctx.writer)?;
    Ok(())
}

fn write_indented_graphql(ctx: &mut GeneratorContext, text: &str) -> Result<()> {
    for line in text.lines() {
        if line.trim().is_empty() {
            writeln!(ctx.writer)?;
        } else {
            writeln!(ctx.writer, "  {line}")?;
        }
    }
    Ok(())
}
