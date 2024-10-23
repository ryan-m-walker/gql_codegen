use graphql_parser::schema::{Field, Text, Type};

pub fn generate_object_field<'a, T: Text<'a>>(field: Field<'a, T>) -> String {
    format!(
        "{}: {}",
        field.name.as_ref(),
        build_object_field(field.field_type, false, false)
    )
}

fn build_object_field<'a, T: Text<'a>>(
    gql_type: Type<'a, T>,
    is_list: bool,
    non_null: bool,
) -> String {
    return match gql_type {
        Type::ListType(inner) => build_object_field(*inner, true, non_null),
        Type::NonNullType(inner) => build_object_field(*inner, is_list, true),
        Type::NamedType(value) => match value.as_ref() {
            "String" => render_value("string", is_list, non_null),
            "Int" => render_value("number", is_list, non_null),
            "Float" => render_value("number", is_list, non_null),
            "Boolean" => render_value("boolean", is_list, non_null),
            "ID" => render_value("string", is_list, non_null),
            _ => render_value(value.as_ref(), is_list, non_null),
        },
    };
}

fn render_value(value: &str, is_list: bool, non_null: bool) -> String {
    if is_list && !non_null {
        return format!("({} | null)[]", value);
    }

    if is_list {
        return format!("{}[]", value);
    }

    if !non_null {
        return format!("{} | null", value);
    }

    value.to_string()
}
