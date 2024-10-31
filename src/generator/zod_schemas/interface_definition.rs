use apollo_compiler::hir::InterfaceTypeDefinition;

use super::render_type::render_type;

pub fn render_interface_definition(definition: &InterfaceTypeDefinition) -> String {
    let mut output = String::new();

    let name = definition.name();

    output.push_str(&format!("export const {}Schema = z.object({{\n", name));

    for field in definition.fields() {
        let field_name = field.name();
        let field_type = field.ty();
        let rendered_type = render_type(&field_type, false);

        output.push_str(&format!("  {}: {},\n", field_name, rendered_type));
    }

    output.push_str("});\n\n");

    output
}
