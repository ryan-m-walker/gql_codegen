use apollo_compiler::ast::{FieldDefinition, InputValueDefinition};
use apollo_compiler::schema::Component;

use crate::generators::GeneratorContext;

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
