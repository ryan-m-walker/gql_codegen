use apollo_compiler::ast::Type;

pub(crate) fn type_to_type_name(ty: &Type) -> String {
    match ty {
        Type::Named(name) => name.to_string(),
        Type::List(list) => type_to_type_name(list),
        Type::NonNullList(list) => type_to_type_name(list),
        Type::NonNullNamed(name) => name.to_string(),
    }
}
