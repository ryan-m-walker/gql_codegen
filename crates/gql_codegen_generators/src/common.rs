pub(crate) fn gql_scalar_to_ts_scalar(gql_type: &str) -> &'static str {
    match gql_type {
        "String" => "string",
        "Int" => "number",
        "Float" => "number",
        "Boolean" => "boolean",
        "ID" => "string",
        _ => "unknown",
    }
}
