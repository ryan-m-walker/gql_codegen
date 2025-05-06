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
use gql_codegen_generators::{
    ts_operation_types::generate_operation_types, ts_schema_types::generate_ts_schema_types,
};
use gql_codegen_js::parse_from_js_file;

use anyhow::{Context, Result, anyhow};
use apollo_compiler::Schema;
use colored::Colorize;
use gql_codegen_types::ReadResult;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

mod args;
mod file;
mod generate;

fn main() {
    if let Err(e) = run_cli() {
        eprintln!("{}", format!("Error ✖︎ {}", e));
        std::process::exit(1);
    }
}

fn run_cli() -> Result<()> {
    let args = Args::parse();
    let config = Config::from_path(&args.config);

    let schema_path = PathBuf::from(&config.schema);

    let schema_source = fs::read_to_string(&schema_path)
            .context("Failed to read schema file. Please ensure that your configuration schema value is pointing to a valid file.")?;

    // TODO: validation errors reporting
    let schema = Schema::parse_and_validate(schema_source, &schema_path).unwrap();

    println!("Scanning for documents...");

    let matches = glob(&config.documents).context(
        "Invalid documents glob pattern. Please check your \"documents\" configuraton value.",
    )?;

    let matches_vec = matches.collect::<Vec<_>>();

    println!("Found {} files", matches_vec.len());

    let read_results = matches_vec
        .par_iter()
        .map(|entry| -> Result<Option<ReadResult>> {
            let Ok(path) = entry else {
                return Err(anyhow!("Invalid glob pattern."));
            };

            let Some(extension) = path.extension() else {
                // TODO: better error message
                return Err(anyhow!("File has no extension: {}", path.display()));
            };

            let extension = extension.to_string_lossy();

            let Some(file_type) = get_file_type(&extension) else {
                return Err(anyhow!("Unsupported file type: {extension}"));
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
                    let documents = parse_from_js_file(path.to_path_buf());

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

    let mut checked_read_results = Vec::new();

    for result in read_results {
        match result? {
            Some(result) => checked_read_results.push(result),
            None => {}
        }
    }

    println!("Found {} documents", checked_read_results.len());

    config
        .outputs
        .par_iter()
        .for_each(|(output_path, output_config)| {
            // TODO: create output directory if it doesn't exist

            let mut writer = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(output_path)
                .unwrap();

            // let mut writer = std::io::stdout();

            if let Some(config) = &output_config.prelude {
                writeln!(writer, "{config}\n").unwrap();
            }

            for generator in &output_config.generators {
                match generator {
                    Generator::TsSchemaTypes { config } => {
                        generate_ts_schema_types(&mut writer, &schema, config).unwrap();
                    }
                    Generator::TsOperationTypes { config } => {
                        generate_operation_types(
                            &mut writer,
                            &schema,
                            &checked_read_results,
                            config,
                        )
                        .unwrap();
                    }
                }
            }
        });

    Ok(())
}

fn make_error() -> Result<()> {
    Err(anyhow!("Error!"))
}
