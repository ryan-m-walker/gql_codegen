use std::borrow::Cow;

use apollo_compiler::Name;
use apollo_compiler::schema::Type;

use crate::Result;
use crate::config::ScalarConfig;
use crate::generators::GeneratorContext;
use crate::generators::common::helpers::{FieldType, get_optional_prop_modifier};
use crate::generators::typescript::helpers::{
    get_readonly_kw, gql_scalar_to_ts, render_description,
};

/// Gets the array type for a field base on immutable types option
/// immutable_types == true -> ReadonlyArray
/// immutable_types == false -> Array
fn get_array_type(ctx: &GeneratorContext) -> &'static str {
    if ctx.options.immutable_types {
        "ReadonlyArray"
    } else {
        "Array"
    }
}

pub(crate) fn render_field(
    ctx: &mut GeneratorContext,
    name: &Name,
    field_type: &FieldType,
) -> Result<()> {
    let readonly = get_readonly_kw(ctx);
    let optional_field = get_optional_prop_modifier(ctx, field_type);
    let array_type = get_array_type(ctx);

    let (description, ty) = match field_type {
        FieldType::InputObject(field) => (&field.description, field.ty.as_ref()),
        FieldType::Object(field) => (&field.description, &field.ty),
    };

    render_description(ctx, description, 1)?;
    write!(ctx.writer, "  {readonly}{name}")?;

    match &ty {
        Type::Named(name) => {
            let field = render_field_type(ctx, name, field_type);
            let maybe_value = wrap_maybe(ctx, field.as_ref(), field_type);
            writeln!(ctx.writer, "{optional_field}: {maybe_value};")?;
        }
        Type::NonNullNamed(name) => {
            let field_type = render_field_type(ctx, name, field_type);
            writeln!(ctx.writer, "{optional_field}: {field_type};")?;
        }
        Type::List(inner) => {
            let inner_type = render_type(ctx, inner.as_ref(), field_type);
            let array = wrap_maybe(ctx, &format!("{array_type}<{inner_type}>"), field_type);
            writeln!(ctx.writer, "{optional_field}: {array};")?;
        }
        Type::NonNullList(inner) => {
            let inner_type = render_type(ctx, inner.as_ref(), field_type);
            writeln!(ctx.writer, "{optional_field}: {array_type}<{inner_type}>;")?;
        }
    }

    Ok(())
}

fn wrap_maybe(ctx: &GeneratorContext, value: &str, field_type: &FieldType) -> String {
    if ctx.options.use_utility_types {
        match field_type {
            FieldType::Object(_) => format!("Maybe<{value}>"),
            FieldType::InputObject(_) => format!("InputMaybe<{value}>"),
        }
    } else {
        format!("{value} | null")
    }
}

fn render_field_type(
    ctx: &GeneratorContext,
    name: &Name,
    field_type: &FieldType,
) -> Cow<'static, str> {
    let name_str = name.as_str();

    let scalar_type = match field_type {
        FieldType::Object(_) => "output",
        FieldType::InputObject(_) => "input",
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

/// Recursively render a type, handling nullability at each level
fn render_type(ctx: &GeneratorContext, ty: &Type, field_type: &FieldType) -> String {
    let array_type = get_array_type(ctx);

    match ty {
        Type::Named(name) => {
            let field = render_field_type(ctx, name, field_type);
            wrap_maybe(ctx, &field, field_type)
        }
        Type::NonNullNamed(name) => render_field_type(ctx, name, field_type).into_owned(),
        Type::List(inner) => {
            let inner_type = render_type(ctx, inner.as_ref(), field_type);
            wrap_maybe(ctx, &format!("{array_type}<{inner_type}>"), field_type)
        }
        Type::NonNullList(inner) => {
            let inner_type = render_type(ctx, inner.as_ref(), field_type);
            format!("{array_type}<{inner_type}>")
        }
    }
}
