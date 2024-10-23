use apollo_compiler::{hir::TypeExtension, HirDatabase, RootDatabase};
use apollo_parser::cst::InterfaceTypeDefinition;

use super::helpers::render_type;

pub fn render_interface_definition(
    definition: &InterfaceTypeDefinition,
    db: &RootDatabase,
) -> Option<String> {
    let mut output = String::new();

    let name = definition.name()?.text();
    let fields_definition = definition.fields_definition()?;
    let interface_type = db.find_interface_by_name(name.to_string())?;

    output.push_str(&format!("export type {} = {{\n", name));

    for field in fields_definition.field_definitions() {
        let field_name = field.name()?.text();
        let field_type = interface_type.field(&field_name)?;
        let rendered_type = render_type(&field_type.ty(), false);

        output.push_str(&format!("  {}: {};\n", field_name, rendered_type));
    }

    for extension in db.extensions().as_ref() {
        if let TypeExtension::InterfaceTypeExtension(extension) = extension {
            if extension.name() == name {
                for field in extension.fields() {
                    let name = field.name();
                    let rendered_type = render_type(&field.ty(), false);
                    output.push_str(&format!("  {}: {};\n", name, rendered_type));
                }
            }
        }
    }

    output.push_str("};\n\n");

    Some(output)
}
