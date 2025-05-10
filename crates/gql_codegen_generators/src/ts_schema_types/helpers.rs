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

// pub(crate) fn render_type(ty: &Type) -> String {
//     match ty {
//         Type::Named(name) => format!("{} | null | undefined", wrap_scalar_type(name)),
//         Type::NonNullNamed(name) => wrap_scalar_type(name).to_string(),
//         Type::List(inner) => {
//             format!("Array<{}> | null | undefined", render_type(inner))
//         }
//         Type::NonNullList(inner) => format!("Array<{}>", render_type(inner)),
//     }
// }
//
// pub(crate) fn wrap_scalar_type(name: &str) -> String {
//     let is_scalar = self.schema.get_scalar(name).is_some();
//     if is_scalar {
//         return format!("Scalars['{name}']");
//     }
//
//     name.to_string()
// }
