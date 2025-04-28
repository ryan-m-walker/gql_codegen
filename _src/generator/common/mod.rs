use apollo_compiler::hir::Type;

pub fn render_type(ty: &Type, non_null: bool) -> String {
    render_wrapped_type(ty, non_null, render_scalar)
}

pub fn render_scalar(value: &str) -> String {
    match value {
        "ID" => String::from("string"),
        "String" => String::from("string"),
        "Boolean" => String::from("boolean"),
        "Int" => String::from("number"),
        "Float" => String::from("number"),
        _ => value.to_string(),
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

            return format!("{} | null", text);
        }
        Type::NonNull { ty, loc: _ } => {
            return render_wrapped_type(ty, true, type_renderer);
        }
        Type::List { ty, loc: _ } => {
            if non_null {
                return format!("Array<{}>", render_wrapped_type(ty, false, type_renderer));
            }

            return format!(
                "Array<{}> | null",
                render_wrapped_type(ty, false, type_renderer)
            );
        }
    }
}

pub fn render_description_comment(description: &str, indentation: usize) -> String {
    let indentation = "  ".repeat(indentation);

    let description_value = description
        .lines()
        .map(|line| format!("{indentation} * {}", line))
        .collect::<Vec<String>>()
        .join("\n");

    format!("{indentation}/**\n{description_value}\n{indentation} */\n")
}
