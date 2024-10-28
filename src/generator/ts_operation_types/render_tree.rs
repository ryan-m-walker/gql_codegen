use super::operation_tree::{OperationField, OperationTree};

pub fn render_operation_tree(operation_tree: &OperationTree, depth: usize) -> String {
    let mut fields: Vec<String> = vec![];

    let indentation = "  ".repeat(depth);

    for field in operation_tree.fields.values() {
        match field {
            OperationField::Field(value) => {
                let mut rendered_field = String::new();

                rendered_field.push_str(indentation.as_str());
                rendered_field.push_str(&value);

                fields.push(rendered_field);
            }
            OperationField::Selection(selection_set) => {
                let mut rendered_field = String::new();

                rendered_field
                    .push_str(&format!("{indentation}{}: ", selection_set.field_name).as_str());

                if selection_set.is_list {
                    rendered_field.push_str("Array<");
                }

                rendered_field.push_str("{");
                rendered_field.push_str("\n");

                rendered_field.push_str(&render_operation_tree(&selection_set, depth + 1));

                rendered_field.push_str("\n");
                rendered_field.push_str(indentation.as_str());

                rendered_field.push_str("}");

                if selection_set.is_list {
                    rendered_field.push_str(">");
                }

                if !selection_set.is_non_null {
                    rendered_field.push_str(" | null");
                }

                rendered_field.push_str(";");

                fields.push(rendered_field);
            }
        }
    }

    fields.join("\n")
}
