//! TypeScript operation types generator
//!
//! Generates TypeScript types for GraphQL operations (queries, mutations, subscriptions).
//! Produces types like `GetUserQuery` and `GetUserQueryVariables`.

use std::io::Write;

use apollo_compiler::ast::{Selection, Type};
use apollo_compiler::validation::Valid;
use apollo_compiler::{Name, Schema};

use super::GeneratorContext;
use crate::config::PluginOptions;
use crate::documents::ParsedFragment;
use crate::Result;
use indexmap::IndexMap;

/// Generate TypeScript types for operations
pub fn generate_typescript_operations(ctx: &GeneratorContext, writer: &mut dyn Write) -> Result<()> {
    writeln!(writer, "// Generated TypeScript operation types")?;
    writeln!(writer)?;

    let generator = OperationTypesGenerator {
        schema: ctx.schema,
        fragments: ctx.fragments,
        options: ctx.options,
    };

    // Generate fragment types first (operations may reference them)
    for (name, fragment) in ctx.fragments.iter() {
        generator.generate_fragment_type(writer, name, fragment)?;
    }

    // Generate operation types
    for (name, operation) in ctx.operations.iter() {
        generator.generate_operation_type(writer, name, &operation.definition)?;
    }

    Ok(())
}

struct OperationTypesGenerator<'a> {
    schema: &'a Valid<Schema>,
    fragments: &'a IndexMap<Name, ParsedFragment<'a>>,
    options: &'a PluginOptions,
}

impl<'a> OperationTypesGenerator<'a> {
    fn generate_fragment_type(
        &self,
        writer: &mut dyn Write,
        name: &Name,
        fragment: &ParsedFragment,
    ) -> Result<()> {
        let type_condition = &fragment.definition.type_condition;

        writeln!(writer, "export interface {} {{", name)?;
        self.render_selection_set(writer, &fragment.definition.selection_set, type_condition, 1)?;
        writeln!(writer, "}}")?;
        writeln!(writer)?;

        Ok(())
    }

    fn generate_operation_type(
        &self,
        writer: &mut dyn Write,
        name: &Name,
        operation: &apollo_compiler::ast::OperationDefinition,
    ) -> Result<()> {
        // Get the root type for this operation
        let root_type_name = match self.schema.root_operation(operation.operation_type) {
            Some(name) => name,
            None => return Ok(()), // No root type defined
        };

        // Generate the result type
        writeln!(writer, "export interface {} {{", name)?;
        self.render_selection_set(writer, &operation.selection_set, root_type_name, 1)?;
        writeln!(writer, "}}")?;
        writeln!(writer)?;

        // Generate variables type if there are variables
        if !operation.variables.is_empty() {
            writeln!(writer, "export interface {}Variables {{", name)?;
            for var in &operation.variables {
                let ts_type = self.graphql_type_to_ts(&var.ty);
                let optional = if var.ty.is_non_null() { "" } else { "?" };
                let readonly = if self.options.immutable_types { "readonly " } else { "" };

                // Handle default values
                let has_default = var.default_value.is_some();
                let optional = if has_default { "?" } else { optional };

                writeln!(writer, "  {}{}{}: {};", readonly, var.name, optional, ts_type)?;
            }
            writeln!(writer, "}}")?;
            writeln!(writer)?;
        }

        Ok(())
    }

    fn render_selection_set(
        &self,
        writer: &mut dyn Write,
        selections: &[Selection],
        parent_type: &Name,
        depth: usize,
    ) -> Result<()> {
        let indent = "  ".repeat(depth);
        let readonly = if self.options.immutable_types { "readonly " } else { "" };

        for selection in selections {
            match selection {
                Selection::Field(field) => {
                    let field_name = field.alias.as_ref().unwrap_or(&field.name);

                    // Handle __typename specially
                    if field.name.as_str() == "__typename" {
                        if !self.options.skip_typename {
                            writeln!(writer, "{}{}{}: '{}';", indent, readonly, field_name, parent_type)?;
                        }
                        continue;
                    }

                    // Look up field type in schema
                    let field_def = match self.schema.type_field(parent_type, &field.name) {
                        Ok(f) => f,
                        Err(_) => continue, // Field not found in schema, skip
                    };

                    // Check for @include/@skip directives
                    let has_conditional = field.directives.iter()
                        .any(|d| d.name.as_str() == "include" || d.name.as_str() == "skip");

                    let optional_marker = if has_conditional || !field_def.ty.is_non_null() {
                        "?"
                    } else {
                        ""
                    };

                    // If field has nested selection, render inline object type
                    if !field.selection_set.is_empty() {
                        let inner_type_name = self.unwrap_type_name(&field_def.ty);

                        // Determine wrapper based on type
                        let (open, close) = self.get_type_wrappers(&field_def.ty);

                        writeln!(writer, "{}{}{}{}: {} {{", indent, readonly, field_name, optional_marker, open)?;
                        self.render_selection_set(writer, &field.selection_set, &inner_type_name, depth + 1)?;
                        writeln!(writer, "{}}}{};", indent, close)?;
                    } else {
                        let ts_type = self.graphql_type_to_ts(&field_def.ty);
                        writeln!(writer, "{}{}{}{}: {};", indent, readonly, field_name, optional_marker, ts_type)?;
                    }
                }

                Selection::FragmentSpread(spread) => {
                    // Inline the fragment's fields
                    if let Some(fragment) = self.fragments.get(&spread.fragment_name) {
                        self.render_selection_set(
                            writer,
                            &fragment.definition.selection_set,
                            &fragment.definition.type_condition,
                            depth,
                        )?;
                    }
                }

                Selection::InlineFragment(inline) => {
                    // For inline fragments, render with type condition comment
                    if let Some(type_condition) = &inline.type_condition {
                        // TODO: Handle union/interface discrimination properly
                        // For now, just inline the fields
                        self.render_selection_set(writer, &inline.selection_set, type_condition, depth)?;
                    } else {
                        // No type condition - just inline
                        self.render_selection_set(writer, &inline.selection_set, parent_type, depth)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Convert GraphQL type to TypeScript type string
    fn graphql_type_to_ts(&self, ty: &Type) -> String {
        match ty {
            Type::Named(name) => {
                let ts = self.scalar_to_ts(name);
                format!("{} | null", ts)
            }
            Type::NonNullNamed(name) => {
                self.scalar_to_ts(name)
            }
            Type::List(inner) => {
                let inner_ts = self.graphql_type_to_ts(inner);
                format!("Array<{}> | null", inner_ts)
            }
            Type::NonNullList(inner) => {
                let inner_ts = self.graphql_type_to_ts(inner);
                format!("Array<{}>", inner_ts)
            }
        }
    }

    /// Convert a scalar/named type to TypeScript
    fn scalar_to_ts(&self, name: &Name) -> String {
        // Check custom scalars first
        if let Some(custom) = self.options.scalars.get(name.as_str()) {
            return custom.clone();
        }

        // Built-in scalar mappings
        match name.as_str() {
            "String" | "ID" => "string".to_string(),
            "Int" | "Float" => "number".to_string(),
            "Boolean" => "boolean".to_string(),
            // For enums and other named types, use the type name
            other => other.to_string(),
        }
    }

    /// Get the innermost type name (unwrap NonNull and List)
    fn unwrap_type_name(&self, ty: &Type) -> Name {
        match ty {
            Type::Named(name) | Type::NonNullNamed(name) => name.clone(),
            Type::List(inner) | Type::NonNullList(inner) => self.unwrap_type_name(inner),
        }
    }

    /// Get opening and closing wrappers for array types
    fn get_type_wrappers(&self, ty: &Type) -> (&'static str, &'static str) {
        match ty {
            Type::Named(_) | Type::NonNullNamed(_) => ("", ""),
            Type::List(_) => ("Array<", "> | null"),
            Type::NonNullList(_) => ("Array<", ">"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::{SourceCache, collect_documents, load_sources};
    use crate::config::StringOrArray;
    use crate::extract::ExtractConfig;
    use crate::schema::load_schema;
    use std::path::PathBuf;

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
    }

    #[test]
    fn test_generate_operation_types() {
        // Load schema
        let schema_sources = StringOrArray::Single("schemas/basic.graphql".into());
        let schema = load_schema(&schema_sources, Some(&fixtures_dir())).unwrap();

        // Load documents
        let mut cache = SourceCache::new();
        let doc_patterns = StringOrArray::Single("documents/queries.graphql".into());
        load_sources(&doc_patterns, Some(&fixtures_dir()), &mut cache).unwrap();
        let docs = collect_documents(&cache, &ExtractConfig::default());

        // Generate
        let ctx = super::super::GeneratorContext {
            schema: &schema,
            operations: &docs.operations,
            fragments: &docs.fragments,
            options: &PluginOptions::default(),
        };

        let mut output = Vec::new();
        generate_typescript_operations(&ctx, &mut output).unwrap();
        let result = String::from_utf8(output).unwrap();

        insta::assert_snapshot!(result);
    }

    #[test]
    fn test_generate_operation_types_with_fragments() {
        let schema_sources = StringOrArray::Single("schemas/basic.graphql".into());
        let schema = load_schema(&schema_sources, Some(&fixtures_dir())).unwrap();

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
        generate_typescript_operations(&ctx, &mut output).unwrap();
        let result = String::from_utf8(output).unwrap();

        insta::assert_snapshot!(result);
    }

    #[test]
    fn test_generate_with_immutable_types() {
        let schema_sources = StringOrArray::Single("schemas/basic.graphql".into());
        let schema = load_schema(&schema_sources, Some(&fixtures_dir())).unwrap();

        let mut cache = SourceCache::new();
        let doc_patterns = StringOrArray::Single("documents/queries.graphql".into());
        load_sources(&doc_patterns, Some(&fixtures_dir()), &mut cache).unwrap();
        let docs = collect_documents(&cache, &ExtractConfig::default());

        let options = PluginOptions {
            immutable_types: true,
            ..Default::default()
        };

        let ctx = super::super::GeneratorContext {
            schema: &schema,
            operations: &docs.operations,
            fragments: &docs.fragments,
            options: &options,
        };

        let mut output = Vec::new();
        generate_typescript_operations(&ctx, &mut output).unwrap();
        let result = String::from_utf8(output).unwrap();

        insta::assert_snapshot!(result);
    }
}
