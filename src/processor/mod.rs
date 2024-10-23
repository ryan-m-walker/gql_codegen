use apollo_compiler::ApolloCompiler;
use apollo_parser::Parser;

use crate::generator::CodeGenerator;

pub struct DocumentSource<'a> {
    document: &'a str,
    filepath: &'a str,
}

pub struct Processor<'a, 'b> {
    pub schema: &'a str,
    pub documents: Vec<DocumentSource<'b>>,
    generators: Vec<Box<dyn CodeGenerator>>,
}

impl<'a, 'b> Processor<'a, 'b> {
    pub fn new(schema: &'a str) -> Self {
        Self {
            schema,
            documents: Vec::new(),
            generators: Vec::new(),
        }
    }

    pub fn add_generator(&mut self, generator: Box<dyn CodeGenerator>) -> &mut Self {
        self.generators.push(generator);
        self
    }

    pub fn add_document(&mut self, filepath: &'b str, document: &'b str) -> &mut Self {
        self.documents.push(DocumentSource { document, filepath });
        self
    }

    pub fn process(&mut self) -> Option<String> {
        let mut compiler = ApolloCompiler::new();
        compiler.add_type_system(self.schema, "schema.graphql");

        for document in &self.documents {
            compiler.add_document(document.document, document.filepath);
        }

        let diagnostics = compiler.validate();

        let mut is_error = false;

        if diagnostics.len() > 0 {
            for diagnostic in diagnostics {
                if diagnostic.data.is_error() {
                    is_error = true;
                    println!("{}", diagnostic);
                }
            }

            if is_error {
                return None;
            }
        }

        let mut output = String::new();

        for generator in &self.generators {
            let schema = Parser::new(self.schema).parse().document();
            output.push_str(&generator.generate(&schema, &compiler.db));

            for source in &self.documents {
                let document = Parser::new(source.document).parse().document();
                output.push_str(&generator.generate(&document, &compiler.db));
            }
        }

        Some(output)
    }
}
