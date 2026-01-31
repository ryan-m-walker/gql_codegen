//! GraphQL document constants generator
//!
//! Generates TypeScript/JavaScript constants containing GraphQL documents.
//! These can be used with Apollo Client, urql, or other GraphQL clients.

use std::io::Write;

use super::document_transform::{write_transformed_operation, TransformOptions};
use super::GeneratorContext;
use crate::config::GraphqlTag;
use crate::Result;

/// Generate document constants
pub fn generate_documents(ctx: &GeneratorContext, writer: &mut dyn Write) -> Result<()> {
    let options = ctx.options;
    let needs_transform = options.inline_fragments || options.dedupe_selections;

    // Add import for graphql tag if needed
    match options.graphql_tag {
        Some(GraphqlTag::Gql) => {
            writeln!(writer, "import {{ gql }} from 'graphql-tag';")?;
            writeln!(writer)?;
        }
        Some(GraphqlTag::Graphql) => {
            writeln!(writer, "import {{ graphql }} from 'graphql-tag';")?;
            writeln!(writer)?;
        }
        Some(GraphqlTag::None) | None => {
            // No import needed - will output raw strings
        }
    }

    // Generate fragment documents first (unless inlining, then skip standalone fragments)
    if !options.inline_fragments {
        for (name, fragment) in ctx.fragments.iter() {
            write_document(writer, name.as_str(), fragment.text, options.graphql_tag)?;
        }
    }

    // Generate operation documents
    for (name, operation) in ctx.operations.iter() {
        if needs_transform {
            let transform_opts = TransformOptions {
                inline_fragments: options.inline_fragments,
                dedupe_selections: options.dedupe_selections,
            };
            // Stream transformed operation to a buffer, then write with indentation
            let mut buffer = Vec::new();
            write_transformed_operation(&mut buffer, &operation.definition, ctx.fragments, &transform_opts)?;
            let text = String::from_utf8(buffer).expect("transform output should be valid UTF-8");
            write_document(writer, name.as_str(), &text, options.graphql_tag)?;
        } else {
            // Use original text directly (zero-copy)
            write_document(writer, name.as_str(), operation.text, options.graphql_tag)?;
        };
    }

    Ok(())
}

fn write_document(
    writer: &mut dyn Write,
    name: &str,
    text: &str,
    tag: Option<GraphqlTag>,
) -> Result<()> {
    let doc_name = format!("{name}Document");

    match tag {
        Some(GraphqlTag::Gql) => {
            writeln!(writer, "export const {doc_name} = gql`")?;
            write_indented_graphql(writer, text)?;
            writeln!(writer, "`;")?;
        }
        Some(GraphqlTag::Graphql) => {
            writeln!(writer, "export const {doc_name} = graphql`")?;
            write_indented_graphql(writer, text)?;
            writeln!(writer, "`;")?;
        }
        Some(GraphqlTag::None) | None => {
            // Output as plain string
            writeln!(writer, "export const {doc_name} = `")?;
            write_indented_graphql(writer, text)?;
            writeln!(writer, "`;")?;
        }
    }

    writeln!(writer)?;
    Ok(())
}

fn write_indented_graphql(writer: &mut dyn Write, text: &str) -> Result<()> {
    for line in text.lines() {
        if line.trim().is_empty() {
            writeln!(writer)?;
        } else {
            writeln!(writer, "  {line}")?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{PluginOptions, StringOrArray};
    use crate::documents::{collect_documents, load_sources, SourceCache};
    use crate::extract::ExtractConfig;
    use crate::schema::{load_schema, resolve_schema_paths};
    use std::path::PathBuf;

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
    }

    #[test]
    fn test_generate_documents_plain() {
        let schema_sources = StringOrArray::Single("schemas/basic.graphql".into());
        let schema = load_schema(&resolve_schema_paths(&schema_sources.as_vec(), Some(&fixtures_dir()))).unwrap();

        let mut cache = SourceCache::new();
        let doc_patterns = StringOrArray::Single("documents/queries.graphql".into());
        load_sources(&doc_patterns, Some(&fixtures_dir()), &mut cache).unwrap();
        let docs = collect_documents(&cache, &ExtractConfig::default());

        let ctx = super::super::GeneratorContext {
            schema: &schema,
            operations: &docs.operations,
            fragments: &docs.fragments,
            options: &PluginOptions::default(),
        };

        let mut output = Vec::new();
        generate_documents(&ctx, &mut output).unwrap();
        let result = String::from_utf8(output).unwrap();

        insta::assert_snapshot!(result);
    }

    #[test]
    fn test_generate_documents_with_gql_tag() {
        let schema_sources = StringOrArray::Single("schemas/basic.graphql".into());
        let schema = load_schema(&resolve_schema_paths(&schema_sources.as_vec(), Some(&fixtures_dir()))).unwrap();

        let mut cache = SourceCache::new();
        let doc_patterns = StringOrArray::Single("documents/queries.graphql".into());
        load_sources(&doc_patterns, Some(&fixtures_dir()), &mut cache).unwrap();
        let docs = collect_documents(&cache, &ExtractConfig::default());

        let options = PluginOptions {
            graphql_tag: Some(GraphqlTag::Gql),
            ..Default::default()
        };

        let ctx = super::super::GeneratorContext {
            schema: &schema,
            operations: &docs.operations,
            fragments: &docs.fragments,
            options: &options,
        };

        let mut output = Vec::new();
        generate_documents(&ctx, &mut output).unwrap();
        let result = String::from_utf8(output).unwrap();

        insta::assert_snapshot!(result);
    }

    #[test]
    fn test_generate_documents_with_fragments() {
        let schema_sources = StringOrArray::Single("schemas/basic.graphql".into());
        let schema = load_schema(&resolve_schema_paths(&schema_sources.as_vec(), Some(&fixtures_dir()))).unwrap();

        let mut cache = SourceCache::new();
        let doc_patterns = StringOrArray::Single("documents/fragments.graphql".into());
        load_sources(&doc_patterns, Some(&fixtures_dir()), &mut cache).unwrap();
        let docs = collect_documents(&cache, &ExtractConfig::default());

        let ctx = super::super::GeneratorContext {
            schema: &schema,
            operations: &docs.operations,
            fragments: &docs.fragments,
            options: &PluginOptions::default(),
        };

        let mut output = Vec::new();
        generate_documents(&ctx, &mut output).unwrap();
        let result = String::from_utf8(output).unwrap();

        insta::assert_snapshot!(result);
    }
}
