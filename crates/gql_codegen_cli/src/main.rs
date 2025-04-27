use args::Args;
use clap::Parser;
use file::{FileType, get_file_type};
use glob::glob;

use gql_codegen_config::Config;
use gql_codegen_errors::CodegenError;
use gql_codegen_js::JSParser;
use oxc::allocator::Allocator;

mod args;
mod file;

fn main() -> Result<(), CodegenError> {
    let args = Args::parse();
    let config = Config::from_path(&args.config);
    let allocator = Allocator::default();

    let Ok(matches) = glob(&config.documents) else {
        println!("Invalid glob pattern: {}", config.documents);
        return CodegenError::InvalidGlobPattern(config.documents);
    };

    for entry in matches {
        let Ok(path) = entry else {
            println!("Error reading file");
            continue;
        };

        let Some(extension) = path.extension() else {
            println!("File has no extension");
            continue;
        };

        let extension = extension.to_string_lossy();

        let Some(file_type) = get_file_type(&extension) else {
            println!("Unsupported file type: {extension}");
            return CodegenError::InvalidFileType(extension.to_string());
        };

        match file_type {
            FileType::GraphQL => {
                //
            }
            FileType::JavaScript | FileType::TypeScript => {
                let parser = JSParser::new(&allocator, path);
                let document = parser.parse().unwrap();
                println!("Parsed document: {}", document);
            }
        }
    }

    Ok(())
}
