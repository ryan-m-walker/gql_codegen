use std::{
    fs::{self},
    path::Path,
};

use anyhow::{Context, Result};
use globset::GlobBuilder;
use gql_codegen_config::Config;
use gql_codegen_js::parse_from_js_file;
use gql_codegen_logger::Logger;
use gql_codegen_types::ReadResult;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use walkdir::{DirEntry, WalkDir};

use crate::file::{FileType, get_file_type};

fn skip_dir(entry: &DirEntry) -> bool {
    entry
        .path()
        .components()
        .any(|c| c.as_os_str() == "node_modules")
}

type ReturnType = Result<Vec<Result<Option<ReadResult>>>>;

pub fn get_read_results(config: &Config, logger: &Logger) -> ReturnType {
    logger.info("Scanning for documents...");

    let globset = GlobBuilder::new(&config.documents)
        .case_insensitive(false)
        .build()?
        .compile_matcher();

    let root = Path::new(&config.src);

    let mut entries_vec = Vec::new();
    let walker = WalkDir::new(root).into_iter();

    for entry in walker.filter_entry(|e| !skip_dir(e)) {
        let Ok(entry) = entry else {
            continue;
        };

        let path = entry.path();

        if let Some(path_str) = path.to_str() {
            if globset.is_match(path_str) {
                entries_vec.push(path.to_path_buf());
            }
        }
    }

    let read_results = entries_vec
        .par_iter()
        .map(|path| -> Result<Option<ReadResult>> {
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
                        path: path.to_path_buf(),
                        documents: vec![document],
                    }))
                }
                FileType::JavaScript | FileType::TypeScript => {
                    let documents = parse_from_js_file(path.to_path_buf())?;

                    if documents.is_empty() {
                        return Ok(None);
                    }

                    Ok(Some(ReadResult {
                        path: path.to_path_buf(),
                        documents,
                    }))
                }
            }
        })
        .collect::<Vec<_>>();

    Ok(read_results)
}
