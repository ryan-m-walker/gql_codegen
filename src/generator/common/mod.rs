use apollo_compiler::hir::Type;

pub fn render_wrapped_type(ty: &Type, value: String, non_null: bool) -> String {
    dbg!(&ty.name());
    match ty {
        Type::Named { name: _, loc: _ } => {
            if non_null {
                return value;
            }

            return format!("{} | null", value);
        }
        Type::NonNull { ty, loc: _ } => {
            return render_type(ty, true);
        }
        Type::List { ty, loc: _ } => {
            if non_null {
                return format!("Array<{}>", render_wrapped_type(ty, value, false));
            }

            return format!("Array<{}> | null", render_wrapped_type(ty, value, false));
        }
    }
}

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

pub struct UnwrapNamedType {
    pub name: String,
    pub is_list: bool,
    pub is_non_null: bool,
}

pub fn unwrap_named_type(ty: &Type) -> UnwrapNamedType {
    unwrap_type_internal(ty, false, false)
}

fn unwrap_type_internal(ty: &Type, is_list: bool, is_non_null: bool) -> UnwrapNamedType {
    match ty {
        Type::Named { name, loc: _ } => UnwrapNamedType {
            name: name.to_string(),
            is_list,
            is_non_null,
        },
        Type::NonNull { ty, loc: _ } => unwrap_type_internal(ty, is_list, true),
        Type::List { ty, loc: _ } => unwrap_type_internal(ty, true, is_non_null),
    }
}
