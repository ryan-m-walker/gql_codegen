use std::{borrow::Cow, io::Write};

use apollo_compiler::{
    Name,
    ast::FieldDefinition,
    schema::{Component, Type},
};

use crate::{
    Result,
    generators::{GeneratorContext, typescript::helpers::render_description},
};

pub(crate) fn render_field(
    name: &Name,
    field: &Component<FieldDefinition>,
    ctx: &GeneratorContext,
    writer: &mut dyn Write,
) -> Result<()> {
    let readonly = if ctx.options.immutable_types {
        "readonly "
    } else {
        ""
    };

    let optional_field = if ctx.options.avoid_optionals { "" } else { "?" };

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

fn render_field_type<'a>(name: &'a Name, ctx: &'a GeneratorContext) -> Cow<'a, str> {
    let scalar = ctx.schema.get_scalar(name);

    if let Some(scalar) = scalar {
        let scalar_name = scalar.name.as_str();

        if ctx.options.use_utility_types {
            return Cow::Owned(format!("Scalars['{scalar_name}']['output']"));
        }

        return Cow::Borrowed(scalar_name);
    }

    Cow::Borrowed(name.as_str())
}

/// Recursively render a type, handling nullability at each level
fn render_type(ty: &Type, array_type: &str, ctx: &GeneratorContext) -> String {
    match ty {
        Type::Named(name) => {
            // Nullable named type - wrap in Maybe
            let field_type = render_field_type(name, ctx);
            wrap_maybe(&field_type, ctx)
        }
        Type::NonNullNamed(name) => {
            // Non-null named type - no wrapping
            render_field_type(name, ctx).into_owned()
        }
        Type::List(inner) => {
            // Nullable list - wrap the array
            let inner_type = render_type(inner.as_ref(), array_type, ctx);
            wrap_maybe(&format!("{array_type}<{inner_type}>"), ctx)
        }
        Type::NonNullList(inner) => {
            // Non-null list - don't wrap array
            let inner_type = render_type(inner.as_ref(), array_type, ctx);
            format!("{array_type}<{inner_type}>")
        }
    }
}
