use apollo_compiler::Name;
use apollo_compiler::ast::{Selection, Type};
use indexmap::IndexMap;

use crate::Result;
use crate::config::TypenamePolicy;
use crate::generators::GeneratorContext;
use crate::generators::common::helpers::{ScalarDirection, get_readonly_kw, indent, render_type};
use crate::generators::typescript_operations::typename::render_op_typename;

#[derive(Debug, Clone)]
pub(crate) struct NormalizedSelection {
    /// The actual GraphQL field name (not alias — needed for schema lookups)
    pub field_name: Name,
    /// The resolved type from the schema field definition
    pub field_type: Type,
    /// The parent type this field belongs to (for __typename literal values)
    pub parent_type: Name,
    /// Whether @include or @skip directives are present (makes field optional)
    pub has_conditional: bool,
    /// Whether this __typename was explicitly selected in the query
    pub explicitly_selected: bool,
    /// Merged sub-selections for nested object types
    pub children: NormalizedSelectionSet,
}

/// Accumulates and deduplicates fields within a single selection set level.
/// Uses IndexMap to preserve insertion order (deterministic output).
#[derive(Debug, Clone)]
pub(crate) struct NormalizedSelectionSet {
    pub fields: IndexMap<String, NormalizedSelection>,
}

impl NormalizedSelectionSet {
    pub fn new() -> Self {
        Self {
            fields: IndexMap::new(),
        }
    }
}

// TODO: null after selection set

/// Pass 1: Walk the AST selection set and build a normalized, deduplicated tree.
///
/// When the same field appears multiple times (directly or via fragments),
/// their sub-selections are merged into a single `NormalizedSelection` entry.
pub(crate) fn collect_selection_set(
    ctx: &GeneratorContext,
    selections: &[Selection],
    parent_type: &Name,
    normalized: &mut NormalizedSelectionSet,
) -> Result<()> {
    let typename_policy = ctx.options.resolved_typename_policy();

    for selection in selections {
        match selection {
            Selection::Field(field) => {
                let response_name = field.alias.as_ref().unwrap_or(&field.name);

                // __typename is a meta-field not in the schema — handle specially
                if field.name == "__typename" {
                    if typename_policy == TypenamePolicy::Skip {
                        continue;
                    }

                    normalized
                        .fields
                        .entry(response_name.to_string())
                        .or_insert_with(|| NormalizedSelection {
                            field_name: field.name.clone(),
                            field_type: Type::NonNullNamed(field.name.clone()),
                            parent_type: parent_type.clone(),
                            has_conditional: false,
                            explicitly_selected: true,
                            children: NormalizedSelectionSet::new(),
                        });

                    continue;
                }

                let Ok(type_field) = ctx.schema.type_field(parent_type, &field.name) else {
                    continue;
                };

                let has_conditional =
                    field.directives.has("skip") || field.directives.has("include");

                // Insert if new, or get existing entry for merging
                let entry = normalized
                    .fields
                    .entry(response_name.to_string())
                    .or_insert_with(|| NormalizedSelection {
                        field_name: field.name.clone(),
                        field_type: type_field.ty.clone(),
                        parent_type: parent_type.clone(),
                        has_conditional,
                        explicitly_selected: false,
                        children: NormalizedSelectionSet::new(),
                    });

                // Sticky conditional: if any occurrence has @skip/@include, field is optional
                if has_conditional {
                    entry.has_conditional = true;
                }

                // Recurse into children — merges sub-selections from duplicate fields
                if !field.selection_set.is_empty() {
                    collect_selection_set(
                        ctx,
                        &field.selection_set,
                        type_field.ty.inner_named_type(),
                        &mut entry.children,
                    )?;
                }
            }

            Selection::FragmentSpread(spread) => {
                if let Some(fragment) = ctx.fragments.get(&spread.fragment_name) {
                    collect_selection_set(
                        ctx,
                        &fragment.definition.selection_set,
                        &fragment.definition.type_condition,
                        normalized,
                    )?;
                }
            }

            Selection::InlineFragment(inline) => {
                let type_name = inline.type_condition.as_ref().unwrap_or(parent_type);
                collect_selection_set(ctx, &inline.selection_set, type_name, normalized)?;
            }
        }
    }

    // In Always mode, inject __typename at the top if not explicitly selected
    if typename_policy == TypenamePolicy::Always && !normalized.fields.contains_key("__typename") {
        let typename_name = Name::new("__typename").unwrap();
        normalized.fields.shift_insert(
            0,
            "__typename".to_string(),
            NormalizedSelection {
                field_name: typename_name.clone(),
                field_type: Type::NonNullNamed(typename_name),
                parent_type: parent_type.clone(),
                has_conditional: false,
                explicitly_selected: false,
                children: NormalizedSelectionSet::new(),
            },
        );
    }

    Ok(())
}

/// Pass 2: Render the normalized tree as TypeScript.
pub(crate) fn render_normalized(
    ctx: &mut GeneratorContext,
    normalized: &NormalizedSelectionSet,
    depth: usize,
) -> Result<()> {
    writeln!(ctx.writer, "{{")?;

    let readonly = get_readonly_kw(ctx);
    let dir = ScalarDirection::Output;
    let avoid_optionals = ctx.options.avoid_optionals.normalize();

    for (response_name, field) in &normalized.fields {
        // __typename: emit string literal type based on policy
        if field.field_name == "__typename" {
            render_op_typename(ctx, response_name, &field.parent_type, depth + 1)?;
            continue;
        }

        indent(ctx, depth + 1)?;

        // Respect avoid_optionals.field — when true, output fields never get `?`
        let is_nullable = !field.field_type.is_non_null() || field.has_conditional;
        let optional = if is_nullable && !avoid_optionals.field {
            "?"
        } else {
            ""
        };

        write!(ctx.writer, "{readonly}{response_name}{optional}: ")?;

        if field.children.fields.is_empty() {
            // Leaf field — render the TypeScript type directly
            let ts_type = render_type(ctx, &field.field_type, dir);
            writeln!(ctx.writer, "{ts_type};")?;
        } else {
            // Nested object — recurse into children
            render_normalized(ctx, &field.children, depth + 1)?;
        }
    }

    indent(ctx, depth)?;
    writeln!(ctx.writer, "}};")?;

    Ok(())
}
