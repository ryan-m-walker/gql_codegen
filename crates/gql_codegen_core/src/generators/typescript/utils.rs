use crate::Result;
use crate::generators::GeneratorContext;
use crate::generators::typescript::helpers::get_export_kw;

const DEFAULT_MAYBE_VALUE: &str = "T | null";
const DEFAULT_INPUT_MAYBE_VALUE: &str = "Maybe<T>";

const DEFAULT_BASE_TYPES: [&str; 5] = [
    "type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };",
    "type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };",
    "type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };",
    "type MakeEmpty<T extends { [key: string]: unknown }, K extends keyof T> = { [_ in K]?: never };",
    "type Incremental<T> = T | { [P in keyof T]?: P extends ' $fragmentName' | '__typename' ? T[P] : never };",
];

pub(crate) fn generate_util_types(ctx: &mut GeneratorContext) -> Result<()> {
    let export = get_export_kw(ctx);

    let maybe_value = ctx
        .options
        .maybe_value
        .as_deref()
        .unwrap_or(DEFAULT_MAYBE_VALUE);

    let input_maybe_value = if let Some(ref input_maybe) = ctx.options.input_maybe_value {
        input_maybe
    } else if let Some(ref maybe) = ctx.options.maybe_value {
        maybe
    } else {
        DEFAULT_INPUT_MAYBE_VALUE
    };

    writeln!(ctx.writer, "{export}type Maybe<T> = {maybe_value};",)?;
    writeln!(
        ctx.writer,
        "{export}type InputMaybe<T> = {input_maybe_value};",
    )?;

    for base_type in DEFAULT_BASE_TYPES {
        writeln!(ctx.writer, "{export}{base_type}")?;
    }

    Ok(())
}
