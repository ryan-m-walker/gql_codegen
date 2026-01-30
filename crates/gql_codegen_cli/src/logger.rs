//! Simple colored logger for CLI output

use console::style;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Quiet,
    Normal,
    Verbose,
}

pub struct Logger {
    level: LogLevel,
}

impl Logger {
    pub fn new(level: LogLevel) -> Self {
        Self { level }
    }

    pub fn success(&self, msg: &str) {
        if self.level != LogLevel::Quiet {
            eprintln!("{} {}", style("✓").green().bold(), msg);
        }
    }

    pub fn info(&self, msg: &str) {
        if self.level != LogLevel::Quiet {
            eprintln!("{} {}", style("•").cyan(), msg);
        }
    }

    pub fn warn(&self, msg: &str) {
        if self.level != LogLevel::Quiet {
            eprintln!("{} {}", style("⚠").yellow().bold(), style(msg).yellow());
        }
    }

    pub fn error(&self, msg: &str) {
        eprintln!("{} {}", style("✗").red().bold(), style(msg).red());
    }

    pub fn debug(&self, msg: &str) {
        if self.level == LogLevel::Verbose {
            eprintln!("{} {}", style("→").dim(), style(msg).dim());
        }
    }

    pub fn file(&self, path: &str) {
        if self.level == LogLevel::Verbose {
            eprintln!("  {} {}", style("→").dim(), style(path).dim());
        }
    }

    pub fn is_verbose(&self) -> bool {
        self.level == LogLevel::Verbose
    }
}
