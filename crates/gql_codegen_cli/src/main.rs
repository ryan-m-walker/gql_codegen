//! Speedy GraphQL Codegen CLI
//!
//! A fast, Rust-powered GraphQL code generator.

mod logger;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::Parser;
use gql_codegen_core::{
    CodegenConfig, FsWriter, GenerateCachedResult, StdoutWriter,
    cache::{Cache, FsCache, NoCache},
    generate_cached, write_outputs,
};

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

    /// Print generated output to stdout instead of writing files
    #[arg(long)]
    stdout: bool,

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

    // Use baseDir from config if set (e.g., from Node CLI), otherwise derive from config path
    let base_dir = config
        .base_dir
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| resolve_base_dir(&args.config));
    config.base_dir = Some(base_dir.to_string_lossy().into_owned());

    logger.debug(&format!("Config: {}", args.config.display()));
    logger.debug(&format!("Base: {}", base_dir.display()));

    let mut cache: Box<dyn Cache> = if args.no_cache {
        Box::new(NoCache)
    } else {
        Box::new(FsCache::new(base_dir.join(".sgc")))
    };

    // Handle --clean flag
    if args.clean {
        let did_clear = cache.clear().context("Failed to clear cache")?;

        if did_clear {
            logger.success("Cache cleared");
        } else {
            logger.success("Cache already clean");
        }

        return Ok(());
    }

    let result = generate_cached(&config, cache.as_mut())?;

    match result {
        GenerateCachedResult::Fresh => {
            logger.success("Nothing changed");
        }
        GenerateCachedResult::Generated(gen_result) => {
            for warning in &gen_result.warnings {
                logger.warn(warning);
            }

            if !args.check {
                let write_result = if args.stdout {
                    let writer = StdoutWriter::new();
                    write_outputs(&gen_result.files, &writer)
                } else {
                    let writer = FsWriter::new();
                    let result = write_outputs(&gen_result.files, &writer);

                    // Log written files (only for fs writer)
                    for path in &result.written {
                        logger.file(&path.display().to_string());
                    }

                    result
                };

                // Handle errors
                if !write_result.is_success() {
                    let (path, err) = &write_result.errors[0];
                    anyhow::bail!("Failed to write {}: {}", path.display(), err);
                }
            }

            let action = if args.check {
                "Would generate"
            } else {
                "Generated"
            };

            let count = gen_result.files.len();
            let plural = if count == 1 { "" } else { "s" };

            logger.success(&format!("{action} {count} file{plural}"));
        }
    }

    Ok(())
}

fn resolve_base_dir(config_path: &Path) -> PathBuf {
    config_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}
