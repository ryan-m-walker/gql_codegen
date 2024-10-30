use crate::generator::common::{render_type, render_wrapped_type};

use super::operation_tree::{OperationField, OperationTree};

pub fn render_operation_tree(operation_tree: &OperationTree, depth: usize) -> String {
    let mut fields: Vec<String> = vec![];

    let indentation = "  ".repeat(depth);

    for field in operation_tree.fields.values() {
        match field {
            OperationField::Scalar(value) => {
                let mut rendered_field = String::new();

                let rendered_type = render_type(&value.ty, false);
                let rendered = format!(
                    "{indentation}{field_name}: {rendered_type};",
                    indentation = indentation,
                    field_name = value.name,
                    rendered_type = rendered_type,
                );
                rendered_field.push_str(&rendered);

                fields.push(rendered_field);
            }
            OperationField::Selection(selection_set) => {
                let Some(ty) = selection_set.ty.as_ref() else {
                    continue;
                };

                let mut rendered_field = String::new();

                rendered_field
                    .push_str(&format!("{indentation}{}: ", selection_set.field_name).as_str());

                let mut inner_type = String::new();
                inner_type.push_str("{");
                inner_type.push_str("\n");

                inner_type.push_str(&render_operation_tree(&selection_set, depth + 1));

                inner_type.push_str("\n");
                inner_type.push_str(indentation.as_str());

                inner_type.push_str("}");

                let wrapped = render_wrapped_type(&ty, false, |_| inner_type.clone());

                rendered_field.push_str(wrapped.as_str());
                rendered_field.push_str(";");

                fields.push(rendered_field);
            }
            OperationField::Typename(typename) => {
                let mut rendered_field = String::new();

                rendered_field.push_str(indentation.as_str());
                rendered_field.push_str("__typename");

                if typename.nullable {
                    rendered_field.push_str("?");
                }

                rendered_field.push_str(&format!(": '{}';", typename.name));
                fields.push(rendered_field);
            }
        }
    }

    fields.join("\n")
}
