use std::collections::HashSet;

use apollo_compiler::{hir::Type, HirDatabase, RootDatabase};
use apollo_parser::cst::{
    CstNode, Definition, Document, EnumTypeDefinition, FragmentDefinition, ObjectTypeDefinition,
    OperationDefinition, SelectionSet,
};

use super::CodeGenerator;

pub struct TypeScriptGenerator {
    _type_name_prefix: Option<String>,
}

impl TypeScriptGenerator {
    pub fn new() -> Self {
        Self {
            _type_name_prefix: None,
        }
    }

    fn render_fragment_definition(
        &self,
        definition: &FragmentDefinition,
        db: &RootDatabase,
    ) -> Option<String> {
        let mut output = String::new();

        let type_name = definition.type_condition()?.named_type()?.name()?.text();
        let fragment_name = definition.fragment_name()?.name()?.text();
        let selection_set = definition.selection_set()?;

        output.push_str(&format!("export type {} = {{\n", fragment_name));
        output.push_str(&self.render_selection_set(
            &type_name,
            &selection_set,
            db,
            &mut HashSet::new(),
            1,
        )?);
        output.push_str("\n};");

        output.push_str("\n\n");

        Some(output)
    }

    fn render_enum_definition(&self, definition: &EnumTypeDefinition) -> Option<String> {
        let mut output = String::new();

        let name = definition.name()?.text();
        let values_definition = definition.enum_values_definition()?;

        output.push_str(&format!("export type {} = ", name));

        let mut values: Vec<String> = vec![];

        for value in values_definition.enum_value_definitions() {
            let enum_value = value.enum_value()?.text();
            values.push(format!("\"{}\"", enum_value.to_string()));
        }

        output.push_str(&values.join(" | "));
        output.push_str(";\n\n");

        Some(output)
    }

    fn render_object_definition(
        &self,
        definition: &ObjectTypeDefinition,
        db: &RootDatabase,
    ) -> Option<String> {
        let mut output = String::new();

        let name = definition.name()?.text();
        let fields_definition = definition.fields_definition()?;
        let object_type = db.find_type_definition_by_name(name.to_string())?;

        if let Some(description) = definition.description() {
            let source_string = description.source_string();
            let description_value = source_string
                .strip_prefix("\"\"\"")?
                .strip_suffix("\"\"\"")?
                .trim();
            // TODO MULTILINE DESCRIPTION
            output.push_str(&format!("/**\n * {}\n */\n", description_value));
        }

        // let type_extensions = db.extensions().iter().filter_map(|t| {
        //     if let Some(name) = t.name() {
        //         if name == name {
        //             return Some(t);
        //         }
        //         return None;
        //     }
        //
        //     return None;
        // });
        // dbg!(type_extensions);

        output.push_str(&format!("export type {} = {{\n", &name));

        for field in fields_definition.field_definitions() {
            let field_name = field.name()?.text();
            let field_type = object_type.field(db, &field_name);
            let rendered_type = render_type(&field_type?.ty(), false);

            output.push_str(&format!("  {}: {};\n", field_name, rendered_type));
        }

        output.push_str("};\n\n");

        Some(output)
    }

    fn render_operation_definition(
        &self,
        _definition: &OperationDefinition,
        _db: &RootDatabase,
    ) -> Option<String> {
        let output = String::new();

        // let operation_name = definition.name()?.text();
        // output.push_str(&format!("export type {} = {{", operation_name));

        // let selection_set = definition.selection_set()?;

        // for selection in selection_set.selections() {
        //     dbg!(selection);
        // }

        Some(output)
    }

    fn render_selection_set(
        &self,
        type_name: &str,
        selection_set: &SelectionSet,
        db: &RootDatabase,
        unique_selections: &mut HashSet<String>,
        nesting_depth: usize,
    ) -> Option<String> {
        let mut output = String::new();

        let mut fragments: Vec<String> = Vec::new();
        let mut fields: Vec<String> = Vec::new();

        let indentation = "  ".repeat(nesting_depth);

        // output.push_str("{\n");
        // output.push_str(&format!("{}__typename?: \"{}\";\n", indentation, type_name));

        // let type_definition = db.find_type_definition_by_name(type_name.to_string())?;
        //
        // for selection in selection_set.selections() {
        //     match selection {
        //         Selection::Field(field) => {
        //             let field_name = field.name()?.text();
        //             let field_type = type_definition.field(db, &field_name)?.ty();
        //             let field_type_name = field_type.name();
        //
        //             if unique_selections.contains(field_name.as_str()) {
        //                 continue;
        //             }
        //
        //             unique_selections.insert(field_name.to_string());
        //
        //             if let Some(nested_selection) = field.selection_set() {
        //                 let rendered = self.render_selection_set(
        //                     &field_type_name,
        //                     &nested_selection,
        //                     db,
        //                     &mut HashSet::new(),
        //                     nesting_depth + 1,
        //                 );
        //                 fields.push(format!(
        //                     "{}{}: {{\n{}\n{}}};",
        //                     indentation, field_name, rendered?, indentation
        //                 ));
        //                 continue;
        //             }
        //
        //             let rendered_type = render_type(&field_type, false);
        //             fields.push(format!("{}{}: {};", indentation, field_name, rendered_type));
        //         }
        //
        //         Selection::FragmentSpread(fragment_spread) => {
        //             let name = fragment_spread.fragment_name()?.name()?.text();
        //             fragments.push(name.to_string());
        //         }
        //
        //         Selection::InlineFragment(inline_fragment) => {
        //             let selection_set = inline_fragment.selection_set()?;
        //             let rendered_selection_set = self.render_selection_set(
        //                 &type_name,
        //                 &selection_set,
        //                 db,
        //                 &mut unique_selections.clone(),
        //                 nesting_depth,
        //             )?;
        //             fields.push(format!("\n{}", rendered_selection_set));
        //         }
        //     }
        // }

        output.push_str(&fields.join("\n"));

        Some(output)
    }
}

impl CodeGenerator for TypeScriptGenerator {
    fn generate(&self, document: &Document, db: &RootDatabase) -> String {
        let mut result = String::new();

        for definition in document.definitions() {
            let rendered_definition = match definition {
                Definition::OperationDefinition(definition) => {
                    self.render_operation_definition(&definition, db)
                }
                Definition::FragmentDefinition(definition) => {
                    self.render_fragment_definition(&definition, db)
                }

                Definition::EnumTypeDefinition(definition) => {
                    self.render_enum_definition(&definition)
                }

                Definition::ObjectTypeDefinition(definition) => {
                    self.render_object_definition(&definition, db)
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

            if let Some(rendered_definition) = rendered_definition {
                result.push_str(&rendered_definition);
            }
        }

        result
    }
}

fn render_type(ty: &Type, non_null: bool) -> String {
    match ty {
        Type::Named { name, loc: _ } => {
            let text = render_scalar(name);

            if non_null {
                return text;
            }

            return format!("{} | null", text);
        }
        Type::NonNull { ty, loc: _ } => {
            return render_type(ty, true);
        }
        Type::List { ty, loc: _ } => {
            if non_null {
                return format!("Array<{}>", render_type(ty, false));
            }

            return format!("Array<{}> | null", render_type(ty, false));
        }
    }
}

fn render_scalar(value: &str) -> String {
    match value {
        "ID" => String::from("string"),
        "String" => String::from("string"),
        "Boolean" => String::from("boolean"),
        "Int" => String::from("number"),
        "Float" => String::from("number"),
        _ => value.to_string(),
    }
}
