use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use oxc::{allocator::Allocator, ast_visit::Visit, parser::Parser, span::SourceType};
use visitor::JSVisitor;

mod visitor;

pub struct JSParser {
    source_path: PathBuf,
}

impl JSParser {
    pub fn new(source_path: PathBuf) -> Self {
        Self { source_path }
    }

    pub fn parse(&self) -> Result<Vec<String>> {
        let allocator = Allocator::default();

        let source_type = SourceType::from_path(&self.source_path)?;

        let source_text = fs::read_to_string(&self.source_path).with_context(|| {
            format!(
                "Failed to read file {}",
                &self.source_path.to_string_lossy()
            )
        })?;

        let parser = Parser::new(&allocator, source_text.as_str(), source_type);
        let parse_result = parser.parse();

        if parse_result.program.is_empty() {
            return Ok(vec![]);
        }

        let mut visitor = JSVisitor::default();
        visitor.visit_program(&parse_result.program);
        Ok(visitor.take_output())
    }
}

pub fn parse_from_js_file(source_path: PathBuf) -> Result<Vec<String>> {
    let parser = JSParser::new(source_path);
    parser.parse()
}
