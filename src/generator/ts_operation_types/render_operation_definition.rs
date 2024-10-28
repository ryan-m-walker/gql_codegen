use std::collections::HashMap;

use apollo_compiler::{
    hir::{OperationDefinition, Selection, SelectionSet},
    HirDatabase, RootDatabase,
};

use crate::generator::common::render_type;

use super::render_selection_set::render_selection_set;

#[derive(Debug, Clone)]
enum FieldType {
    Selection(HashMap<String, FieldType>),
    Field(String),
}

pub fn render_operation_definition(
    definition: &OperationDefinition,
    db: &RootDatabase,
) -> Option<String> {
    let mut result = String::new();

    let operation_name = definition.name()?; // TODO: handler where this is actually undefined
    result.push_str(&format!("export type {operation_name} = {{\n"));

    let mut rendered_field_map: HashMap<String, FieldType> = HashMap::new();
    populate_rendered_field_name(&mut rendered_field_map, &definition.selection_set(), db);

    for value in rendered_field_map.values() {
        println!("{:?}", value);
    }

    // let inline_fields = definition.fields_in_fragment_spread(db);
    // dbg!(&inline_fields);

    // let selection_set = definition.selection_set().to_owned();
    // let flat = db.flattened_operation_fields(selection_set);

    // dbg!(selection_set);
    // for field in flat.as_ref() {
    //     println!("{:?}", field.name());
    // }
    // let rendered_selection_set = render_selection_set(&definition.selection_set(), db, 1);
    // result.push_str(&rendered_selection_set?);

    result.push_str("};\n\n");

    Some(result)
}

fn populate_rendered_field_name(
    rendered_field_map: &mut HashMap<String, FieldType>,
    selection_set: &SelectionSet,
    db: &RootDatabase,
) -> Option<()> {
    for field in selection_set.selection() {
        match field {
            Selection::Field(field) => {
                let name = if let Some(alias) = field.alias() {
                    alias.name()
                } else {
                    field.name()
                };

                if field.selection_set().selection().len() > 0 {
                    populate_rendered_field_name(rendered_field_map, &field.selection_set(), db);
                }

                if rendered_field_map.contains_key(name) {
                    continue;
                }

                let rendered_type = render_type(&field.ty(db)?, false);
                let rendered = format!("{}: {};", name, rendered_type);
                rendered_field_map.insert(name.to_string(), FieldType::Field(rendered));
            }

            _ => {}
        }
    }

    Some(())
}
