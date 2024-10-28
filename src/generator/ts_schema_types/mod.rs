use apollo_compiler::RootDatabase;
use apollo_parser::cst::{Definition, Document};

use self::{
    enum_definition::render_enum_definition,
    input_object_definition::render_input_object_definition,
    interface_definition::render_interface_definition, object_definition::render_object_definition,
    union_definition::render_union_definition,
};

use super::CodeGenerator;

pub struct TsSchemaTypesGenerator;

pub(self) mod descriptions;
pub(self) mod enum_definition;
pub(self) mod input_object_definition;
pub(self) mod interface_definition;
pub(self) mod object_definition;
pub(self) mod union_definition;

impl CodeGenerator for TsSchemaTypesGenerator {
    fn generate(&self, document: &Document, db: &RootDatabase) -> String {
        let mut result = String::new();

        for definition in document.definitions() {
            let rendered_definition = match definition {
                Definition::EnumTypeDefinition(definition) => {
                    render_enum_definition(&definition, db)
                }

                Definition::ObjectTypeDefinition(definition) => {
                    render_object_definition(&definition, db)
                }

                Definition::InputObjectTypeDefinition(definition) => {
                    render_input_object_definition(&definition, db)
                }

                Definition::UnionTypeDefinition(definition) => {
                    render_union_definition(&definition, db)
                }

                Definition::InterfaceTypeDefinition(definition) => {
                    render_interface_definition(&definition, db)
                }

                Definition::ScalarTypeDefinition(definition) => {
                    if let Some(name) = definition.name() {
                        Some(format!("export type {} = unknown;\n\n", name.text()))
                    } else {
                        None
                    }
                }

                _ => None,
            };

            if let Some(rendered) = rendered_definition {
                result.push_str(&rendered);
            }
        }

        result
    }
}
