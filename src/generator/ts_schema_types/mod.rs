use apollo_compiler::{HirDatabase, RootDatabase};

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
    fn generate(&self, db: &RootDatabase) -> String {
        let mut result = String::new();

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
                "export type {} = unknown;\n\n",
                scalar_definition.name()
            ));
        }

        result
    }
}
