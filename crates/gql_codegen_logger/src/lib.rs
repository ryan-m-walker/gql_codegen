use colored::Colorize;

#[derive(Debug, Clone, Default, PartialEq)]
pub enum LogLevel {
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Default)]
pub struct Logger {
    pub level: LogLevel,
}

impl Logger {
    pub fn new(level: LogLevel) -> Self {
        Self { level }
    }

    pub fn debug(&self, message: &str) {
        match self.level {
            LogLevel::Debug => {
                println!("{}", format!("[DEBUG] {message}").dimmed());
            }
            _ => {}
        }
    }

    pub fn info(&self, message: &str) {
        match self.level {
            LogLevel::Debug | LogLevel::Info => {
                let prefix = "[INFO]".blue();
                println!("{prefix}  {message}");
            }
            _ => {}
        }
    }

    pub fn warn(&self, message: &str) {
        match self.level {
            LogLevel::Debug | LogLevel::Info | LogLevel::Warn => {
                println!("{}", format!("[WARN]  {message}").yellow());
            }
            _ => {}
        }
    }

    pub fn error(&self, message: &str) {
        let prefix = "[ERROR]".red();
        eprintln!("{}", format!("{prefix} {message}").red());
    }
}
