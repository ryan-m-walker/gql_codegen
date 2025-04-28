use apollo_compiler::hir::ObjectTypeDefinition;

use super::render_type::render_type;
use crate::generator::common::render_description_comment;

pub fn render_object_definition(definition: &ObjectTypeDefinition) -> String {
    let mut output = String::new();

    let name = definition.name();

    if let Some(description) = definition.description() {
        output.push_str(&render_description_comment(&description, 0));
    }

    output.push_str(&format!(
        "export const {}Schema: z.ZodType<{}> = z.object({{\n",
        name, name
    ));
    output.push_str(&format!(
        "  __typename: z.literal('{}').nullish(),\n",
        definition.name()
    ));

    for field in definition.fields() {
        let field_name = field.name();
        let field_type = field.ty();
        let rendered_type = render_type(&field_type, false);

        if let Some(description) = field.description() {
            output.push_str(&render_description_comment(&description, 1));
        }

        output.push_str(&format!("  {}: {},\n", field_name, rendered_type));
    }

    output.push_str("})");

    for interface in definition.implements_interfaces() {
        output.push_str(&format!(".merge({}Schema)", interface.interface()));
    }

    output.push_str(";\n\n");

    output
}
