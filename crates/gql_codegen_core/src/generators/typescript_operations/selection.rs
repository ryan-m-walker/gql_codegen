use apollo_compiler::Name;
use apollo_compiler::ast::{Selection, Type};
use apollo_compiler::schema::ExtendedType;
use indexmap::IndexMap;

use crate::Result;
use crate::config::TypenamePolicy;
use crate::generators::GeneratorContext;
use crate::generators::common::helpers::{
    FieldType, ScalarDirection, get_array_type, get_optional_prop_modifier, get_readonly_kw,
    indent, render_decl_closing, render_type,
};
use crate::generators::typescript_operations::field::render_field;
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
    /// Merged sub-selections for nested object types
    pub children: NormalizedSelectionSet,
}

/// Accumulates and deduplicates fields within a single selection set level.
/// Uses IndexMap to preserve insertion order (deterministic output).
///
/// When a field returns a union or interface type, inline fragments with
/// type conditions populate `variants` instead of merging into `fields`.
/// Shared fields (selected outside any inline fragment) stay in `fields`.
#[derive(Debug, Clone)]
pub(crate) struct NormalizedSelectionSet {
    pub fields: IndexMap<String, NormalizedSelection>,
    /// Discriminated union variants keyed by concrete type name.
    /// Populated when parent is a union/interface with inline fragments.
    pub variants: IndexMap<Name, NormalizedSelectionSet>,
}

impl NormalizedSelectionSet {
    pub fn new() -> Self {
        Self {
            fields: IndexMap::new(),
            variants: IndexMap::new(),
        }
    }
}

/// Check if a type name corresponds to a union or interface in the schema.
fn is_abstract_type(ctx: &GeneratorContext, type_name: &Name) -> bool {
    matches!(
        ctx.schema.types.get(type_name.as_str()),
        Some(ExtendedType::Union(_) | ExtendedType::Interface(_))
    )
}

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
                    let frag_type = &fragment.definition.type_condition;

                    if frag_type != parent_type && is_abstract_type(ctx, parent_type) {
                        let variant = normalized
                            .variants
                            .entry(frag_type.clone())
                            .or_insert_with(NormalizedSelectionSet::new);

                        collect_selection_set(
                            ctx,
                            &fragment.definition.selection_set,
                            frag_type,
                            variant,
                        )?;
                    } else {
                        collect_selection_set(
                            ctx,
                            &fragment.definition.selection_set,
                            &fragment.definition.type_condition,
                            normalized,
                        )?;
                    }
                }
            }

            Selection::InlineFragment(inline) => {
                let type_name = inline.type_condition.as_ref().unwrap_or(parent_type);

                // No type condition or same as parent → merge flat (e.g. directive grouping)
                if inline.type_condition.is_none() || type_name == parent_type {
                    collect_selection_set(ctx, &inline.selection_set, type_name, normalized)?;
                } else if is_abstract_type(ctx, parent_type) {
                    let variant = normalized
                        .variants
                        .entry(type_name.clone())
                        .or_insert_with(NormalizedSelectionSet::new);

                    collect_selection_set(ctx, &inline.selection_set, type_name, variant)?;
                } else {
                    // Not abstract — merge directly (current behavior)
                    collect_selection_set(ctx, &inline.selection_set, type_name, normalized)?;
                }
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
                children: NormalizedSelectionSet::new(),
            },
        );
    }

    // Inject __typename into each variant too (Always mode)
    if typename_policy == TypenamePolicy::Always {
        for (variant_type, variant_fields) in &mut normalized.variants {
            if !variant_fields.fields.contains_key("__typename") {
                let typename_name = Name::new("__typename").unwrap();
                variant_fields.fields.shift_insert(
                    0,
                    "__typename".to_string(),
                    NormalizedSelection {
                        field_name: typename_name.clone(),
                        field_type: Type::NonNullNamed(typename_name),
                        parent_type: variant_type.clone(),
                        has_conditional: false,
                        children: NormalizedSelectionSet::new(),
                    },
                );
            }
        }
    }

    Ok(())
}

/// Pass 2: Render the normalized tree as TypeScript.
pub(crate) fn render_normalized(
    ctx: &mut GeneratorContext,
    normalized: &NormalizedSelectionSet,
    depth: usize,
) -> Result<()> {
    // writeln!(ctx.writer, "{{")?;

    for (response_name, field) in &normalized.fields {
        // __typename: emit string literal type based on policy
        if field.field_name == "__typename" {
            render_op_typename(ctx, response_name, &field.parent_type, depth + 1)?;
            continue;
        }

        render_field(ctx, response_name, field, depth + 1)?;
    }

    indent(ctx, depth)?;

    // if depth == 0 {
    //     render_decl_closing(ctx)?;
    // } else {
    //     writeln!(ctx.writer, "}};")?;
    // }

    Ok(())
}

/// Render a single field entry: indentation, name, optionality, and value.
/// Shared between `render_normalized` and `render_variants`.
fn _render_field(
    ctx: &mut GeneratorContext,
    response_name: &str,
    field: &NormalizedSelection,
    depth: usize,
) -> Result<()> {
    let readonly = get_readonly_kw(ctx);
    let avoid_optionals = ctx.options.avoid_optionals.normalize();

    indent(ctx, depth)?;

    let is_nullable = !field.field_type.is_non_null() || field.has_conditional;

    let optional = if is_nullable && !avoid_optionals.field {
        "?"
    } else {
        ""
    };

    write!(ctx.writer, "{readonly}{response_name}{optional}: ")?;

    // if !field.children.variants.is_empty() {
    //     return render_variant_field(ctx, &field.children, &field.field_type, depth);
    // }
    //
    // if field.children.fields.is_empty() {
    //     let ts_type = render_type(ctx, &field.field_type, ScalarDirection::Output);
    //     writeln!(ctx.writer, "{ts_type};")?;
    //     return Ok(());
    // }

    // TODO: this doesn't work so right
    // Wrap in array layers (e.g. `ReadonlyArray<`) for list types
    // let nullable = !field.field_type.is_non_null();
    // let list_depth = write_list_open(ctx, &field.field_type)?;
    //
    // render_normalized(ctx, &field.children, depth + list_depth)?;

    // if list_depth > 0 {
    //     // Close list layers with nullability
    //     for i in (0..list_depth).rev() {
    //         indent(ctx, depth + i)?;
    //         write!(ctx.writer, ">")?;
    //         if !is_non_null_at_depth(&field.field_type, i) {
    //             write!(ctx.writer, " | null")?;
    //         }
    //         if i == 0 {
    //             writeln!(ctx.writer, ";")?;
    //         } else {
    //             writeln!(ctx.writer)?;
    //         }
    //     }
    // } else if nullable {
    // Non-list nullable: already closed by render_normalized's `};`, need ` | null`
    // Actually render_normalized writes `};` — we need to adjust
    // }

    Ok(())
}

/// Render a field whose children contain union/interface variants.
/// Handles Array/null wrapping around the inline discriminated union.
fn render_variant_field(
    ctx: &mut GeneratorContext,
    children: &NormalizedSelectionSet,
    field_type: &Type,
    depth: usize,
) -> Result<()> {
    let nullable = !field_type.is_non_null();

    // Unwrap list layers inline (e.g. `ReadonlyArray<\n`)
    let list_depth = write_list_open(ctx, field_type)?;
    let variant_depth = depth + list_depth + 1;

    if list_depth == 0 {
        writeln!(ctx.writer)?;
    }

    // Render the union variants
    render_variants(ctx, children, variant_depth)?;

    if list_depth > 0 {
        // Nullable inner type gets | null before closing >
        if nullable {
            indent(ctx, variant_depth)?;
            writeln!(ctx.writer, "| null")?;
        }

        // Close list layers: >; or > | null
        for i in (0..list_depth).rev() {
            indent(ctx, depth + i)?;
            write!(ctx.writer, ">")?;
            if !is_non_null_at_depth(field_type, i) {
                write!(ctx.writer, " | null")?;
            }
            if i == 0 {
                writeln!(ctx.writer, ";")?;
            } else {
                writeln!(ctx.writer)?;
            }
        }
    } else if nullable {
        indent(ctx, variant_depth)?;
        writeln!(ctx.writer, "| null;")?;
    } else {
        indent(ctx, depth)?;
        writeln!(ctx.writer, ";")?;
    }

    Ok(())
}

/// Write `Array<` or `ReadonlyArray<` for each list layer, inline after `: `.
/// Returns the number of list layers written.
fn write_list_open(ctx: &mut GeneratorContext, ty: &Type) -> Result<usize> {
    match ty {
        Type::NonNullList(inner) | Type::List(inner) => {
            let array_type = get_array_type(ctx);
            write!(ctx.writer, "{array_type}<")?;
            let layers = write_list_open(ctx, inner)?;
            Ok(layers)
        }
        _ => Ok(0),
    }
}

/// Check if the type at a given list nesting depth is non-null.
fn is_non_null_at_depth(ty: &Type, target_depth: usize) -> bool {
    if target_depth == 0 {
        return ty.is_non_null();
    }
    match ty {
        Type::NonNullList(inner) | Type::List(inner) => {
            is_non_null_at_depth(inner, target_depth - 1)
        }
        _ => ty.is_non_null(),
    }
}

/// Render the discriminated union variants.
///
/// Output format (for each variant):
/// ```text
///   | { __typename?: 'Book'; shared_field: T; book_field: U }
///   | { __typename?: 'Movie'; shared_field: T; movie_field: V }
/// ```
pub(crate) fn render_variants(
    ctx: &mut GeneratorContext,
    selection_set: &NormalizedSelectionSet,
    depth: usize,
) -> Result<()> {
    for (type_name, variant) in &selection_set.variants {
        indent(ctx, depth)?;
        writeln!(ctx.writer, "| {{")?;

        // __typename for this variant
        render_op_typename(ctx, "__typename", type_name, depth + 1)?;

        // Shared fields (from parent's fields, skip __typename — already rendered above)
        for (name, field) in &selection_set.fields {
            if field.field_name == "__typename" {
                continue;
            }
            render_field(ctx, name, field, depth + 1)?;
        }

        // Variant-specific fields
        for (name, field) in &variant.fields {
            if field.field_name == "__typename" {
                continue;
            }
            render_field(ctx, name, field, depth + 1)?;
        }

        indent(ctx, depth)?;
        writeln!(ctx.writer, "}}")?;
    }

    if ctx.options.future_proof_unions {
        let readonly = get_readonly_kw(ctx);
        indent(ctx, depth)?;
        write!(ctx.writer, "| {{ {readonly}__typename?: '%other' }}")?;
    }

    Ok(())
}
