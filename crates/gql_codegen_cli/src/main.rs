use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

use args::Args;
use clap::Parser;
use file::{FileType, get_file_type};
use glob::glob;

use gql_codegen_config::{Config, Generator};
use gql_codegen_errors::CodegenError;
use gql_codegen_generators::{
    ts_operation_types::generate_operation_types, ts_schema_types::generate_ts_schema_types,
};
use gql_codegen_js::parse_from_js_file;

use apollo_compiler::{Schema, parser};
use gql_codegen_types::ReadResult;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

mod args;
mod file;
mod generate;

fn main() -> Result<(), CodegenError> {
    let args = Args::parse();
    let config = Config::from_path(&args.config);

    let schema_path = PathBuf::from(&config.schema);
    let Ok(schema_source) = fs::read_to_string(&schema_path) else {
        println!("Unable to read schema file");
        return Err(CodegenError::FileReadError);
    };

    // let schema = Schema::parse_and_validate(schema_source, &schema_path).unwrap();
    // SchemaBuilder::new().

    let gql_parser = parser::Parser::default();

    let mut schema_builder = Schema::builder();
    gql_parser.parse_into_schema_builder(schema_source, &schema_path, &mut schema_builder);

    println!("Scanning for documents...");

    let Ok(matches) = glob(&config.documents) else {
        println!("Invalid glob pattern: {}", config.documents);
        return Err(CodegenError::InvalidGlobPattern(config.documents));
    };

    let matches_vec = matches.collect::<Vec<_>>();

    println!("Found {} files", matches_vec.len());

    let read_results = matches_vec
        .par_iter()
        .map(|entry| {
            let Ok(path) = entry else {
                panic!("Invalid glob pattern: {}", config.documents);
            };

            let Some(extension) = path.extension() else {
                panic!("File has no extension: {}", path.display());
            };

            let extension = extension.to_string_lossy();

            let Some(file_type) = get_file_type(&extension) else {
                panic!("Unsupported file type: {extension}");
            };

            match file_type {
                FileType::GraphQL => {
                    let Ok(document) = fs::read_to_string(path) else {
                        panic!("Unable to read file: {}", path.to_string_lossy());
                    };

                    Some(ReadResult {
                        path: path.to_string_lossy().to_string(),
                        documents: vec![document],
                    })
                }
                FileType::JavaScript | FileType::TypeScript => {
                    let documents = parse_from_js_file(path.to_path_buf());

                    if documents.is_empty() {
                        return None;
                    }

                    Some(ReadResult {
                        path: path.to_string_lossy().to_string(),
                        documents,
                    })
                }
            }
        })
        .filter_map(|entry| entry)
        .collect::<Vec<_>>();

    println!("Found {} documents", read_results.len());

    for read_result in &read_results {
        for (i, source_text) in read_result.documents.iter().enumerate() {
            let path = if i == 0 {
                read_result.path.clone()
            } else {
                format!("{}#{}", read_result.path, i + 1)
            };

            let ast = gql_parser.parse_ast(source_text, path).unwrap();
            schema_builder.add_ast(&ast);
        }
    }

    let schema = schema_builder.build().unwrap();

    config
        .outputs
        .par_iter()
        .for_each(|(output_path, output_config)| {
            // TODO: create output directory if it doesn't exist

            let mut _writer = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(output_path)
                .unwrap();

            let mut writer = std::io::stdout();

            if let Some(config) = &output_config.prelude {
                writeln!(writer, "{config}\n").unwrap();
            }

            for generator in &output_config.generators {
                match generator {
                    Generator::TsSchemaTypes { config } => {
                        generate_ts_schema_types(&mut writer, &schema, config).unwrap();
                    }
                    Generator::TsOperationTypes { config } => {
                        generate_operation_types(&mut writer, &schema, &read_results, config)
                            .unwrap();
                    }
                }
            }
        });

    Ok(())
}
