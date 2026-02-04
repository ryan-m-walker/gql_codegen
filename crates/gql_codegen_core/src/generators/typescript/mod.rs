//! TypeScript schema types generator
//!
//! Generates TypeScript types for GraphQL schema types (objects, interfaces,
//! enums, unions, input objects).

use std::collections::HashSet;
use std::io::Write;

use apollo_compiler::ast::{Selection, Type};
use apollo_compiler::schema::ExtendedType;

use super::GeneratorContext;
use crate::Result;
use crate::config::{NamingCase, NamingConvention, PluginOptions};
use crate::generators::typescript::helpers::{
    get_export_kw, render_decl_closing, render_decl_opening, render_description,
};
use crate::generators::typescript::r#enum::render_enum;
use crate::generators::typescript::object::render_object;
use crate::generators::typescript::scalars::render_scalars;
use crate::generators::typescript::utils::generate_util_types;

mod r#enum;
mod field;
mod helpers;
mod object;
mod scalars;
mod utils;

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

/// Apply naming convention to a type name
fn transform_type_name<'a>(name: &'a str, options: &PluginOptions) -> std::borrow::Cow<'a, str> {
    let (case, transform_underscore) = get_type_name_case(options);
    case.apply(name, transform_underscore)
}

/// Collect all types referenced in operations and fragments
fn collect_operation_types(ctx: &GeneratorContext) -> HashSet<String> {
    let mut referenced = HashSet::new();
    let schema = ctx.schema;

    // Helper to get field type from schema
    fn get_field_type<'a>(
        schema: &'a apollo_compiler::validation::Valid<apollo_compiler::Schema>,
        parent_type: &str,
        field_name: &apollo_compiler::Name,
    ) -> Option<&'a Type> {
        let ty = schema.types.get(parent_type)?;
        match ty {
            ExtendedType::Object(obj) => obj.fields.get(field_name).map(|f| &f.ty),
            ExtendedType::Interface(iface) => iface.fields.get(field_name).map(|f| &f.ty),
            _ => None,
        }
    }

    // Helper to get all field types for transitive expansion
    fn collect_field_types(ty: &ExtendedType, out: &mut HashSet<String>) {
        let fields = match ty {
            ExtendedType::Object(obj) => Some(&obj.fields),
            ExtendedType::Interface(iface) => Some(&iface.fields),
            ExtendedType::InputObject(input) => {
                // Input objects have different field type
                for (_, field) in input.fields.iter() {
                    collect_type_name(&field.ty, out);
                }
                return;
            }
            _ => None,
        };
        if let Some(fields) = fields {
            for (_, field) in fields.iter() {
                collect_type_name(&field.ty, out);
            }
        }
    }

    // Helper to extract type name from a Type
    fn collect_type_name(ty: &Type, out: &mut HashSet<String>) {
        match ty {
            Type::Named(name) | Type::NonNullNamed(name) => {
                out.insert(name.to_string());
            }
            Type::List(inner) | Type::NonNullList(inner) => {
                collect_type_name(inner, out);
            }
        }
    }

    fn unwrap_type_name(ty: &Type) -> apollo_compiler::Name {
        match ty {
            Type::Named(name) | Type::NonNullNamed(name) => name.clone(),
            Type::List(inner) | Type::NonNullList(inner) => unwrap_type_name(inner),
        }
    }

    // Collect types from a selection set (iterative to avoid nested fn issues)
    fn collect_from_selections(
        selections: &[Selection],
        parent_type: &str,
        schema: &apollo_compiler::validation::Valid<apollo_compiler::Schema>,
        out: &mut HashSet<String>,
    ) {
        let mut stack: Vec<(&[Selection], String)> = vec![(selections, parent_type.to_string())];

        while let Some((sels, parent)) = stack.pop() {
            for selection in sels {
                match selection {
                    Selection::Field(field) => {
                        if let Some(field_ty) = get_field_type(schema, &parent, &field.name) {
                            collect_type_name(field_ty, out);
                            let field_type_name = unwrap_type_name(field_ty);
                            if !field.selection_set.is_empty() {
                                stack.push((&field.selection_set, field_type_name.to_string()));
                            }
                        }
                    }
                    Selection::InlineFragment(inline) => {
                        let type_name = inline
                            .type_condition
                            .as_ref()
                            .map(|t| t.to_string())
                            .unwrap_or_else(|| parent.clone());
                        out.insert(type_name.clone());
                        if !inline.selection_set.is_empty() {
                            stack.push((&inline.selection_set, type_name));
                        }
                    }
                    Selection::FragmentSpread(_) => {
                        // Fragment types are collected when we process fragments
                    }
                }
            }
        }
    }

    // Collect from operations
    for (_, operation) in ctx.operations.iter() {
        for var in &operation.definition.variables {
            collect_type_name(&var.ty, &mut referenced);
        }

        if let Some(root_type) = schema.root_operation(operation.definition.operation_type) {
            referenced.insert(root_type.to_string());
            collect_from_selections(
                &operation.definition.selection_set,
                root_type.as_str(),
                schema,
                &mut referenced,
            );
        }
    }

    // Collect from fragments
    for (_, fragment) in ctx.fragments.iter() {
        let type_condition = fragment.definition.type_condition.as_str();
        referenced.insert(type_condition.to_string());
        collect_from_selections(
            &fragment.definition.selection_set,
            type_condition,
            schema,
            &mut referenced,
        );
    }

    // Transitively expand to include all referenced types
    let mut to_process: Vec<String> = referenced.iter().cloned().collect();
    while let Some(type_name) = to_process.pop() {
        if let Some(ty) = schema.types.get(type_name.as_str()) {
            // Collect field types into a temp set, then add new ones
            let mut new_types = HashSet::new();
            collect_field_types(ty, &mut new_types);
            for field_type in new_types {
                if referenced.insert(field_type.clone()) {
                    to_process.push(field_type);
                }
            }

            // Collect union members
            if let ExtendedType::Union(union) = ty {
                for member in &union.members {
                    let member_name = member.name.to_string();
                    if referenced.insert(member_name.clone()) {
                        to_process.push(member_name);
                    }
                }
            }
            // Collect interface implementers
            if let ExtendedType::Interface(_) = ty {
                for (impl_name, impl_ty) in schema.types.iter() {
                    if let ExtendedType::Object(obj) = impl_ty {
                        if obj
                            .implements_interfaces
                            .iter()
                            .any(|i| i.name.as_str() == type_name)
                        {
                            let name = impl_name.to_string();
                            if referenced.insert(name.clone()) {
                                to_process.push(name);
                            }
                        }
                    }
                }
            }
        }
    }

    referenced
}

/// Generate TypeScript types from the GraphQL schema
pub fn generate_typescript(ctx: &GeneratorContext, writer: &mut dyn Write) -> Result<()> {
    let export = get_export_kw(ctx);

    // Collect referenced types if only_operation_types is enabled
    let referenced_types = if ctx.options.only_operation_types {
        Some(collect_operation_types(ctx))
    } else {
        None
    };

    if ctx.options.use_utility_types {
        generate_util_types(ctx, writer)?;
        render_scalars(ctx, writer)?;
    }

    // Iterate over schema types
    for (name, ty) in ctx.schema.types.iter() {
        // Skip built-in types
        if name.as_str().starts_with("__") {
            continue;
        }

        // Skip types not referenced in operations (if only_operation_types is enabled)
        if let Some(ref referenced) = referenced_types {
            if !referenced.contains(name.as_str()) {
                continue;
            }
        }

        let readonly = if ctx.options.immutable_types {
            "readonly "
        } else {
            ""
        };

        // Apply naming convention to type name
        let type_name = transform_type_name(name.as_str(), ctx.options);

        match ty {
            apollo_compiler::schema::ExtendedType::Object(obj) => {
                render_object(obj, ctx, writer)?;
                // render_description(&obj.description, 0, writer)?;
                // render_decl_opening(&type_name, ctx, writer)?;
                //
                // if !ctx.options.skip_typename {
                //     let optional = if ctx.options.non_optional_typename {
                //         ""
                //     } else {
                //         "?"
                //     };
                //
                //     writeln!(writer, "  {readonly}__typename{optional}: '{type_name}';")?;
                // }
                //
                // for (field_name, field) in obj.fields.iter() {
                //     render_field(field_name, field, ctx, writer)?;
                // }
                //
                // render_decl_closing(ctx, writer)?;
                // writeln!(writer)?;
            }

            apollo_compiler::schema::ExtendedType::Enum(en) => {
                render_enum(en, ctx, writer)?;
            }

            apollo_compiler::schema::ExtendedType::Interface(iface) => {
                render_decl_opening(&type_name, ctx, writer)?;

                for (field_name, field) in iface.fields.iter() {
                    let field_type = format_type(&field.ty, ctx.options);
                    writeln!(writer, "  {readonly}{field_name}: {field_type};")?;
                }

                render_decl_closing(ctx, writer)?;
                writeln!(writer)?;
            }

            apollo_compiler::schema::ExtendedType::Union(union) => {
                let members: Vec<_> = union
                    .members
                    .iter()
                    .map(|m| transform_type_name(m.name.as_str(), ctx.options))
                    .collect();
                writeln!(
                    writer,
                    "{export}type {type_name} = {};",
                    members.join(" | ")
                )?;
                writeln!(writer)?;
            }

            apollo_compiler::schema::ExtendedType::InputObject(input) => {
                render_decl_opening(&type_name, ctx, writer)?;

                for (field_name, field) in input.fields.iter() {
                    // Use input-specific type formatting for input objects
                    let field_type = format_input_type(&field.ty, ctx.options);
                    let optional = if field.ty.is_non_null() { "" } else { "?" };
                    writeln!(writer, "  {readonly}{field_name}{optional}: {field_type};")?;
                }

                render_decl_closing(ctx, writer)?;
                writeln!(writer)?;
            }

            apollo_compiler::schema::ExtendedType::Scalar(scalar) => {
                if ctx.options.use_utility_types {
                    continue;
                }

                if matches!(name.as_str(), "String" | "ID" | "Int" | "Float" | "Boolean") {
                    continue;
                }

                let custom_type = ctx.options.scalars.get(name.as_str());

                let ts_type = if let Some(ref custom_type) = custom_type {
                    custom_type
                } else if let Some(ref default_type) = ctx.options.default_scalar_type {
                    default_type
                } else {
                    "unknown"
                };

                render_description(&scalar.description, 0, writer)?;
                writeln!(writer, "{export}type {type_name} = {ts_type};")?;
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

/// Wrap a type with nullable wrapper (Maybe or explicit null)
#[inline]
fn wrap_nullable(ts_type: &str, _options: &crate::config::PluginOptions, is_input: bool) -> String {
    if is_input {
        format!("InputMaybe<{ts_type}>")
    } else {
        format!("Maybe<{ts_type}>")
    }
}
