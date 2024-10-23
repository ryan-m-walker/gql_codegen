use apollo_compiler::{HirDatabase, RootDatabase};
use apollo_parser::cst::EnumTypeDefinition;

pub fn render_enum_definition(
    definition: &EnumTypeDefinition,
    db: &RootDatabase,
) -> Option<String> {
    let mut output = String::new();

    let name = definition.name()?.text();
    let values_definition = definition.enum_values_definition()?;

    output.push_str(&format!("export type {} = ", name));

    let mut values: Vec<String> = vec![];

    for value in values_definition.enum_value_definitions() {
        let enum_value = value.enum_value()?.text();
        values.push(format!("'{}'", enum_value.to_string()));
    }

    for extension in db.extensions().as_ref() {
        if let apollo_compiler::hir::TypeExtension::EnumTypeExtension(extension) = extension {
            if extension.name() == name {
                for value in extension.values() {
                    values.push(format!("'{}'", value.enum_value()));
                }
            }
        }
    }

    values.push("'%future added value'".to_string());

    output.push_str(&values.join(" | "));
    output.push_str(";\n\n");

    Some(output)
}
