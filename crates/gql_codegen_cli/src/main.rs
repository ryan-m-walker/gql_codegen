use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use args::Args;
use clap::Parser;

use gql_codegen_config::{Config, Generator};
use gql_codegen_generators::{
    documents::generate_documents, ts_operation_types::generate_operation_types,
    ts_schema_types::generate_ts_schema_types,
};

use anyhow::{Context, Result};
use apollo_compiler::{Name, ast::Definition, parser};
use gql_codegen_logger::{LogLevel, Logger};
use gql_codegen_types::{FragmentResult, OperationResult};
use indexmap::IndexMap;
use rayon::{
    current_thread_index,
    iter::{IntoParallelRefIterator, ParallelIterator},
};

use crate::{get_read_results::get_read_results, get_schema::get_schema, path_parser::expand_path};

mod args;
mod collector;
mod file;
mod get_read_results;
mod get_schema;
mod path_parser;

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
    let schema = get_schema(&config, logger)?;
    let read_results = get_read_results(&config, logger)?;

    let mut fragment_map: IndexMap<Name, FragmentResult> = IndexMap::new();
    let mut operations_map: IndexMap<Name, OperationResult> = IndexMap::new();
    let mut file_path_map: IndexMap<PathBuf, Vec<Name>> = IndexMap::new();
    let mut anonymous_operation_count = 1;
    let mut document_count = 0;

    for read_result in read_results {
        if let Some(result) = read_result? {
            for document in &result.documents {
                let ast = parser::Parser::new()
                    .parse_ast(document, &result.path)
                    .unwrap(); // TODO: handle errors

                document_count += 1;

                for definition in ast.definitions {
                    match definition {
                        Definition::OperationDefinition(operation) => {
                            if let Some(name) = &operation.name {
                                if operations_map.contains_key(name) {
                                    logger.warn(&format!("Duplicate operation name \"{name}\" found in file \"{}\". Skipping.", result.path.display()));
                                    continue;
                                }

                                let temp_path = expand_path(
                                    Path::new(""),
                                    result.path.as_ref(),
                                    name,
                                    &operation.operation_type,
                                );

                                if !file_path_map.contains_key(&temp_path) {
                                    file_path_map.insert(temp_path.clone(), vec![]);
                                }

                                file_path_map
                                    .get_mut(&temp_path)
                                    .unwrap()
                                    .push(name.clone());

                                operations_map.insert(
                                    name.clone(),
                                    OperationResult::new(
                                        operation,
                                        ast.sources.clone(),
                                        result.path.clone(),
                                    ),
                                );

                                continue;
                            }

                            operations_map.insert(
                                Name::new(&format!(
                                    "AnonymousOperation{anonymous_operation_count}"
                                ))?,
                                OperationResult::new(
                                    operation,
                                    ast.sources.clone(),
                                    result.path.clone(),
                                ),
                            );

                            anonymous_operation_count += 1;
                        }
                        Definition::FragmentDefinition(fragment) => {
                            if fragment.name.as_str() == "UserPhoneSettingsContentSection" {
                                panic!("FOUND");
                            }
                            if fragment_map.contains_key(&fragment.name) {
                                logger.warn(&format!(
                                    "Duplicate fragment name \"{}\" found in file \"{}\". Skipping.",
                                    fragment.name, result.path.display()
                                ));
                                continue;
                            }

                            fragment_map.insert(
                                fragment.name.clone(),
                                FragmentResult::new(
                                    fragment,
                                    ast.sources.clone(),
                                    result.path.clone(),
                                ),
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    logger.info(&format!("{document_count} documents found."));

    let codegen_results = config
        .outputs
        .par_iter()
        .map(|(output_path, output_config)| -> Result<()> {
            let thread_index = current_thread_index().unwrap_or_default();
            logger.debug(&format!(
                "Generating {output_path} in thread {thread_index}"
            ));

            // TODO: use src for output path
            let path = PathBuf::from(&output_path);

            if let Some(parent) = path.parent() {
                dbg!(parent);
                fs::create_dir_all(parent)?;
            }

            let mut writer = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(output_path)
                .context("Failed to write output file.")?;

            if let Some(config) = &output_config.prelude {
                writeln!(writer, "{config}\n")?;
            }

            // let format_config = output_config.formatting.unwrap_or_default();
            // let formatter = Formatter::from_config(format_config);
            // let ctx =
            //     gql_codegen_types::Context::new(&schema, &operations_map, &fragments_map, logger);

            for generator in &output_config.generators {
                match generator {
                    Generator::TsSchemaTypes { config } => {
                        generate_ts_schema_types(
                            &mut writer,
                            &schema,
                            &operations_map,
                            &fragment_map,
                            config,
                            logger,
                        )?;
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
                    Generator::Documents { config } => {
                        generate_documents(
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
