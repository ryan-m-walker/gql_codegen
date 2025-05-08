use std::collections::HashMap;

use anyhow::{Result, anyhow};
use apollo_compiler::{
    Name, Node, Schema,
    ast::{DirectiveList, FragmentDefinition, OperationDefinition, Selection, Type},
    schema::ObjectType,
    validation::Valid,
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
    schema: &'a Valid<Schema>,
    operation: &'a Node<OperationDefinition>,
    fragments: &'a HashMap<Name, Node<FragmentDefinition>>,
    pub(crate) root_selection_refs: IndexSet<String>,
    pub(crate) normalized_fields: IndexMap<String, OperationTreeNode>,
}

impl<'a> OperationTree<'a> {
    pub fn new(
        schema: &'a Valid<Schema>,
        operation: &'a Node<OperationDefinition>,
        fragments: &'a HashMap<Name, Node<FragmentDefinition>>,
    ) -> Result<Self> {
        let mut tree = Self {
            schema,
            operation,
            fragments,
            root_selection_refs: IndexSet::new(),
            normalized_fields: IndexMap::new(),
        };

        tree.populate()?;
        Ok(tree)
    }

    fn populate(&mut self) -> Result<()> {
        let op_name = self.operation.operation_type;

        let Some(root_op) = self.schema.root_operation(op_name) else {
            return Err(anyhow!(
                "Root operation type \"{op_name}\" not found in schema."
            ));
        };

        let Some(op_type) = self.schema.get_object(root_op) else {
            return Err(anyhow!(
                "Root operation type \"{op_name}\" not found in schema."
            ));
        };

        for selection in &self.operation.selection_set {
            self.populate_selection(selection, None, op_type)?;
        }

        Ok(())
    }

    fn populate_selection(
        &mut self,
        selection: &Selection,
        parent_path: Option<&str>,
        parent_type: &Node<ObjectType>, // parent_type: &Type,
    ) -> Result<()> {
        match selection {
            Selection::Field(field) => {
                let field_name = field.alias.clone().unwrap_or(field.name.clone());

                let field_path = match &parent_path {
                    Some(parent) => format!("{parent}.{field_name}"),
                    None => field_name.to_string(),
                };

                if let Some(path) = parent_path {
                    if let Some(parent) = self.normalized_fields.get_mut(path) {
                        parent.selection_refs.insert(field_path.clone());
                    }
                } else {
                    self.root_selection_refs.insert(field_path.clone());
                }

                if field_name == "__typename" {
                    // TODO: handle __typename
                    return Ok(());
                }

                let Some(field_definition) = parent_type.fields.get(&field.name) else {
                    panic!("TODO 2");
                };

                if !self.normalized_fields.contains_key(&field_path) {
                    self.normalized_fields.insert(
                        field_path.clone(),
                        OperationTreeNode {
                            selection_refs: IndexSet::new(),
                            directives: field.directives.clone(),
                            field_name: field_name.to_string(),
                            field_type: field_definition.ty.clone(),
                            parent_type_name: parent_type.name.to_string(),
                        },
                    );
                }

                let type_name = self.type_to_type_name(&field_definition.ty);

                if !field.selection_set.is_empty() {
                    let Some(field_type) = self.schema.get_object(&type_name) else {
                        return Err(anyhow!("Field \"{}\" not found in schema.", field.name));
                    };

                    for child_selection in &field.selection_set {
                        self.populate_selection(child_selection, Some(&field_path), field_type)?;
                    }
                }
            }
            Selection::FragmentSpread(fragment_spread) => {
                let Some(fragment) = self.fragments.get(&fragment_spread.fragment_name) else {
                    return Err(anyhow!(
                        "Fragment \"{}\" not found in schema.",
                        fragment_spread.fragment_name
                    ));
                };

                for selection in &fragment.selection_set {
                    self.populate_selection(selection, parent_path, parent_type)?;
                }
            }
            Selection::InlineFragment(inline_fragment) => {
                let Some(type_condition) = &inline_fragment.type_condition else {
                    return Err(anyhow!("Inline fragment without type condition found."));
                };

                let Some(fragment_type) = self.schema.get_object(type_condition) else {
                    return Err(anyhow!(
                        "Fragment type \"{}\" not found in schema.",
                        type_condition
                    ));
                };

                for selection in &inline_fragment.selection_set {
                    self.populate_selection(selection, parent_path, fragment_type)?;
                }
            }
        }

        Ok(())
    }

    fn type_to_type_name(&self, ty: &Type) -> String {
        match ty {
            Type::Named(name) => name.to_string(),
            Type::List(list) => self.type_to_type_name(list),
            Type::NonNullList(list) => self.type_to_type_name(list),
            Type::NonNullNamed(name) => name.to_string(),
        }
    }
}
