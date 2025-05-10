use std::mem;

use oxc::{
    ast::ast::{CallExpression, Expression, TaggedTemplateExpression},
    ast_visit::Visit,
};

#[derive(Debug, Default)]
pub(crate) struct JSVisitor {
    documents: Vec<String>,
}

impl<'a> Visit<'a> for JSVisitor {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Expression::Identifier(id) = &it.callee {
            if id.name == "graphql" || id.name == "gql" {
                // TODO?
            }
        }
    }

    fn visit_tagged_template_expression(&mut self, it: &TaggedTemplateExpression<'a>) {
        if let Expression::Identifier(id) = &it.tag {
            if id.name != "graphql" && id.name != "gql" {
                return;
            }

            if !it.quasi.expressions.is_empty() || it.quasi.quasis.len() > 1 {
                panic!("Tagged template expression with expressions");
            }

            let quasi = &it.quasi.quasis[0];
            let raw = quasi.value.raw.into();
            self.documents.push(raw);
        }
    }
}

impl JSVisitor {
    pub fn take_output(&mut self) -> Vec<String> {
        mem::take(&mut self.documents)
    }
}
