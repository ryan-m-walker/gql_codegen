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
use crate::generators::typescript::base_types::generate_base_types;
use crate::generators::typescript::scalars::generate_scalars;
use crate::generators::typescript::utils::{
    generate_decl_closing, generate_decl_opening, get_export_kw,
};

mod base_types;
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
fn transform_type_name<'a>(name: &'a str, options: &PluginOptions) -> std::borrow::Cow<'a, str> {
    let (case, transform_underscore) = get_type_name_case(options);
    case.apply(name, transform_underscore)
}

/// Apply naming convention to an enum value
fn transform_enum_value<'a>(value: &'a str, options: &PluginOptions) -> std::borrow::Cow<'a, str> {
    let (case, transform_underscore) = get_enum_value_case(options);
    case.apply(value, transform_underscore)
}

/// Apply enum prefix and suffix to a type name
fn apply_enum_affixes<'a>(
    type_name: &'a str,
    options: &PluginOptions,
) -> std::borrow::Cow<'a, str> {
    use std::borrow::Cow;
    match (&options.enum_prefix, &options.enum_suffix) {
        (None, None) => Cow::Borrowed(type_name),
        (prefix, suffix) => {
            let prefix = prefix.as_deref().unwrap_or("");
            let suffix = suffix.as_deref().unwrap_or("");
            Cow::Owned(format!("{prefix}{type_name}{suffix}"))
        }
    }
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
    let options = ctx.options;
    let schema = ctx.schema;
    let export = get_export_kw(ctx);

    // Collect referenced types if only_operation_types is enabled
    let referenced_types = if options.only_operation_types {
        Some(collect_operation_types(ctx))
    } else {
        None
    };

    generate_base_types(ctx, writer)?;
    generate_scalars(ctx, writer)?;

    // Iterate over schema types
    for (name, ty) in schema.types.iter() {
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

        let readonly = if options.immutable_types {
            "readonly "
        } else {
            ""
        };

        // Apply naming convention to type name
        let type_name = transform_type_name(name.as_str(), options);

        match ty {
            apollo_compiler::schema::ExtendedType::Object(obj) => {
                generate_decl_opening(&type_name, ctx, writer)?;

                if !options.skip_typename {
                    // __typename uses original GraphQL name, not transformed
                    writeln!(writer, "  {readonly}__typename: '{name}';")?;
                }

                for (field_name, field) in obj.fields.iter() {
                    let field_type = format_type(&field.ty, options);
                    writeln!(writer, "  {readonly}{field_name}: {field_type};")?;
                }

                generate_decl_closing(ctx, writer)?;
                writeln!(writer)?;
            }

            apollo_compiler::schema::ExtendedType::Enum(en) => {
                let enum_name = apply_enum_affixes(&type_name, options);
                if options.enums_as_types {
                    write!(writer, "{export}type {enum_name} = ")?;

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
                    let const_kw = if options.const_enums { "const " } else { "" };
                    writeln!(writer, "{export}{const_kw}enum {enum_name} {{")?;
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
                generate_decl_opening(&type_name, ctx, writer)?;

                for (field_name, field) in iface.fields.iter() {
                    let field_type = format_type(&field.ty, options);
                    writeln!(writer, "  {readonly}{field_name}: {field_type};")?;
                }

                generate_decl_closing(ctx, writer)?;
                writeln!(writer)?;
            }

            apollo_compiler::schema::ExtendedType::Union(union) => {
                let members: Vec<_> = union
                    .members
                    .iter()
                    .map(|m| transform_type_name(m.name.as_str(), options))
                    .collect();
                writeln!(
                    writer,
                    "{export}type {type_name} = {};",
                    members.join(" | ")
                )?;
                writeln!(writer)?;
            }

            apollo_compiler::schema::ExtendedType::InputObject(input) => {
                generate_decl_opening(&type_name, ctx, writer)?;

                for (field_name, field) in input.fields.iter() {
                    // Use input-specific type formatting for input objects
                    let field_type = format_input_type(&field.ty, options);
                    let optional = if field.ty.is_non_null() { "" } else { "?" };
                    writeln!(writer, "  {readonly}{field_name}{optional}: {field_type};")?;
                }

                generate_decl_closing(ctx, writer)?;
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
                    writeln!(writer, "{export}type {type_name} = {ts_type};")?;
                } else if options.strict_scalars {
                    return Err(crate::Error::Config(format!(
                        "Unknown scalar '{name}' found but strictScalars is enabled. Add it to the scalars config."
                    )));
                } else {
                    // Use default_scalar_type or fallback to "unknown"
                    let ts_type = options.default_scalar_type.as_deref().unwrap_or("unknown");
                    writeln!(writer, "{export}type {type_name} = {ts_type};")?;
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
