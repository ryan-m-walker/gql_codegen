use apollo_compiler::hir::InputObjectTypeDefinition;

use super::render_type::render_type;

pub fn render_input_object_definition(definition: &InputObjectTypeDefinition) -> String {
    let mut output = String::new();

    let name = definition.name();

    // if let Some(description) = definition.description() {
    //     output.push_str(&render_description_comment(&description, 0));
    // }

    output.push_str(&format!(
        "export const {}InputSchema = z.object({{\n",
        &name
    ));

    for field in definition.fields() {
        let field_name = field.name();
        let field_type = field.ty();
        let rendered_type = render_type(&field_type, false);

        // if let Some(description) = field.description() {
        //     output.push_str(&render_description_comment(&description, 1));
        // }

        output.push_str(&format!("  {}: {},\n", field_name, rendered_type));
    }

    output.push_str("});\n\n");

    output
}
