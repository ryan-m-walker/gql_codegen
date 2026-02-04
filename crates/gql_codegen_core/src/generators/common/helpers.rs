use crate::generators::GeneratorContext;

pub(crate) fn get_readonly_kw(ctx: &GeneratorContext) -> &'static str {
    if ctx.options.immutable_types {
        "readonly "
    } else {
        ""
    }
}

pub(crate) fn get_optional_prop_modifier(ctx: &GeneratorContext) -> &'static str {
    if ctx.options.avoid_optionals { "" } else { "?" }
}
