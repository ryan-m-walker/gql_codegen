use apollo_compiler::{hir::TypeExtension, HirDatabase, RootDatabase};
use apollo_parser::cst::{CstNode, InputObjectTypeDefinition};

use crate::generator::common::render_type;

use super::descriptions::render_description_comment;

pub fn render_input_object_definition(
    definition: &InputObjectTypeDefinition,
    db: &RootDatabase,
) -> Option<String> {
    let mut output = String::new();

    let name = definition.name()?.text();
    let fields_definition = definition.input_fields_definition()?;
    let input_object_type = db.find_input_object_by_name(name.to_string())?;

    if let Some(description) = definition.description() {
        let source_string = description.source_string();
        output.push_str(&render_description_comment(&source_string, 0));
    }

    output.push_str(&format!("export type {} = {{\n", &name));

    output.push_str(&format!("  __typename?: '{}';\n", input_object_type.name()));

    for field in fields_definition.input_value_definitions() {
        let field_name = field.name()?.text();
        let field_type = input_object_type.field(&field_name);
        let rendered_type = render_type(&field_type?.ty(), false);

        if let Some(description) = field.description() {
            let source_string = description.source_string();
            output.push_str(&render_description_comment(&source_string, 1));
        }

        output.push_str(&format!("  {}: {};\n", field_name, rendered_type));
    }

    for extension in db.extensions().as_ref() {
        if let TypeExtension::InputObjectTypeExtension(extension) = extension {
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
