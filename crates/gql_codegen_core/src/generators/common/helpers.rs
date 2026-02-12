use std::borrow::Cow;

use apollo_compiler::Name;
use apollo_compiler::ast::{FieldDefinition, InputValueDefinition, Type};
use apollo_compiler::collections::IndexSet;
use apollo_compiler::schema::{Component, ComponentName};

use crate::config::ScalarConfig;
use crate::generators::GeneratorContext;
use crate::{DeclarationKind, Result};

pub(crate) fn indent(ctx: &mut GeneratorContext, depth: usize) -> Result<()> {
    let indent = "  ".repeat(depth);
    write!(ctx.writer, "{indent}")?;
    Ok(())
}

pub(crate) fn get_export_kw(ctx: &GeneratorContext) -> &'static str {
    if ctx.options.no_export { "" } else { "export " }
}

pub(crate) fn get_readonly_kw(ctx: &GeneratorContext) -> &'static str {
    if ctx.options.immutable_types {
        "readonly "
    } else {
        ""
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FieldType<'a> {
    Object(&'a Component<FieldDefinition>),
    InputObject(&'a InputValueDefinition),
}

impl FieldType<'_> {
    pub fn direction(&self) -> ScalarDirection {
        match self {
            FieldType::Object(_) => ScalarDirection::Output,
            FieldType::InputObject(_) => ScalarDirection::Input,
        }
    }
}

/// Lightweight enum for scalar rendering direction.
/// Use this when you only need input/output context without a full field definition
/// (e.g., in the operations plugin after normalization).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScalarDirection {
    Input,
    Output,
}

pub(crate) fn get_optional_prop_modifier(
    ctx: &GeneratorContext,
    field_type: &FieldType,
) -> &'static str {
    let normalized = ctx.options.avoid_optionals.normalize();

    let avoid = match field_type {
        FieldType::Object(field) => field.ty.is_non_null(),
        FieldType::InputObject(field) => {
            let has_default = field.default_value.is_some();

            if field.ty.is_non_null() {
                !has_default || normalized.default_value || normalized.input_value
            } else {
                normalized.input_value
            }
        }
    };

    if avoid { "" } else { "?" }
}

/// Unwrap NonNull/List wrappers to get the inner named type.
pub(crate) fn unwrap_type_name(ty: &Type) -> Name {
    match ty {
        Type::Named(name) | Type::NonNullNamed(name) => name.clone(),
        Type::List(inner) | Type::NonNullList(inner) => unwrap_type_name(inner),
    }
}

pub(crate) fn wrap_maybe(ctx: &GeneratorContext, value: &str, dir: ScalarDirection) -> String {
    if ctx.options.use_utility_types {
        match dir {
            ScalarDirection::Output => format!("Maybe<{value}>"),
            ScalarDirection::Input => format!("InputMaybe<{value}>"),
        }
    } else {
        format!("{value} | null")
    }
}

/// Recursively render a type, handling nullability at each level
pub(crate) fn render_type(ctx: &GeneratorContext, ty: &Type, dir: ScalarDirection) -> String {
    let array_type = get_array_type(ctx);

    match ty {
        Type::Named(name) => {
            let field = render_field_type(ctx, name, dir);
            wrap_maybe(ctx, &field, dir)
        }
        Type::NonNullNamed(name) => render_field_type(ctx, name, dir).into_owned(),
        Type::List(inner) => {
            let inner_type = render_type(ctx, inner.as_ref(), dir);
            wrap_maybe(ctx, &format!("{array_type}<{inner_type}>"), dir)
        }
        Type::NonNullList(inner) => {
            let inner_type = render_type(ctx, inner.as_ref(), dir);
            format!("{array_type}<{inner_type}>")
        }
    }
}

/// Gets the array type for a field base on immutable types option
/// immutable_types == true -> ReadonlyArray
/// immutable_types == false -> Array
pub(crate) fn get_array_type(ctx: &GeneratorContext) -> &'static str {
    if ctx.options.immutable_types {
        "ReadonlyArray"
    } else {
        "Array"
    }
}

/// Map built-in GraphQL scalar types to TypeScript types.
/// Returns None for custom scalars that need separate handling.
pub(crate) fn gql_scalar_to_ts(name: &str) -> Option<&'static str> {
    match name {
        "String" | "ID" => Some("string"),
        "Int" | "Float" => Some("number"),
        "Boolean" => Some("boolean"),
        _ => None,
    }
}

pub(crate) fn render_field_type(
    ctx: &GeneratorContext,
    name: &Name,
    dir: ScalarDirection,
) -> Cow<'static, str> {
    let name_str = name.as_str();

    let scalar_type = match dir {
        ScalarDirection::Output => "output",
        ScalarDirection::Input => "input",
    };

    if ctx.schema.get_scalar(name).is_some() {
        if ctx.options.use_utility_types {
            return Cow::Owned(format!("Scalars['{name_str}']['{scalar_type}']"));
        }

        if let Some(mapped) = ctx.options.scalars.get(name_str) {
            match mapped {
                ScalarConfig::Simple(value) => return Cow::Owned(value.clone()),
                ScalarConfig::Detailed { input: _, output } => return Cow::Owned(output.clone()),
            }
        }

        if let Some(ts_type) = gql_scalar_to_ts(name_str) {
            return Cow::Borrowed(ts_type);
        }

        return ctx
            .options
            .default_scalar_type
            .as_ref()
            .map(|s| Cow::Owned(s.clone()))
            .unwrap_or(Cow::Borrowed("unknown"));
    }

    Cow::Owned(ctx.transform_type_name(name_str).into_owned())
}

/// TODO: make this less strict, allowing strings which we parse or fallback to default
pub(crate) fn get_decl_kind_kw(ctx: &GeneratorContext) -> &'static str {
    match ctx.options.declaration_kind {
        Some(DeclarationKind::Type) | None => "type",
        Some(DeclarationKind::Interface) => "interface",
        Some(DeclarationKind::Class) => "class",
        Some(DeclarationKind::AbstractClass) => "abstract class",
    }
}

/// Renders the declaration prefix: `{export}{decl_kind} {name}{separator}`
/// without the opening brace. This allows callers to compose differently:
/// - Schema types: prefix + `{` (via render_decl_opening)
/// - Operations: prefix + inline object from render_normalized
pub(crate) fn render_decl_prefix(
    ctx: &mut GeneratorContext,
    name: &str,
    implements_interfaces: Option<&IndexSet<ComponentName>>,
) -> Result<()> {
    let export = get_export_kw(ctx);
    let decl_kind = get_decl_kind_kw(ctx);

    let separator = match ctx.options.declaration_kind {
        Some(DeclarationKind::Type) | None => " = ",
        _ => " ",
    };

    write!(ctx.writer, "{export}{decl_kind} {name}{separator}")?;

    if let Some(interfaces) = implements_interfaces {
        if !interfaces.is_empty() {
            match ctx.options.declaration_kind {
                Some(DeclarationKind::Type) | None => {
                    for interface in interfaces {
                        write!(ctx.writer, "{interface}")?;
                        write!(ctx.writer, " & ")?;
                    }
                }
                _ => {
                    write!(ctx.writer, "implements ")?;

                    // TODO: transform interface name
                    for (i, interface) in interfaces.iter().enumerate() {
                        write!(ctx.writer, "{interface}")?;
                        if i < interfaces.len() - 1 {
                            write!(ctx.writer, ", ")?;
                        }
                    }

                    write!(ctx.writer, " ")?;
                }
            }
        }
    }

    Ok(())
}

pub(crate) fn render_decl_opening(
    ctx: &mut GeneratorContext,
    name: &str,
    implements_interfaces: Option<&IndexSet<ComponentName>>,
) -> Result<()> {
    render_decl_prefix(ctx, name, implements_interfaces)?;
    writeln!(ctx.writer, "{{")?;
    Ok(())
}

pub(crate) fn render_decl_closing(ctx: &mut GeneratorContext) -> Result<()> {
    let semi = match ctx.options.declaration_kind {
        Some(DeclarationKind::Type) | None => ";",
        _ => "",
    };

    writeln!(ctx.writer, "}}{semi}")?;
    Ok(())
}
