use anyhow::{Result, anyhow};
use apollo_compiler::{
    Name, Node, Schema,
    ast::{DirectiveList, Selection, Type},
    schema::{InputObjectType, InterfaceType, ObjectType},
    validation::Valid,
};
use gql_codegen_types::{FragmentResult, OperationResult};
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
pub(crate) enum OperationTreeInput<'a> {
    Operation(&'a OperationResult),
    Fragment(&'a FragmentResult),
}

#[derive(Debug, Clone)]
pub(crate) struct OperationTree<'a> {
    schema: &'a Valid<Schema>,
    input: OperationTreeInput<'a>,
    fragment_results: &'a IndexMap<Name, FragmentResult>,
    pub(crate) root_selection_refs: IndexSet<String>,
    pub(crate) normalized_fields: IndexMap<String, OperationTreeNode>,
}

enum FieldType {
    Object(Node<ObjectType>),
    InputObject(Node<InputObjectType>),
    Interface(Node<InterfaceType>),
}

struct FieldData {
    ty: Type,
    name: String,
}

impl<'a> OperationTree<'a> {
    pub fn new(
        schema: &'a Valid<Schema>,
        input: OperationTreeInput<'a>,
        fragment_results: &'a IndexMap<Name, FragmentResult>,
    ) -> Result<Self> {
        let mut tree = Self {
            schema,
            input,
            fragment_results,
            root_selection_refs: IndexSet::new(),
            normalized_fields: IndexMap::new(),
        };

        tree.populate()?;
        Ok(tree)
    }

    fn populate(&mut self) -> Result<()> {
        match self.input {
            OperationTreeInput::Operation(operation_result) => {
                self.populate_operation(operation_result)?;
            }
            OperationTreeInput::Fragment(fragment_result) => {
                self.populate_fragment(fragment_result)?;
            }
        }

        Ok(())
    }

    fn populate_operation(&mut self, operation_result: &OperationResult) -> Result<()> {
        let op_name = operation_result.operation.operation_type;

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

        for selection in &operation_result.operation.selection_set {
            self.populate_selection(selection, None, &op_type.name)?;
        }

        Ok(())
    }

    fn populate_fragment(&mut self, fragment_result: &FragmentResult) -> Result<()> {
        let type_condition = &fragment_result.fragment.type_condition;

        for selection in &fragment_result.fragment.selection_set {
            self.populate_selection(selection, None, type_condition)?;
        }

        Ok(())
    }

    fn populate_selection(
        &mut self,
        selection: &Selection,
        parent_path: Option<&str>,
        // parent_type: &FieldType,
        parent_type: &Name,
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

                let Ok(field_definition) = self.schema.type_field(parent_type, &field.name) else {
                    return Ok(());
                };

                if !self.normalized_fields.contains_key(&field_path) {
                    self.normalized_fields.insert(
                        field_path.clone(),
                        OperationTreeNode {
                            selection_refs: IndexSet::new(),
                            directives: field.directives.clone(),
                            field_name: field_name.to_string(),
                            field_type: field_definition.ty.clone(),
                            parent_type_name: parent_type.to_string(),
                        },
                    );
                }

                let type_name = self.type_to_type_name(&field_definition.ty);

                if !field.selection_set.is_empty() {
                    for child_selection in &field.selection_set {
                        self.populate_selection(child_selection, Some(&field_path), &type_name)?;
                    }
                }
            }
            Selection::FragmentSpread(fragment_spread) => {
                let fragment_result = self.fragment_results.get(&fragment_spread.fragment_name);

                let Some(fragment_result) = fragment_result else {
                    return Err(anyhow!(
                        "Fragment \"{}\" not found in schema.",
                        fragment_spread.fragment_name
                    ));
                };

                let type_condition = &fragment_result.fragment.type_condition;

                let Some(fragment_type) = self.get_type_name_for_type(type_condition) else {
                    return Err(anyhow!(
                        "Fragment type \"{}\" not found in schema.",
                        type_condition
                    ));
                };

                for selection in &fragment_result.fragment.selection_set {
                    self.populate_selection(selection, parent_path, &fragment_type)?;
                }
            }
            Selection::InlineFragment(inline_fragment) => {
                let type_condition = &inline_fragment.type_condition;

                let Some(type_condition) = type_condition else {
                    return Err(anyhow!("Inline fragment type condition is missing."));
                };

                // let Some(fragment_type) = self.get_type_for_type_name(type_condition) else {
                //     return Err(anyhow!(
                //         "Fragment type \"{}\" not found in schema.",
                //         type_condition
                //     ));
                // };

                for selection in &inline_fragment.selection_set {
                    self.populate_selection(selection, parent_path, type_condition)?;
                }
            }
        }

        Ok(())
    }

    fn get_type_name_for_type(&self, type_name: &Name) -> Option<Name> {
        if let Some(object_type) = self.schema.get_object(type_name) {
            return Some(object_type.name.clone());
        }

        if let Some(input_object_type) = self.schema.get_input_object(type_name) {
            return Some(input_object_type.name.clone());
        }

        if let Some(interface_type) = self.schema.get_interface(type_name) {
            return Some(interface_type.name.clone());
        }

        None
    }

    fn get_type_for_type_name(&self, type_name: &str) -> Option<FieldType> {
        if let Some(object_type) = self.schema.get_object(type_name) {
            return Some(FieldType::Object(object_type.clone()));
        }

        if let Some(input_object_type) = self.schema.get_input_object(type_name) {
            return Some(FieldType::InputObject(input_object_type.clone()));
        }

        if let Some(enum_type) = self.schema.get_interface(type_name) {
            return Some(FieldType::Interface(enum_type.clone()));
        }

        None
    }

    fn type_to_type_name(&self, ty: &Type) -> Name {
        match ty {
            Type::Named(name) => name.clone(),
            Type::List(list) => self.type_to_type_name(list),
            Type::NonNullList(list) => self.type_to_type_name(list),
            Type::NonNullNamed(name) => name.clone(),
        }
    }
}
