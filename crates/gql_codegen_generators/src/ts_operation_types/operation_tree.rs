use std::{any::type_name, collections::HashMap};

use anyhow::{Result, anyhow};
use apollo_compiler::{
    Name, Node, Schema,
    ast::{DirectiveList, Selection, Type},
    schema::{InputObjectType, InterfaceType, ObjectType},
    validation::Valid,
};
use colored::Colorize;
use gql_codegen_errors::GQLValidationError;
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
pub(crate) struct OperationTree<'a> {
    schema: &'a Valid<Schema>,
    operation_result: &'a OperationResult,
    fragment_results: &'a HashMap<Name, FragmentResult>,
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
        operation_result: &'a OperationResult,
        fragment_results: &'a HashMap<Name, FragmentResult>,
    ) -> Result<Self> {
        let mut tree = Self {
            schema,
            operation_result,
            fragment_results,
            root_selection_refs: IndexSet::new(),
            normalized_fields: IndexMap::new(),
        };

        tree.populate()?;
        Ok(tree)
    }

    fn populate(&mut self) -> Result<()> {
        let op_name = self.operation_result.operation.operation_type;

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

        for selection in &self.operation_result.operation.selection_set {
            self.populate_selection(selection, None, &op_type.name)?;
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

                let Some(fragment_type) = self.schema.get_object(type_condition) else {
                    return Err(anyhow!(
                        "Fragment type \"{}\" not found in schema.",
                        type_condition
                    ));
                };

                for selection in &fragment_result.fragment.selection_set {
                    self.populate_selection(selection, parent_path, &fragment_type.name)?;
                }
            }
            _ => {
                //
            } // Selection::InlineFragment(inline_fragment) => {
              //     let Some(type_condition) = &inline_fragment.type_condition else {
              //         return Err(anyhow!("Inline fragment without type condition found."));
              //     };
              //
              //     let Some(fragment_type) = self.schema.get_object(type_condition) else {
              //         return Err(anyhow!(
              //             "Fragment type \"{}\" not found in schema.",
              //             type_condition
              //         ));
              //     };
              //
              //     for selection in &inline_fragment.selection_set {
              //         self.populate_selection(selection, parent_path, &fragment_type.name)?;
              //     }
              // }
        }

        // match selection {
        //     Selection::Field(field) => {

        //         match parent_type {
        //             FieldType::Object(object) => {
        //                 // let type_name = self.type_to_type_name(&field_definition.ty);
        //
        //                 let Some(field_definition) = object.fields.get(&field.name) else {
        //                     panic!("TODO");
        //                 };
        //
        //                 let type_name = self.type_to_type_name(&field_definition.ty);
        //
        //                 if !field.selection_set.is_empty() {
        //                     let Some(field_type) = self.get_type_for_type_name(&type_name) else {
        //                         return Err(anyhow!(
        //                             "Cannot query field \"{}\" on type \"{}\". Type \"{}\" not found.",
        //                             field.name,
        //                             parent_type.name,
        //                             type_name
        //                         ));
        //                     };
        //
        //                     for child_selection in &field.selection_set {
        //                         self.populate_selection(
        //                             child_selection,
        //                             Some(&field_path),
        //                             field_type,
        //                         )?;
        //                     }
        //                 }
        //             }
        //             _ => {}
        //         }
        //
        //         let Some(field_definition) = parent_type.fields.get(&field.name) else {
        //             if let Some(location) = &field.name.location() {
        //                 let loc = location
        //                     .line_column_range(&self.operation_result.sources)
        //                     .unwrap();
        //
        //                 let (_, source) = self.operation_result.sources.last().unwrap();
        //
        //                 let lines = source.source_text().lines().collect::<Vec<_>>();
        //                 let line_before = lines.get(loc.start.line - 2).unwrap_or(&"").to_string();
        //                 let line = lines.get(loc.start.line - 1).unwrap_or(&"").to_string();
        //                 let line_after = lines.get(loc.start.line).unwrap_or(&"").to_string();
        //
        //                 println!();
        //                 println!("      {}", source.path().to_string_lossy().bold());
        //                 println!("     │");
        //                 println!("   {} │ {line_before}", loc.start.line - 1);
        //                 println!("   {} │ {}", loc.start.line, line);
        //
        //                 let indent = " ".repeat(loc.start.column - 1);
        //                 let underline = "^".repeat(loc.end.column - loc.start.column).red().bold();
        //
        //                 println!("     │ {indent}{underline}");
        //
        //                 println!("   {} │ {line_after}", loc.start.line + 1);
        //                 println!();
        //             }
        //
        //             return Err(GQLValidationError::new(
        //                 format!(
        //                     "Cannot query field \"{}\" on type \"{}\" at {:?}:{:?}",
        //                     field.name,
        //                     parent_type.name,
        //                     field.location().unwrap().offset(),
        //                     field.location().unwrap().end_offset(),
        //                 ),
        //                 vec![(0, 0)],
        //             ))?;
        //         };
        //
        //         if !self.normalized_fields.contains_key(&field_path) {
        //             self.normalized_fields.insert(
        //                 field_path.clone(),
        //                 OperationTreeNode {
        //                     selection_refs: IndexSet::new(),
        //                     directives: field.directives.clone(),
        //                     field_name: field_name.to_string(),
        //                     field_type: field_definition.ty.clone(),
        //                     parent_type_name: parent_type.name.to_string(),
        //                 },
        //             );
        //         }
        //
        //         let type_name = self.type_to_type_name(&field_definition.ty);
        //
        //         if !field.selection_set.is_empty() {
        //             let Some(field_type) = self.get_type_for_type_name(&type_name) else {
        //                 return Err(anyhow!(
        //                     "Cannot query field \"{}\" on type \"{}\". Type \"{}\" not found.",
        //                     field.name,
        //                     parent_type.name,
        //                     type_name
        //                 ));
        //             };
        //
        //             for child_selection in &field.selection_set {
        //                 self.populate_selection(child_selection, Some(&field_path), field_type)?;
        //             }
        //         }
        //     }
        //     Selection::FragmentSpread(fragment_spread) => {
        //         let Some(fragment_result) =
        //             self.fragment_results.get(&fragment_spread.fragment_name)
        //         else {
        //             return Err(anyhow!(
        //                 "Fragment \"{}\" not found in schema.",
        //                 fragment_spread.fragment_name
        //             ));
        //         };
        //
        //         for selection in &fragment_result.fragment.selection_set {
        //             self.populate_selection(selection, parent_path, parent_type)?;
        //         }
        //     }
        //     Selection::InlineFragment(inline_fragment) => {
        //         let Some(type_condition) = &inline_fragment.type_condition else {
        //             return Err(anyhow!("Inline fragment without type condition found."));
        //         };
        //
        //         let Some(fragment_type) = self.schema.get_object(type_condition) else {
        //             return Err(anyhow!(
        //                 "Fragment type \"{}\" not found in schema.",
        //                 type_condition
        //             ));
        //         };
        //
        //         for selection in &inline_fragment.selection_set {
        //             self.populate_selection(selection, parent_path, fragment_type)?;
        //         }
        //     }
        // }

        Ok(())
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
