use std::{borrow::Cow, io::Write};

use apollo_compiler::{
    Name,
    ast::FieldDefinition,
    schema::{Component, Type},
};

use crate::{
    Result,
    generators::{
        GeneratorContext,
        typescript::helpers::{
            get_optional_prop_modifier, get_readonly_kw, gql_scalar_to_ts, render_description,
        },
    },
};

pub(crate) fn render_field(
    name: &Name,
    field: &Component<FieldDefinition>,
    ctx: &GeneratorContext,
    writer: &mut dyn Write,
) -> Result<()> {
    let readonly = get_readonly_kw(ctx);
    let optional_field = get_optional_prop_modifier(ctx);

    let array_type = if ctx.options.immutable_types {
        "ReadonlyArray"
    } else {
        "Array"
    };

    render_description(&field.description, 1, writer)?;
    write!(writer, "  {readonly}{name}")?;

    match &field.ty {
        Type::Named(name) => {
            write!(writer, "{optional_field}: ")?;
            let field_type = render_field_type(name, ctx);
            let maybe_value = wrap_maybe(field_type.as_ref(), ctx);
            writeln!(writer, "{maybe_value};")?;
        }
        Type::NonNullNamed(name) => {
            write!(writer, ": ")?;
            let field_type = render_field_type(name, ctx);
            write!(writer, "{field_type};")?;
            writeln!(writer)?;
        }
        Type::List(inner) => {
            // Nullable list - wrap the array, and handle inner type nullability
            let inner_type = render_type(inner.as_ref(), array_type, ctx);
            let array = wrap_maybe(&format!("{array_type}<{inner_type}>"), ctx);
            writeln!(writer, "{optional_field}: {array};")?;
        }
        Type::NonNullList(inner) => {
            // Non-null list - don't wrap array, but inner items may still be nullable
            let inner_type = render_type(inner.as_ref(), array_type, ctx);
            writeln!(writer, ": {array_type}<{inner_type}>;")?;
        }
    }

    Ok(())
}

fn wrap_maybe(value: &str, ctx: &GeneratorContext) -> String {
    if ctx.options.use_utility_types {
        format!("Maybe<{value}>")
    } else {
        format!("{value} | null")
    }
}

fn render_field_type(name: &Name, ctx: &GeneratorContext) -> Cow<'static, str> {
    let name_str = name.as_str();

    if ctx.schema.get_scalar(name).is_some() {
        if ctx.options.use_utility_types {
            return Cow::Owned(format!("Scalars['{name_str}']['output']"));
        }

        if let Some(mapped) = ctx.options.scalars.get(name_str) {
            return Cow::Owned(mapped.clone());
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

    Cow::Owned(name_str.to_string())
}

/// Recursively render a type, handling nullability at each level
fn render_type(ty: &Type, array_type: &str, ctx: &GeneratorContext) -> String {
    match ty {
        Type::Named(name) => {
            let field_type = render_field_type(name, ctx);
            wrap_maybe(&field_type, ctx)
        }
        Type::NonNullNamed(name) => render_field_type(name, ctx).into_owned(),
        Type::List(inner) => {
            let inner_type = render_type(inner.as_ref(), array_type, ctx);
            wrap_maybe(&format!("{array_type}<{inner_type}>"), ctx)
        }
        Type::NonNullList(inner) => {
            let inner_type = render_type(inner.as_ref(), array_type, ctx);
            format!("{array_type}<{inner_type}>")
        }
    }
}
