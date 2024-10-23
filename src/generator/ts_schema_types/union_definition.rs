use apollo_compiler::{hir::TypeExtension, HirDatabase, RootDatabase};
use apollo_parser::cst::UnionTypeDefinition;

pub fn render_union_definition(
    definition: &UnionTypeDefinition,
    db: &RootDatabase,
) -> Option<String> {
    let mut output = String::new();

    let name = definition.name()?.text();
    let types_definition = definition.union_member_types()?;

    output.push_str(&format!("export type {} = ", name));

    let mut types: Vec<String> = vec![];

    for member in types_definition.named_types() {
        types.push(member.name()?.text().to_string());
    }

    for extension in db.extensions().as_ref() {
        if let TypeExtension::UnionTypeExtension(extension) = extension {
            if extension.name() == name {
                for member in extension.members() {
                    types.push(member.name().to_string());
                }
            }
        }
    }

    output.push_str(&types.join(" | "));
    output.push_str(";\n\n");

    Some(output)
}
