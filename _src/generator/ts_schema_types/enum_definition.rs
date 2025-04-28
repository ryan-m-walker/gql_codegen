use apollo_compiler::hir::EnumTypeDefinition;

pub fn render_enum_definition(definition: &EnumTypeDefinition) -> String {
    let mut output = String::new();

    let name = definition.name();

    output.push_str(&format!("export type {} = ", name));

    let mut values: Vec<String> = vec![];

    for value in definition.values() {
        values.push(format!("'{}'", value.enum_value()));
    }

    values.push("'%future added value'".to_string());

    output.push_str(&values.join(" | "));
    output.push_str(";\n\n");

    output
}
