use std::fs;

use glob::glob;
use oxc::{
    allocator::Allocator,
    ast_visit::Visit,
    parser::{Parser, ParserReturn},
    span::SourceType,
};
use visitor::JSVisitor;

mod visitor;

fn main() {
    let matches = glob("test").unwrap();
    let allocator = Allocator::default();

    for entry in matches {
        let Ok(path) = entry else {
            println!("Error reading file");
            continue;
        };

        let mut visitor = JSVisitor;
        let extension = path.extension().unwrap_or_default();

        if extension == "gql" || extension == "graphql" {
            // TODO: handle raw parsing
        }

        let Ok(source_type) = SourceType::from_path(&path) else {
            println!("Unexpected file extension: {}", extension.display());
            continue;
        };

        let source_text = fs::read_to_string(&path).unwrap();
        let parser = Parser::new(&allocator, source_text.as_str(), source_type);
        let ParserReturn { program, .. } = parser.parse();

        if program.is_empty() {
            continue;
        }

        visitor.visit_program(&program);
    }
}
