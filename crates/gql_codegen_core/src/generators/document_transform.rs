//! Document transformation utilities
//!
//! Handles inlining fragments and deduping selections in GraphQL documents.

use std::collections::HashSet;
use std::io::Write;

use apollo_compiler::Name;
use apollo_compiler::ast::{OperationDefinition, Selection};

use crate::Result;
use crate::documents::ParsedFragment;

/// Transform options for document generation
pub struct TransformOptions {
    pub inline_fragments: bool,
    pub dedupe_selections: bool,
}

/// Transform and write an operation document directly to a writer
pub fn write_transformed_operation<'a>(
    writer: &mut dyn Write,
    operation: &OperationDefinition,
    fragments: &indexmap::IndexMap<Name, ParsedFragment<'a>>,
    options: &TransformOptions,
) -> Result<()> {
    // Write operation header
    if let Some(name) = &operation.name {
        let op_type = match operation.operation_type {
            apollo_compiler::ast::OperationType::Query => "query",
            apollo_compiler::ast::OperationType::Mutation => "mutation",
            apollo_compiler::ast::OperationType::Subscription => "subscription",
        };
        write!(writer, "{op_type} {}", name.as_str())?;

        // Write variables
        if !operation.variables.is_empty() {
            write!(writer, "(")?;
            let mut first = true;
            for v in &operation.variables {
                if !first {
                    write!(writer, ", ")?;
                }
                first = false;
                write!(writer, "${}: {}", v.name, v.ty)?;
                if let Some(default) = &v.default_value {
                    write!(writer, " = {default}")?;
                }
            }
            write!(writer, ")")?;
        }
    }

    // Write selection set
    writeln!(writer, " {{")?;
    write_selection_set(writer, &operation.selection_set, fragments, options, 1)?;
    write!(writer, "}}")?;

    Ok(())
}

fn write_selection_set<'a>(
    writer: &mut dyn Write,
    selections: &[Selection],
    fragments: &indexmap::IndexMap<Name, ParsedFragment<'a>>,
    options: &TransformOptions,
    indent: usize,
) -> Result<()> {
    let indent_str = "  ".repeat(indent);
    let mut seen_fields: HashSet<String> = HashSet::new();

    for selection in selections.iter() {
        match selection {
            Selection::Field(field) => {
                let field_key = if let Some(alias) = &field.alias {
                    format!("{}:{}", alias, field.name)
                } else {
                    field.name.to_string()
                };

                // Skip if we've seen this field and dedupe is enabled
                if options.dedupe_selections && seen_fields.contains(&field_key) {
                    continue;
                }
                seen_fields.insert(field_key);

                write!(writer, "{indent_str}")?;

                // Write alias if present
                if let Some(alias) = &field.alias {
                    write!(writer, "{}: ", alias.as_str())?;
                }

                write!(writer, "{}", field.name.as_str())?;

                // Write arguments
                if !field.arguments.is_empty() {
                    write!(writer, "(")?;
                    let mut first = true;
                    for arg in &field.arguments {
                        if !first {
                            write!(writer, ", ")?;
                        }
                        first = false;
                        write!(writer, "{}: {}", arg.name, arg.value)?;
                    }
                    write!(writer, ")")?;
                }

                // Write nested selection set
                if !field.selection_set.is_empty() {
                    writeln!(writer, " {{")?;
                    write_selection_set(
                        writer,
                        &field.selection_set,
                        fragments,
                        options,
                        indent + 1,
                    )?;
                    write!(writer, "{indent_str}}}")?;
                }

                writeln!(writer)?;
            }

            Selection::FragmentSpread(spread) => {
                if options.inline_fragments {
                    // Inline the fragment's selection set
                    if let Some(fragment) = fragments.get(&spread.fragment_name) {
                        write_selection_set(
                            writer,
                            &fragment.definition.selection_set,
                            fragments,
                            options,
                            indent,
                        )?;
                    } else {
                        // Fragment not found, write as-is
                        writeln!(writer, "{indent_str}...{}", spread.fragment_name.as_str())?;
                    }
                } else {
                    // Keep fragment spread as-is
                    writeln!(writer, "{indent_str}...{}", spread.fragment_name.as_str())?;
                }
            }

            Selection::InlineFragment(inline) => {
                write!(writer, "{indent_str}...")?;

                if let Some(type_cond) = &inline.type_condition {
                    write!(writer, " on {}", type_cond.as_str())?;
                }

                writeln!(writer, " {{")?;
                write_selection_set(
                    writer,
                    &inline.selection_set,
                    fragments,
                    options,
                    indent + 1,
                )?;
                writeln!(writer, "{indent_str}}}")?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // Tests will be added with snapshot testing
}
