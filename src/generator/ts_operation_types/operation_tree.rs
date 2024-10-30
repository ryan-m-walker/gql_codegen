use apollo_compiler::{
    hir::{OperationDefinition, Selection, SelectionSet, Type},
    RootDatabase,
};
use indexmap::IndexMap;

#[derive(Debug, Clone)]
pub struct TypenameField {
    pub name: String,
    pub nullable: bool,
}

#[derive(Debug, Clone)]
pub struct ScalarField {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub enum OperationField {
    Selection(OperationTree),
    Scalar(ScalarField),
    Typename(TypenameField),
}

#[derive(Debug, Clone)]
/// The compiler does not merge or deduplicate fields so we we need to walk the operations CSTs
/// and build a tree of the fields that are selected.
pub struct OperationTree {
    pub fields: IndexMap<String, OperationField>,
    pub field_name: String,
    pub ty: Option<Type>,
}

pub fn build_operation_tree(
    operation: &OperationDefinition,
    db: &RootDatabase,
) -> Option<OperationTree> {
    let mut render_tree = OperationTree {
        fields: IndexMap::new(),
        field_name: operation.name()?.to_string(),
        ty: None,
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
                    if let Some(selection) = render_tree.fields.get_mut(name) {
                        if let OperationField::Selection(ref mut render_tree) = selection {
                            populate_operation_tree(&field.selection_set(), db, render_tree);
                        }
                    } else {
                        let mut new_render_tree = OperationTree {
                            fields: IndexMap::new(),
                            field_name: name.to_string(),
                            ty: Some(field_type.clone()),
                        };

                        populate_operation_tree(&field.selection_set(), db, &mut new_render_tree);
                        render_tree
                            .fields
                            .insert(name.to_string(), OperationField::Selection(new_render_tree));
                    }

                    continue;
                }

                // TODO: make this configurable
                if !render_tree.fields.contains_key("__typename") {
                    render_tree.fields.insert(
                        "__typename".to_string(),
                        OperationField::Typename(TypenameField {
                            name: parent.name().to_string(),
                            nullable: true,
                        }),
                    );
                }

                // override typename with non-nullable value if specifically selected
                if name == "__typename" {
                    render_tree.fields.insert(
                        name.to_string(),
                        OperationField::Typename(TypenameField {
                            name: parent.name().to_string(),
                            nullable: false,
                        }),
                    );
                }

                if render_tree.fields.contains_key(name) {
                    continue;
                }

                render_tree.fields.insert(
                    name.to_string(),
                    OperationField::Scalar(ScalarField {
                        name: name.to_string(),
                        ty: field_type.clone(),
                    }),
                );
            }

            Selection::FragmentSpread(fragment_spread) => {
                populate_operation_tree(
                    &fragment_spread.fragment(db)?.selection_set(),
                    db,
                    render_tree,
                );
            }

            Selection::InlineFragment(inline_fragment) => {
                populate_operation_tree(&inline_fragment.selection_set(), db, render_tree);
            }
        }
    }

    Some(())
}
