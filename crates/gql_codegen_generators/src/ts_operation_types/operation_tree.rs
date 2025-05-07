use std::collections::HashMap;

use anyhow::Result;
use apollo_compiler::{
    Name, Node,
    ast::{DirectiveList, Type},
    executable::{Fragment, Operation, Selection},
};
use indexmap::{IndexMap, IndexSet};

#[derive(Debug, Clone)]
pub(crate) struct OperationTreeNode {
    pub selection_refs: IndexSet<String>,
    pub field_name: String,
    pub field_type: Type,
    pub directives: DirectiveList,
    pub parent_type_name: String,
}

#[derive(Debug, Clone)]
pub(crate) struct OperationTree<'a> {
    operation: &'a Node<Operation>,
    fragments: &'a HashMap<Name, Node<Fragment>>,
    pub(crate) root_selection_refs: IndexSet<String>,
    pub(crate) normalized_fields: IndexMap<String, OperationTreeNode>,
}

impl<'a> OperationTree<'a> {
    pub fn new(
        operation: &'a Node<Operation>,
        fragments: &'a HashMap<Name, Node<Fragment>>,
    ) -> Result<Self> {
        let mut tree = Self {
            operation,
            fragments,
            root_selection_refs: IndexSet::new(),
            normalized_fields: IndexMap::new(),
        };

        tree.populate()?;
        Ok(tree)
    }

    fn populate(&mut self) -> Result<()> {
        let op_name = self.operation.operation_type.to_string();
        for selection in &self.operation.selection_set.selections {
            dbg!(&self.operation);
            self.populate_selection(selection, None, &op_name)?;
        }

        Ok(())
    }

    fn populate_selection(
        &mut self,
        selection: &Selection,
        parent_path: Option<String>,
        parent_type_name: &str,
    ) -> Result<()> {
        match selection {
            Selection::Field(field) => {
                let field_name = field.alias.clone().unwrap_or(field.name.clone());
                let field_type = field.ty();

                let field_path = match &parent_path {
                    Some(parent) => format!("{parent}.{field_name}"),
                    None => field_name.to_string(),
                };

                if let Some(path) = parent_path {
                    if let Some(parent) = self.normalized_fields.get_mut(&path) {
                        parent.selection_refs.insert(field_path.clone());
                    }
                } else {
                    self.root_selection_refs.insert(field_path.clone());
                }

                self.normalized_fields.insert(
                    field_path.clone(),
                    OperationTreeNode {
                        selection_refs: IndexSet::new(),
                        field_name: field_name.to_string(),
                        field_type: field_type.clone(),
                        directives: field.directives.clone(),
                        parent_type_name: parent_type_name.to_string(),
                    },
                );

                if !field.selection_set.is_empty() {
                    for child_selection in &field.selection_set.selections {
                        self.populate_selection(
                            child_selection,
                            Some(field_path.clone()),
                            &field.selection_set.ty,
                        )?;
                    }
                }
            }
            Selection::FragmentSpread(fragment_spread) => {
                let fragment = self.fragments.get(fragment_spread.fragment_name.as_str());

                let Some(fragment) = fragment else {
                    return Err(anyhow::anyhow!(
                        "Fragment \"{}\" for fragment spread not found.",
                        fragment_spread.fragment_name
                    ));
                };

                for child_selection in &fragment.selection_set.selections {
                    self.populate_selection(
                        child_selection,
                        parent_path.clone(),
                        parent_type_name,
                    )?;
                }
            }
            Selection::InlineFragment(inline_fragment) => {
                for child_selection in &inline_fragment.selection_set.selections {
                    self.populate_selection(
                        child_selection,
                        parent_path.clone(),
                        parent_type_name,
                    )?;
                }
            }
        }

        Ok(())
    }
}
