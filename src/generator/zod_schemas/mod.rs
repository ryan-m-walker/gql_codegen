use apollo_compiler::{HirDatabase, RootDatabase};
use enum_definition::render_enum_definition;
use input_object_definition::render_input_object_definition;
use interface_definition::render_interface_definition;
use object_definition::render_object_definition;
use union_definition::render_union_definition;

use super::CodeGenerator;

mod enum_definition;
mod input_object_definition;
mod interface_definition;
mod object_definition;
mod render_type;
mod union_definition;

pub struct ZodSchemasGenerator;

impl CodeGenerator for ZodSchemasGenerator {
    fn generate(&self, db: &RootDatabase) -> String {
        let mut result = String::new();

        result.push_str("import { z } from 'zod';\n\n");

        for enum_definition in db.type_system_definitions().enums.values() {
            if enum_definition.is_introspection() {
                continue;
            }

            result.push_str(&render_enum_definition(&enum_definition));
        }

        for object_definition in db.type_system_definitions().objects.values() {
            if object_definition.is_introspection() {
                continue;
            }

            result.push_str(&render_object_definition(&object_definition));
        }

        for input_object_definition in db.type_system_definitions().input_objects.values() {
            result.push_str(&render_input_object_definition(&input_object_definition));
        }

        for interface_definition in db.type_system_definitions().interfaces.values() {
            result.push_str(&render_interface_definition(&interface_definition));
        }

        for union_definition in db.type_system_definitions().unions.values() {
            result.push_str(&render_union_definition(&union_definition));
        }

        for scalar_definition in db.type_system_definitions().scalars.values() {
            if !scalar_definition.is_custom() {
                continue;
            }

            result.push_str(&format!(
                "export const {}Schema = z.unknown();\n\n",
                scalar_definition.name()
            ));
        }

        result
    }
}
