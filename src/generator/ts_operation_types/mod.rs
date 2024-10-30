use apollo_compiler::{HirDatabase, RootDatabase};

use self::{operation_tree::build_operation_tree, render_tree::render_operation_tree};

use super::CodeGenerator;

pub(self) mod operation_tree;
pub(self) mod render_tree;

pub struct TsOperationsTypeGenerator;

impl CodeGenerator for TsOperationsTypeGenerator {
    fn generate(&self, db: &RootDatabase) -> String {
        let mut result = String::new();

        for operation in db.all_operations().as_ref() {
            if let Some(tree) = build_operation_tree(&operation, db) {
                result.push_str(&format!("export type {} = {{\n", operation.name().unwrap()));
                let rendered = render_operation_tree(&tree, 1);
                result.push_str(&rendered);
                result.push_str("\n};\n\n");
            }
        }

        result
    }
}
