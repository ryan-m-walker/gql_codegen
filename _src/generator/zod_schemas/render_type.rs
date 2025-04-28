use apollo_compiler::hir::Type;

pub fn render_type(ty: &Type, non_null: bool) -> String {
    render_wrapped_type(ty, non_null, render_scalar)
}

pub fn render_scalar(value: &str) -> String {
    match value {
        "ID" => String::from("z.string()"),
        "String" => String::from("z.string()"),
        "Boolean" => String::from("z.boolean()"),
        "Int" => String::from("z.number().int()"),
        "Float" => String::from("z.number()"),
        _ => format!("{}Schema", value).to_string(),
    }
}

pub fn render_wrapped_type<F>(ty: &Type, non_null: bool, type_renderer: F) -> String
where
    F: Fn(&str) -> String,
{
    match ty {
        Type::Named { name, loc: _ } => {
            let text = type_renderer(name);

            if non_null {
                return text;
            }

            return format!("{}.nullish()", text);
        }
        Type::NonNull { ty, loc: _ } => {
            return render_wrapped_type(ty, true, type_renderer);
        }
        Type::List { ty, loc: _ } => {
            if non_null {
                return format!("{}.array()", render_wrapped_type(ty, false, type_renderer));
            }

            return format!(
                "{}.array().nullish()",
                render_wrapped_type(ty, false, type_renderer)
            );
        }
    }
}
