//! Speedy GraphQL Codegen CLI
//!
//! A fast, Rust-powered GraphQL code generator.

mod logger;

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Mutex;

use anyhow::{Context, Result};
use clap::Parser;
use gql_codegen_core::{
    CodegenConfig, ExtractConfig, GenerateInput, SourceCache,
    cache::{Cache, FsCache, NoCache},
    collect_documents, generate_from_input, load_schema, load_sources,
};
use rayon::prelude::*;

use crate::logger::{LogLevel, Logger};

#[derive(Parser, Debug)]
#[command(name = "sgc")]
#[command(about = "Speedy GraphQL Codegen - A fast GraphQL code generator")]
#[command(version)]
struct Args {
    /// Path to the config file (JSON)
    #[arg(short, long, default_value = "codegen.json")]
    config: PathBuf,

    /// Check mode - validate without writing files
    #[arg(long)]
    check: bool,

    /// Disable caching (always regenerate)
    #[arg(long)]
    no_cache: bool,

    /// Clear the cache directory and exit
    #[arg(long)]
    clean: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Suppress output (only show errors)
    #[arg(short, long)]
    quiet: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();

    let log_level = if args.quiet {
        LogLevel::Quiet
    } else if args.verbose {
        LogLevel::Verbose
    } else {
        LogLevel::Normal
    };

    let logger = Logger::new(log_level);

    match run(&args, &logger) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            logger.error(&e.to_string());
            if args.verbose {
                for cause in e.chain().skip(1) {
                    eprintln!("  Caused by: {cause}");
                }
            }
            ExitCode::FAILURE
        }
    }
}

fn run(args: &Args, logger: &Logger) -> Result<()> {
    let config_content = fs::read_to_string(&args.config)
        .with_context(|| format!("Failed to read config: {}", args.config.display()))?;

    let mut config: CodegenConfig = serde_json::from_str(&config_content)
        .with_context(|| format!("Failed to parse config: {}", args.config.display()))?;

    let base_dir = resolve_base_dir(&args.config);
    config.base_dir = Some(base_dir.to_string_lossy().into_owned());

    logger.debug(&format!("Config: {}", args.config.display()));
    logger.debug(&format!("Base: {}", base_dir.display()));

    // Handle --clean flag
    if args.clean {
        let cache_dir = base_dir.join(".sgc");
        if cache_dir.exists() {
            fs::remove_dir_all(&cache_dir)
                .with_context(|| format!("Failed to remove: {}", cache_dir.display()))?;
            logger.success("Cache cleared");
        } else {
            logger.success("Cache already clean");
        }
        return Ok(());
    }

    let mut cache: Box<dyn Cache> = if args.no_cache {
        Box::new(NoCache)
    } else {
        Box::new(FsCache::new(base_dir.join(".sgc")))
    };

    // Check cache - skip if nothing changed
    if cache.check(&config, &config_content, &base_dir) {
        logger.success("Nothing changed");
        return Ok(());
    }

    logger.debug("Cache miss - regenerating...");

    let schema = load_schema(&config.schema, Some(&base_dir))?;
    let mut source_cache = SourceCache::new();
    load_sources(&config.documents, Some(&base_dir), &mut source_cache)?;

    let extract_config = ExtractConfig::default();
    let documents = collect_documents(&source_cache, &extract_config);

    for warning in &documents.warnings {
        logger.warn(warning);
    }

    let input = GenerateInput {
        schema: &schema,
        documents: &documents,
        generates: &config.generates,
    };
    let result = generate_from_input(&input)?;

    if !args.check {
        write_outputs(&result.files, logger)?;
    }

    let action = if args.check {
        "Would generate"
    } else {
        "Generated"
    };
    let count = result.files.len();
    let plural = if count == 1 { "" } else { "s" };
    logger.success(&format!("{action} {count} file{plural}"));

    // Commit cache after successful generation
    cache.commit();
    cache.flush().ok();

    Ok(())
}

fn resolve_base_dir(config_path: &Path) -> PathBuf {
    config_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn write_outputs(files: &[gql_codegen_core::GeneratedFile], logger: &Logger) -> Result<()> {
    // First pass: collect and create all unique parent directories (sequential to avoid races)
    let directories: HashSet<PathBuf> = files
        .iter()
        .filter_map(|f| {
            let path = PathBuf::from(&f.path);
            path.parent()
                .filter(|p| !p.as_os_str().is_empty())
                .map(|p| p.to_path_buf())
        })
        .collect();

    for dir in &directories {
        if !dir.exists() {
            fs::create_dir_all(dir)
                .with_context(|| format!("Failed to create: {}", dir.display()))?;
        }
    }

    // Second pass: parallel write all files
    // Use Mutex for logger since it's not Sync
    let logger = Mutex::new(logger);
    let errors: Vec<_> = files
        .par_iter()
        .filter_map(|file| {
            let path = PathBuf::from(&file.path);
            match fs::write(&path, &file.content) {
                Ok(()) => {
                    if let Ok(l) = logger.lock() {
                        l.file(&path.display().to_string());
                    }
                    None
                }
                Err(e) => Some(format!("Failed to write {}: {}", path.display(), e)),
            }
        })
        .collect();

    if let Some(first_error) = errors.into_iter().next() {
        anyhow::bail!(first_error);
    }

    Ok(())
}
