//! TypeScript schema types generator
//!
//! Generates TypeScript types for GraphQL schema types (objects, interfaces,
//! enums, unions, input objects).

use std::io::Write;

use super::GeneratorContext;
use crate::config::{NamingCase, NamingConvention, PluginOptions};
use crate::Result;

/// Get the naming case for type names from options
fn get_type_name_case(options: &PluginOptions) -> (NamingCase, bool) {
    match &options.naming_convention {
        None => (NamingCase::Keep, false),
        // Simple convention preserves underscores by default (matches graphql-codegen)
        Some(NamingConvention::Simple(case)) => (*case, false),
        Some(NamingConvention::Detailed(config)) => {
            let case = config.type_names.unwrap_or(NamingCase::Keep);
            (case, config.transform_underscore)
        }
    }
}

/// Get the naming case for enum values from options
fn get_enum_value_case(options: &PluginOptions) -> (NamingCase, bool) {
    match &options.naming_convention {
        None => (NamingCase::Keep, false),
        // Simple convention preserves underscores by default (matches graphql-codegen)
        Some(NamingConvention::Simple(case)) => (*case, false),
        Some(NamingConvention::Detailed(config)) => {
            let case = config.enum_values.unwrap_or(NamingCase::Keep);
            (case, config.transform_underscore)
        }
    }
}

/// Apply naming convention to a type name
fn transform_type_name(name: &str, options: &PluginOptions) -> String {
    let (case, transform_underscore) = get_type_name_case(options);
    case.apply(name, transform_underscore)
}

/// Apply naming convention to an enum value
fn transform_enum_value(value: &str, options: &PluginOptions) -> String {
    let (case, transform_underscore) = get_enum_value_case(options);
    case.apply(value, transform_underscore)
}

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

        // Apply naming convention to type name
        let type_name = transform_type_name(name.as_str(), options);

        match ty {
            apollo_compiler::schema::ExtendedType::Object(obj) => {
                writeln!(writer, "export interface {type_name} {{")?;

                if !options.skip_typename {
                    // __typename uses original GraphQL name, not transformed
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
                    write!(writer, "export type {type_name} = ")?;

                    for (i, value) in en.values.keys().enumerate() {
                        let transformed_value = transform_enum_value(value.as_str(), options);
                        write!(writer, "'{transformed_value}'")?;

                        if i < en.values.len() - 1 {
                            write!(writer, " | ")?;
                        }
                    }

                    if options.future_proof_enums {
                        write!(writer, " | '%future added value'")?;
                    }

                    writeln!(writer, ";")?;
                } else {
                    writeln!(writer, "export enum {type_name} {{")?;
                    for value in en.values.keys() {
                        let transformed_value = transform_enum_value(value.as_str(), options);
                        // Enum member name is transformed, value stays original
                        writeln!(writer, "  {transformed_value} = '{value}',")?;
                    }
                    writeln!(writer, "}}")?;
                }
                writeln!(writer)?;
            }

            apollo_compiler::schema::ExtendedType::Interface(iface) => {
                writeln!(writer, "export interface {type_name} {{")?;

                for (field_name, field) in iface.fields.iter() {
                    let field_type = format_type(&field.ty, options);
                    writeln!(writer, "  {readonly}{field_name}: {field_type};")?;
                }

                writeln!(writer, "}}")?;
                writeln!(writer)?;
            }

            apollo_compiler::schema::ExtendedType::Union(union) => {
                let members: Vec<_> = union
                    .members
                    .iter()
                    .map(|m| transform_type_name(m.name.as_str(), options))
                    .collect();
                writeln!(writer, "export type {type_name} = {};", members.join(" | "))?;
                writeln!(writer)?;
            }

            apollo_compiler::schema::ExtendedType::InputObject(input) => {
                writeln!(writer, "export interface {type_name} {{")?;

                for (field_name, field) in input.fields.iter() {
                    // Use input-specific type formatting for input objects
                    let field_type = format_input_type(&field.ty, options);
                    let optional = if field.ty.is_non_null() { "" } else { "?" };
                    writeln!(writer, "  {readonly}{field_name}{optional}: {field_type};")?;
                }

                writeln!(writer, "}}")?;
                writeln!(writer)?;
            }

            apollo_compiler::schema::ExtendedType::Scalar(_) => {
                // Skip built-in scalars
                match name.as_str() {
                    "String" | "ID" | "Int" | "Float" | "Boolean" => continue,
                    _ => {}
                }

                // Check for custom scalar mapping
                if let Some(ts_type) = options.scalars.get(name.as_str()) {
                    writeln!(writer, "export type {type_name} = {ts_type};")?;
                } else if options.strict_scalars {
                    return Err(crate::Error::Config(format!(
                        "Unknown scalar '{name}' found but strictScalars is enabled. Add it to the scalars config."
                    )));
                } else {
                    // Use default_scalar_type or fallback to "unknown"
                    let ts_type = options.default_scalar_type.as_deref().unwrap_or("unknown");
                    writeln!(writer, "export type {type_name} = {ts_type};")?;
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
    format_type_with_context(ty, options, false)
}

/// Format a GraphQL type for input context (uses input_maybe_value if set)
fn format_input_type(
    ty: &apollo_compiler::schema::Type,
    options: &crate::config::PluginOptions,
) -> String {
    format_type_with_context(ty, options, true)
}

/// Format a GraphQL type as a TypeScript type with input context awareness
fn format_type_with_context(
    ty: &apollo_compiler::schema::Type,
    options: &crate::config::PluginOptions,
    is_input: bool,
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

            wrap_nullable(&ts_type, options, is_input)
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
            let inner_type = format_type_with_context(inner, options, is_input);
            let array_type = format!("Array<{inner_type}>");
            wrap_nullable(&array_type, options, is_input)
        }
        apollo_compiler::schema::Type::NonNullList(inner) => {
            let inner_type = format_type_with_context(inner, options, is_input);
            format!("Array<{inner_type}>")
        }
    }
}

// TODO: CLAUDE: double check replace T doesn't conflict with other Ts that might be used
//
/// Wrap a type with nullable wrapper (Maybe or explicit null)
fn wrap_nullable(ts_type: &str, options: &crate::config::PluginOptions, is_input: bool) -> String {
    if options.avoid_optionals {
        format!("{ts_type} | null")
    } else if is_input {
        // Use input_maybe_value if set, otherwise fall back to maybe_value pattern
        if let Some(ref input_maybe) = options.input_maybe_value {
            input_maybe.replace("T", ts_type)
        } else if let Some(ref maybe) = options.maybe_value {
            maybe.replace("T", ts_type)
        } else {
            format!("Maybe<{ts_type}>")
        }
    } else if let Some(ref maybe) = options.maybe_value {
        maybe.replace("T", ts_type)
    } else {
        format!("Maybe<{ts_type}>")
    }
}
