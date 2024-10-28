use super::operation_tree::{OperationField, OperationTree};

pub fn render_operation_tree(operation_tree: &OperationTree, depth: usize) -> String {
    let mut output = String::new();

    let indentation = "  ".repeat(depth);

    for field in operation_tree.fields.values() {
        match field {
            OperationField::Field(value) => {
                output.push_str(indentation.as_str());
                output.push_str(&value);
                output.push_str("\n");
            }
            OperationField::Selection(selection_set) => {
                output.push_str(&format!("{indentation}{}: ", selection_set.field_name).as_str());

                if selection_set.is_list {
                    output.push_str("Array<");
                }

                output.push_str("{");
                output.push_str("\n");

                output.push_str(&render_operation_tree(&selection_set, depth + 1));

                output.push_str("\n");
                output.push_str(indentation.as_str());

                output.push_str("}");

                if selection_set.is_list {
                    output.push_str(">");
                }

                if !selection_set.is_non_null {
                    output.push_str(" | null");
                }

                output.push_str(";");

                output.push_str("\n");
            }
        }
    }

    output
}
