use std::mem;

use oxc::{
    ast::ast::{CallExpression, Expression, TaggedTemplateExpression},
    ast_visit::{walk, Visit},
};

#[derive(Debug, Default)]
pub(crate) struct JSVisitor {
    documents: Vec<String>,
}

impl JSVisitor {
    fn extract_document(&mut self, expr: &TaggedTemplateExpression) {
        if let Expression::Identifier(id) = &expr.tag {
            if id.name != "graphql" && id.name != "gql" {
                return;
            }

            if !expr.quasi.expressions.is_empty() || expr.quasi.quasis.len() > 1 {
                panic!("Tagged template expression with expressions");
            }

            let quasi = &expr.quasi.quasis[0];
            let raw = quasi.value.raw.into();
            self.documents.push(raw);
        }
    }
}

impl<'a> Visit<'a> for JSVisitor {
    fn visit_tagged_template_expression(&mut self, expr: &TaggedTemplateExpression<'a>) {
        self.extract_document(expr);
    }

    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        walk::walk_call_expression(self, expr);
    }
}

impl JSVisitor {
    pub fn take_output(&mut self) -> Vec<String> {
        mem::take(&mut self.documents)
    }
}
