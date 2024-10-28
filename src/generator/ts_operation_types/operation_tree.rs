use std::usize;

use apollo_compiler::{
    hir::{OperationDefinition, Selection, SelectionSet},
    RootDatabase,
};
use indexmap::IndexMap;

use crate::generator::common::render_type;

#[derive(Debug, Clone)]
pub enum OperationField {
    Selection(OperationTree),
    Field(String),
}

#[derive(Debug, Clone)]
pub struct OperationTree {
    pub is_non_null: bool,
    pub is_list: bool,
    pub fields: IndexMap<String, OperationField>,
    pub field_name: String,
}

pub fn build_operation_tree(
    operation: &OperationDefinition,
    db: &RootDatabase,
) -> Option<OperationTree> {
    let mut render_tree = OperationTree {
        is_non_null: false,
        is_list: false,
        fields: IndexMap::new(),
        field_name: operation.name()?.to_string(),
    };

    populate_operation_tree(&operation.selection_set(), db, &mut render_tree);

    return Some(render_tree);
}

fn populate_operation_tree(
    selection_set: &SelectionSet,
    db: &RootDatabase,
    render_tree: &mut OperationTree,
) -> Option<()> {
    for field in selection_set.selection() {
        match field {
            Selection::Field(field) => {
                let name = if let Some(alias) = field.alias() {
                    alias.name()
                } else {
                    field.name()
                };

                let parent = field.parent_type(db)?;
                let field_type = parent.field(db, field.name())?.ty();

                if field.selection_set().selection().len() > 0 {
                    dbg!(&field_type);

                    if let Some(selection) = render_tree.fields.get_mut(name) {
                        if let OperationField::Selection(ref mut render_tree) = selection {
                            populate_operation_tree(&field.selection_set(), db, render_tree);
                        }
                    } else {
                        let mut new_render_tree = OperationTree {
                            is_non_null: field_type.is_non_null(),
                            is_list: field_type.is_list(),
                            fields: IndexMap::new(),
                            field_name: name.to_string(),
                        };

                        populate_operation_tree(&field.selection_set(), db, &mut new_render_tree);
                        render_tree
                            .fields
                            .insert(name.to_string(), OperationField::Selection(new_render_tree));
                    }

                    continue;
                }

                if !render_tree.fields.contains_key("__typename") {
                    let rendered_field = format!("__typename?: '{}';", parent.name());
                    render_tree.fields.insert(
                        "__typename".to_string(),
                        OperationField::Field(rendered_field.to_string()),
                    );
                }

                if render_tree.fields.contains_key(name) {
                    continue;
                }

                let rendered_type = render_type(&field_type, false);
                let rendered_field = format!("{name}: {rendered_type};");

                render_tree.fields.insert(
                    name.to_string(),
                    OperationField::Field(rendered_field.to_string()),
                );
            }

            _ => {}
        }
    }

    Some(())
}
