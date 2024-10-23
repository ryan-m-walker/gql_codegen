use graphql_parser::schema::{Definition, Document, TypeDefinition};

use crate::helpers::generate_object_field;

pub fn generate<'a>(document: &Document<'a, &'a str>) -> String {
    let mut output = String::new();

    for definition in &document.definitions {
        match definition {
            Definition::TypeDefinition(TypeDefinition::Enum(definition)) => {
                if let Some(description) = &definition.description {
                    output.push_str(&format!("/**\n * {}\n */\n", description.trim()));
                }

                output.push_str(&format!("export type {} = ", definition.name));

                let values = definition
                    .values
                    .iter()
                    .map(|value| format!("\"{}\"", value.name.to_string()))
                    .collect::<Vec<String>>()
                    .join(" | ");

                output.push_str(&values);
                output.push_str(";\n\n");
            }

            Definition::TypeDefinition(TypeDefinition::Union(definition)) => {
                if let Some(description) = &definition.description {
                    output.push_str(&format!("/**\n * {}\n */\n", description.trim()));
                }

                output.push_str(&format!("export type {} = ", definition.name));

                let types = definition.types.join(" | ");

                output.push_str(&types);
                output.push_str(";\n\n");
            }

            // TODO: Scalar map config
            Definition::TypeDefinition(TypeDefinition::Scalar(definition)) => {
                if let Some(description) = &definition.description {
                    output.push_str(&format!("/**\n * {}\n */\n", description.trim()));
                }

                output.push_str(&format!("export type {} = any;\n\n", definition.name));
            }

            Definition::TypeDefinition(TypeDefinition::Object(definition)) => {
                if let Some(description) = &definition.description {
                    output.push_str(&format!("/**\n * {}\n */\n", description.trim()));
                }

                output.push_str(&format!("export interface {} {{ ", definition.name));

                let fields = definition
                    .fields
                    .iter()
                    .map(|field| return generate_object_field(field.clone()))
                    .collect::<Vec<String>>()
                    .join("; ");

                output.push_str(&fields);
                output.push_str(" }\n\n");
            }
            _ => {}
        }
    }

    output
}
