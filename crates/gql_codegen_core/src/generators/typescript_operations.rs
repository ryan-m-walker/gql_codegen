//! TypeScript operation types generator
//!
//! Generates TypeScript types for GraphQL operations (queries, mutations, subscriptions).
//! Produces types like `GetUserQuery` and `GetUserQueryVariables`.

use std::io::Write;

use apollo_compiler::ast::{Selection, Type};
use apollo_compiler::validation::Valid;
use apollo_compiler::{Name, Schema};
use rayon::prelude::*;

use super::GeneratorContext;
use crate::config::PluginOptions;
use crate::documents::ParsedFragment;
use crate::Result;
use indexmap::IndexMap;

/// Generate TypeScript types for operations
pub fn generate_typescript_operations(ctx: &GeneratorContext, writer: &mut dyn Write) -> Result<()> {
    let generator = OperationTypesGenerator {
        schema: ctx.schema,
        fragments: ctx.fragments,
        options: ctx.options,
    };

    // Generate fragment types in parallel
    let t0 = std::time::Instant::now();
    let fragments: Vec<_> = ctx.fragments.iter().collect();
    let fragment_buffers: Vec<Vec<u8>> = fragments
        .par_iter()
        .map(|(name, fragment)| {
            let mut buffer = Vec::new();
            generator.generate_fragment_type(&mut buffer, name, fragment).ok();
            buffer
        })
        .collect();

    // Write all fragments to output
    for buffer in &fragment_buffers {
        writer.write_all(buffer)?;
    }
    crate::timing!("    Fragments", t0.elapsed(), "{} total", ctx.fragments.len());

    // Generate operation types in parallel
    let t0 = std::time::Instant::now();
    let operations: Vec<_> = ctx.operations.iter().collect();
    let operation_buffers: Vec<Vec<u8>> = operations
        .par_iter()
        .map(|(name, operation)| {
            let mut buffer = Vec::new();
            generator.generate_operation_type(&mut buffer, name, &operation.definition).ok();
            buffer
        })
        .collect();

    // Write all operations to output
    for buffer in &operation_buffers {
        writer.write_all(buffer)?;
    }
    crate::timing!("    Operations", t0.elapsed(), "{} total", ctx.operations.len());

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

        writeln!(writer, "export interface {name} {{")?;
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
        writeln!(writer, "export interface {name} {{")?;
        self.render_selection_set(writer, &operation.selection_set, root_type_name, 1)?;
        writeln!(writer, "}}")?;
        writeln!(writer)?;

        // Generate variables type if there are variables
        if !operation.variables.is_empty() {
            writeln!(writer, "export interface {name}Variables {{")?;
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
                            writeln!(writer, "{indent}{readonly}{field_name}: '{parent_type}';")?;
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
                        let inner_type_name = Self::unwrap_type_name(&field_def.ty);

                        // Determine wrapper based on type
                        let (open, close) = self.get_type_wrappers(&field_def.ty);

                        writeln!(writer, "{indent}{readonly}{field_name}{optional_marker}: {open} {{")?;
                        self.render_selection_set(writer, &field.selection_set, &inner_type_name, depth + 1)?;
                        writeln!(writer, "{indent}}}{close};")?;
                    } else {
                        let ts_type = self.graphql_type_to_ts(&field_def.ty);
                        writeln!(writer, "{indent}{readonly}{field_name}{optional_marker}: {ts_type};")?;
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
                format!("{ts} | null")
            }
            Type::NonNullNamed(name) => {
                self.scalar_to_ts(name)
            }
            Type::List(inner) => {
                let inner_ts = self.graphql_type_to_ts(inner);
                format!("Array<{inner_ts}> | null")
            }
            Type::NonNullList(inner) => {
                let inner_ts = self.graphql_type_to_ts(inner);
                format!("Array<{inner_ts}>")
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
    fn unwrap_type_name(ty: &Type) -> Name {
        let mut current = ty;
        loop {
            match current {
                Type::Named(name) | Type::NonNullNamed(name) => return name.clone(),
                Type::List(inner) | Type::NonNullList(inner) => current = inner,
            }
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
