//! Operation type collection for `only_operation_types` feature.
//!
//! This module implements transitive type collection for the `only_operation_types` option.
//! When enabled, only types that are actually referenced by operations and fragments
//! are included in the generated output.
//!
//! ## How it differs from graphql-codegen
//!
//! The graphql-codegen typescript plugin's `only_operation_types` implementation simply
//! skips entire type categories (objects, interfaces, unions, inputs) and only keeps
//! enums and scalars. This has known issues:
//! - <https://github.com/dotansimha/graphql-code-generator/issues/9665>
//! - <https://github.com/dotansimha/graphql-code-generator/issues/4562>
//!
//! Our implementation does proper transitive closure analysis:
//! 1. Collect types directly referenced in operations and fragments
//! 2. Collect variable input types
//! 3. Transitively expand to include all field types
//! 4. Include union members and interface implementers

use std::collections::HashSet;

use apollo_compiler::ast::{Selection, Type};
use apollo_compiler::schema::ExtendedType;
use apollo_compiler::validation::Valid;
use apollo_compiler::{Name, Schema};

use super::GeneratorContext;
use crate::generators::common::helpers::unwrap_type_name;

/// Collect all types referenced in operations and fragments.
///
/// This performs a transitive closure over the type graph, starting from:
/// - Root operation types (Query, Mutation, Subscription)
/// - Operation variable types
/// - Fragment type conditions
///
/// And expanding to include:
/// - All field return types
/// - Union member types
/// - Types implementing referenced interfaces
pub fn collect_operation_types(ctx: &GeneratorContext) -> HashSet<String> {
    let mut referenced = HashSet::new();
    let schema = ctx.schema;

    // Collect from operations
    for (_, operation) in ctx.operations.iter() {
        // Collect variable types
        for var in &operation.definition.variables {
            collect_type_name(&var.ty, &mut referenced);
        }

        // Collect from selection set starting at root type
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
    expand_transitive_types(&mut referenced, schema);

    referenced
}

/// Collect types from a selection set using iterative traversal.
fn collect_from_selections(
    selections: &[Selection],
    parent_type: &str,
    schema: &Valid<Schema>,
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
                    // Fragment types are collected when we process fragments directly
                }
            }
        }
    }
}

/// Expand the set of referenced types to include all transitively reachable types.
///
/// This handles:
/// - Field return types (for objects, interfaces, input objects)
/// - Union members
/// - Types implementing referenced interfaces
fn expand_transitive_types(referenced: &mut HashSet<String>, schema: &Valid<Schema>) {
    let mut to_process: Vec<String> = referenced.iter().cloned().collect();

    while let Some(type_name) = to_process.pop() {
        if let Some(ty) = schema.types.get(type_name.as_str()) {
            // Collect field types
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

            // Collect types implementing this interface
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
}

/// Get the return type of a field from a parent type.
fn get_field_type<'a>(
    schema: &'a Valid<Schema>,
    parent_type: &str,
    field_name: &Name,
) -> Option<&'a Type> {
    let ty = schema.types.get(parent_type)?;
    match ty {
        ExtendedType::Object(obj) => obj.fields.get(field_name).map(|f| &f.ty),
        ExtendedType::Interface(iface) => iface.fields.get(field_name).map(|f| &f.ty),
        _ => None,
    }
}

/// Collect all field types from a type definition.
fn collect_field_types(ty: &ExtendedType, out: &mut HashSet<String>) {
    match ty {
        ExtendedType::Object(obj) => {
            for (_, field) in obj.fields.iter() {
                collect_type_name(&field.ty, out);
            }
        }
        ExtendedType::Interface(iface) => {
            for (_, field) in iface.fields.iter() {
                collect_type_name(&field.ty, out);
            }
        }
        ExtendedType::InputObject(input) => {
            for (_, field) in input.fields.iter() {
                collect_type_name(&field.ty, out);
            }
        }
        _ => {}
    }
}

/// Extract the type name from a Type, adding it to the output set.
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
