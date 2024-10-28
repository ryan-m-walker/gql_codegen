use apollo_compiler::hir::Type;

pub fn render_type(ty: &Type, non_null: bool) -> String {
    match ty {
        Type::Named { name, loc: _ } => {
            let text = render_scalar(name);

            if non_null {
                return text;
            }

            return format!("{} | null", text);
        }
        Type::NonNull { ty, loc: _ } => {
            return render_type(ty, true);
        }
        Type::List { ty, loc: _ } => {
            if non_null {
                return format!("Array<{}>", render_type(ty, false));
            }

            return format!("Array<{}> | null", render_type(ty, false));
        }
    }
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
