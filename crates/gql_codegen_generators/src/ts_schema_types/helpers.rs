pub(crate) fn get_scalar_type(name: &str) -> String {
    match name {
        "ID" => "string".to_string(),
        "String" => "string".to_string(),
        "Int" => "number".to_string(),
        "Float" => "number".to_string(),
        "Boolean" => "boolean".to_string(),
        _ => "unknown".to_string(),
    }
}
