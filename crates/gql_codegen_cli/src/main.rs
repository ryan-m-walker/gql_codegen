use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

use args::Args;
use clap::Parser;
use file::{FileType, get_file_type};
use glob::glob;

use gql_codegen_config::{Config, Generator};
use gql_codegen_generators::{
    ts_operation_types::generate_operation_types, ts_schema_types::generate_ts_schema_types,
};
use gql_codegen_js::parse_from_js_file;

use anyhow::{Context, Result, anyhow};
use apollo_compiler::{
    Name, Node, Schema,
    ast::{Definition, FragmentDefinition, OperationDefinition},
    parser,
};
use gql_codegen_logger::{LogLevel, Logger};
use gql_codegen_types::ReadResult;
use rayon::{
    current_thread_index,
    iter::{IntoParallelRefIterator, ParallelIterator},
};

mod args;
mod file;
mod generate;

fn main() {
    let args = Args::parse();
    let logger = Logger::new(LogLevel::Debug);
    println!();

    if let Err(e) = run_cli(&args, &logger) {
        if logger.level == LogLevel::Debug {
            logger.error(&format!("{e:?}"));
        } else {
            logger.error(&format!("{e}"));
        }
        println!();

        std::process::exit(1);
    }

    logger.info("Code generation completed.");
    println!();
}

fn run_cli(args: &Args, logger: &Logger) -> Result<()> {
    let config = Config::from_path(&args.config);

    let mut schema = Schema::builder();

    for schema_path in &config.schemas {
        let path = PathBuf::from(schema_path);

        logger.info("Parsing schema file...");
        logger.debug(&format!(
            "Using schema filepath path {}",
            path.to_string_lossy()
        ));

        let schema_source = fs::read_to_string(&schema_path)
                .context("Failed to read schema file. Please ensure that your configuration schema value is pointing to a valid file.")?;
        schema = schema.parse(schema_source, path);
    }

    let schema = match schema.build().unwrap().validate() {
        Ok(valid) => valid,
        Err(with_errors) => {
            let mut message = String::from("Error parsing schema:\n");

            for error in with_errors.errors.iter() {
                // TODO: show sources
                message.push_str(&format!("{}", error.error));
            }

            return Err(anyhow!(message));
        }
    };

    logger.info("Scanning for documents...");

    let matches = glob(&config.documents).context(
        "Invalid documents glob pattern. Please check your \"documents\" configuraton value.",
    )?;

    let matches_vec = matches.collect::<Vec<_>>();
    let file_count = matches_vec.len();

    logger.info(&format!(
        "Found {} {}.",
        file_count,
        pluralize("file", file_count)
    ));

    let read_results = matches_vec
        .par_iter()
        .map(|entry| -> Result<Option<ReadResult>> {
            let Ok(path) = entry else {
                return Err(anyhow!("Invalid glob pattern."));
            };

            let Some(extension) = path.extension() else {
                logger.warn(&format!("Encountered a file with no extension: \"{}\"", path.display()));
                return Ok(None);
            };

            let extension = extension.to_string_lossy();

            let Some(file_type) = get_file_type(&extension) else {
                logger.warn(&format!("Encountered a file with an unsupported file extension: \"{extension}\" for file \"{}\"", path.display()));
                logger.warn("Please make sure the file has either a valid GraphQL, JavaScript or TypeScript extension.");
                return Ok(None);
            };

            match file_type {
                FileType::GraphQL => {
                    let document = fs::read_to_string(path).with_context(|| {
                        format!("Failed to read file {}", path.to_string_lossy())
                    })?;

                    Ok(Some(ReadResult {
                        path: path.to_string_lossy().to_string(),
                        documents: vec![document],
                    }))
                }
                FileType::JavaScript | FileType::TypeScript => {
                    let documents = parse_from_js_file(path.to_path_buf())?;

                    if documents.is_empty() {
                        return Ok(None);
                    }

                    Ok(Some(ReadResult {
                        path: path.to_string_lossy().to_string(),
                        documents,
                    }))
                }
            }
        })
        .collect::<Vec<_>>();

    let mut fragment_map: HashMap<Name, Node<FragmentDefinition>> = HashMap::new();
    let mut operations_map: HashMap<Name, Node<OperationDefinition>> = HashMap::new();
    let mut anonymous_operation_count = 0;

    for read_result in read_results {
        if let Some(result) = read_result? {
            for (i, document) in result.documents.iter().enumerate() {
                let path = if i == 0 {
                    result.path.clone()
                } else {
                    format!("{}#{}", result.path, i + 1)
                };

                let ast = parser::Parser::new().parse_ast(document, &path).unwrap();

                for definition in ast.definitions {
                    match definition {
                        Definition::OperationDefinition(operation) => {
                            if let Some(name) = &operation.name {
                                if operations_map.contains_key(name) {
                                    logger.warn(&format!("Duplicate operation name \"{name}\" found in file \"{}\". Skipping.", result.path));
                                    continue;
                                }

                                operations_map.insert(name.clone(), operation);
                                continue;
                            }

                            operations_map.insert(
                                Name::new(&format!(
                                    "AnonymousOperation{anonymous_operation_count}"
                                ))?,
                                operation,
                            );

                            anonymous_operation_count += 1;
                        }
                        Definition::FragmentDefinition(fragment) => {
                            if fragment_map.contains_key(&fragment.name) {
                                logger.warn(&format!(
                                    "Duplicate fragment name \"{}\" found in file \"{}\". Skipping.",
                                    fragment.name, result.path
                                ));
                                continue;
                            }

                            fragment_map.insert(fragment.name.clone(), fragment);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    let codegen_results = config
        .outputs
        .par_iter()
        .map(|(output_path, output_config)| -> Result<()> {
            let thread_index = current_thread_index().unwrap_or_default();
            logger.debug(&format!(
                "Generating {output_path} in thread {thread_index}"
            ));

            // TODO: create output directory if it doesn't exist

            let mut writer = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(output_path)
                .context("Failed to write output file.")?;

            if let Some(config) = &output_config.prelude {
                writeln!(writer, "{config}\n")?;
            }
            for generator in &output_config.generators {
                match generator {
                    Generator::TsSchemaTypes { config } => {
                        generate_ts_schema_types(&mut writer, &schema, config, logger)?;
                    }
                    Generator::TsOperationTypes { config } => {
                        generate_operation_types(
                            &mut writer,
                            &schema,
                            &operations_map,
                            &fragment_map,
                            config,
                            logger,
                        )?;
                    }
                }
            }

            Ok(())
        })
        .collect::<Vec<_>>();

    for result in codegen_results {
        if result.as_ref().is_err() {
            return result;
        }
    }

    Ok(())
}

fn pluralize(word: &str, count: usize) -> String {
    if count == 1 {
        return word.to_string();
    }

    format!("{word}s")
}
