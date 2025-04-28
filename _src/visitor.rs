use apollo_parser::Parser;
use oxc::{
    ast::ast::{CallExpression, Expression, TaggedTemplateExpression},
    ast_visit::Visit,
};

pub struct JSVisitor;

impl Default for JSVisitor {
    fn default() -> Self {
        Self
    }
}

// TODO: handle /** GraphQL */ comments

impl<'a> Visit<'a> for JSVisitor {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Expression::Identifier(id) = &it.callee {
            if id.name == "graphql" || id.name == "gql" {
                dbg!(it);
            }
        }
    }

    fn visit_tagged_template_expression(&mut self, it: &TaggedTemplateExpression<'a>) {
        if let Expression::Identifier(id) = &it.tag {
            if id.name != "graphql" && id.name != "gql" {
                return;
            }

            // TODO: better error handling
            if !it.quasi.expressions.is_empty() || it.quasi.quasis.len() > 1 {
                panic!("Tagged template expression with expressions");
            }

            let quasi = &it.quasi.quasis[0];
            let raw = quasi.value.raw.as_str();
            self.parse_raw(raw);
        }
    }
}

impl JSVisitor {
    fn parse_raw(&self, raw: &str) {
        let parser = Parser::new(raw);
        let cst = parser.parse();

        let doc = cst.document();
        dbg!(doc);
    }
}
