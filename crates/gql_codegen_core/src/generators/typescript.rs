//! TypeScript schema types generator
//!
//! Generates TypeScript types for GraphQL schema types (objects, interfaces,
//! enums, unions, input objects).

use std::io::Write;

use super::GeneratorContext;
use crate::Result;

/// Generate TypeScript types from the GraphQL schema
pub fn generate_typescript(ctx: &GeneratorContext, writer: &mut dyn Write) -> Result<()> {
    let options = ctx.options;
    let schema = ctx.schema;

    // Generate Maybe type alias (unless using null style)
    if !options.avoid_optionals {
        writeln!(writer, "export type Maybe<T> = T | null;")?;
        writeln!(writer)?;
    }

    // Iterate over schema types
    for (name, ty) in schema.types.iter() {
        // Skip built-in types
        if name.as_str().starts_with("__") {
            continue;
        }

        let readonly = if options.immutable_types {
            "readonly "
        } else {
            ""
        };

        match ty {
            apollo_compiler::schema::ExtendedType::Object(obj) => {
                writeln!(writer, "export interface {name} {{")?;

                if !options.skip_typename {
                    writeln!(writer, "  {readonly}__typename: '{name}';")?;
                }

                for (field_name, field) in obj.fields.iter() {
                    let field_type = format_type(&field.ty, options);
                    writeln!(writer, "  {readonly}{field_name}: {field_type};")?;
                }

                writeln!(writer, "}}")?;
                writeln!(writer)?;
            }

            apollo_compiler::schema::ExtendedType::Enum(en) => {
                if options.enums_as_types {
                    write!(writer, "export type {name} = ")?;

                    for (i, value) in en.values.keys().enumerate() {
                        write!(writer, "'{value}'")?;

                        if i < en.values.len() - 1 {
                            write!(writer, " | ")?;
                        }
                    }

                    if options.future_proof_enums {
                        write!(writer, " | '%future added value'")?;
                    }

                    writeln!(writer, ";")?;
                } else {
                    writeln!(writer, "export enum {name} {{")?;
                    for value in en.values.keys() {
                        writeln!(writer, "  {value} = '{value}',")?;
                    }
                    writeln!(writer, "}}")?;
                }
                writeln!(writer)?;
            }

            apollo_compiler::schema::ExtendedType::Interface(iface) => {
                writeln!(writer, "export interface {name} {{")?;

                for (field_name, field) in iface.fields.iter() {
                    let field_type = format_type(&field.ty, options);
                    writeln!(writer, "  {readonly}{field_name}: {field_type};")?;
                }

                writeln!(writer, "}}")?;
                writeln!(writer)?;
            }

            apollo_compiler::schema::ExtendedType::Union(union) => {
                let members: Vec<_> = union.members.iter().map(|m| m.name.to_string()).collect();
                writeln!(writer, "export type {} = {};", name, members.join(" | "))?;
                writeln!(writer)?;
            }

            apollo_compiler::schema::ExtendedType::InputObject(input) => {
                writeln!(writer, "export interface {name} {{")?;

                for (field_name, field) in input.fields.iter() {
                    let field_type = format_type(&field.ty, options);
                    let optional = if field.ty.is_non_null() { "" } else { "?" };
                    writeln!(writer, "  {readonly}{field_name}{optional}: {field_type};")?;
                }

                writeln!(writer, "}}")?;
                writeln!(writer)?;
            }

            apollo_compiler::schema::ExtendedType::Scalar(_) => {
                // Check for custom scalar mapping
                if let Some(ts_type) = options.scalars.get(name.as_str()) {
                    writeln!(writer, "export type {name} = {ts_type};")?;
                } else {
                    // Default scalar mappings
                    let ts_type = match name.as_str() {
                        "String" | "ID" => continue, // Built-in, skip
                        "Int" | "Float" => continue,
                        "Boolean" => continue,
                        _ => "unknown",
                    };
                    writeln!(writer, "export type {name} = {ts_type};")?;
                }
                writeln!(writer)?;
            }
        }
    }

    Ok(())
}

/// Format a GraphQL type as a TypeScript type
fn format_type(
    ty: &apollo_compiler::schema::Type,
    options: &crate::config::PluginOptions,
) -> String {
    match ty {
        apollo_compiler::schema::Type::Named(name) => {
            let ts_type = match name.as_str() {
                "String" | "ID" => "string".to_string(),
                "Int" | "Float" => "number".to_string(),
                "Boolean" => "boolean".to_string(),
                other => {
                    // Check custom scalars
                    options
                        .scalars
                        .get(other)
                        .cloned()
                        .unwrap_or_else(|| other.to_string())
                }
            };

            if options.avoid_optionals {
                format!("{ts_type} | null")
            } else {
                format!("Maybe<{ts_type}>")
            }
        }
        apollo_compiler::schema::Type::NonNullNamed(name) => match name.as_str() {
            "String" | "ID" => "string".to_string(),
            "Int" | "Float" => "number".to_string(),
            "Boolean" => "boolean".to_string(),
            other => options
                .scalars
                .get(other)
                .cloned()
                .unwrap_or_else(|| other.to_string()),
        },
        apollo_compiler::schema::Type::List(inner) => {
            let inner_type = format_type(inner, options);
            if options.avoid_optionals {
                format!("Array<{inner_type}> | null")
            } else {
                format!("Maybe<Array<{inner_type}>>")
            }
        }
        apollo_compiler::schema::Type::NonNullList(inner) => {
            let inner_type = format_type(inner, options);
            format!("Array<{inner_type}>")
        }
    }
}
