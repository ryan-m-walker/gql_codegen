use apollo_compiler::hir::UnionTypeDefinition;

pub fn render_union_definition(definition: &UnionTypeDefinition) -> String {
    let mut output = String::new();

    let name = definition.name();
    let members = definition.members();

    output.push_str(&format!("export type {} = ", name));

    let mut types: Vec<String> = vec![];

    for member in members {
        types.push(member.name().to_string());
    }

    output.push_str(&types.join(" | "));
    output.push_str(";\n\n");

    output
}
