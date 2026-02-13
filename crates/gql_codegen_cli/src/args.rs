use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "sgc")]
#[command(about = "Speedy GraphQL Codegen - A fast GraphQL code generator")]
#[command(version)]
pub(crate) struct CliArgs {
    /// Path to the config file (JSON)
    #[arg(short, long, default_value = "codegen.json")]
    pub config: PathBuf,

    /// Check mode - validate without writing files
    #[arg(long)]
    pub check: bool,

    /// Print generated output to stdout instead of writing files
    #[arg(long)]
    pub stdout: bool,

    /// Disable caching (always regenerate)
    #[arg(long)]
    pub no_cache: bool,

    /// Clear the cache directory and exit
    #[arg(long)]
    pub clean_cache: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Suppress output (only show errors)
    #[arg(short, long)]
    pub quiet: bool,

    /// Show timing information for performance debugging
    #[arg(long)]
    pub timing: bool,

    /// Max diagnostics to show per error (0 = all, default 3)
    #[arg(long)]
    pub max_diagnostics: Option<usize>,
}
