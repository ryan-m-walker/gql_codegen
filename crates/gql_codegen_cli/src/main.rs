//! Speedy GraphQL Codegen CLI
//!
//! A fast, Rust-powered GraphQL code generator.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::Parser;
use gql_codegen_core::cache::{Cache, FsCache, NoCache};
use gql_codegen_core::diagnostic::{
    self, Color, DEFAULT_MAX_DIAGNOSTICS, Diagnostic, DiagnosticCategory, DiagnosticLocation,
    Diagnostics,
};
use gql_codegen_core::{
    CodegenConfig, FsWriter, GenerateCachedResult, StdoutWriter, generate_cached, write_outputs,
};

mod args;
mod load_schema;
mod logger;

use crate::args::CliArgs;
use crate::logger::{LogLevel, Logger};

fn main() -> ExitCode {
    let args = CliArgs::parse();

    if args.timing {
        gql_codegen_core::timing::enable_timing();
    }

    let log_level = if args.quiet {
        LogLevel::Quiet
    } else if args.verbose {
        LogLevel::Verbose
    } else {
        LogLevel::Normal
    };

    let logger = Logger::new(log_level);

    let max_diag = args.max_diagnostics.unwrap_or(DEFAULT_MAX_DIAGNOSTICS);

    match run(&args, &logger) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            // Try to render structured diagnostics for core errors
            if let Some(diagnostics) = e.downcast_ref::<Diagnostics>() {
                let color = Color::StderrIsTerminal;
                let _ = diagnostic::render_diagnostics(
                    diagnostics,
                    None,
                    color,
                    max_diag,
                    &mut std::io::stderr(),
                );
            } else {
                logger.error(&e.to_string());
                if args.verbose {
                    for cause in e.chain().skip(1) {
                        eprintln!("  Caused by: {cause}");
                    }
                }
            }
            ExitCode::FAILURE
        }
    }
}

fn run(args: &CliArgs, logger: &Logger) -> Result<()> {
    let config_content = fs::read_to_string(&args.config)
        .with_context(|| format!("Failed to read config: {}", args.config.display()))?;

    let mut config: CodegenConfig = serde_json::from_str(&config_content).map_err(|e| {
        let d = Diagnostic::error(DiagnosticCategory::Config, e.to_string())
            .with_location(DiagnosticLocation {
                file: args.config.clone(),
                line: e.line(),
                column: e.column(),
                length: None,
            })
            .with_inline_source(config_content.clone());
        anyhow::Error::new(Diagnostics::from(d))
    })?;

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
    if args.clean_cache {
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
            let color = Color::StderrIsTerminal;
            // Render warnings (respecting max-diagnostics cap)
            let warnings: Vec<_> = gen_result.diagnostics.warnings().collect();
            if !warnings.is_empty() {
                let max = args.max_diagnostics.unwrap_or(DEFAULT_MAX_DIAGNOSTICS);
                let show = if max > 0 {
                    max.min(warnings.len())
                } else {
                    warnings.len()
                };
                for w in &warnings[..show] {
                    let _ = diagnostic::render_diagnostic(w, None, color, &mut std::io::stderr());
                }
                if max > 0 && warnings.len() > max {
                    eprintln!(
                        "... and {} more warning{} (Hint: run with --max-diagnostics=0 to show all)",
                        warnings.len() - max,
                        if warnings.len() - max == 1 { "" } else { "s" }
                    );
                }
            }

            if !args.check {
                let write_result = if args.stdout {
                    let writer = StdoutWriter::new();
                    write_outputs(&gen_result.files, &writer)
                } else {
                    let writer = FsWriter::new();
                    let result = write_outputs(&gen_result.files, &writer);

                    for path in &result.written {
                        logger.file(&path.display().to_string());
                    }

                    result
                };

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
