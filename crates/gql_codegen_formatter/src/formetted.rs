use std::{fmt::Display, io};

use crate::{FormatterConfig, IndentStyle, QuoteStyle};

pub struct Formatted<'a> {
    value: String,
    config: &'a FormatterConfig,
    indent_level: u8,
}

impl<'a> Formatted<'a> {
    pub(crate) fn new(
        value: Option<String>,
        config: &'a FormatterConfig,
        indent_level: u8,
    ) -> Self {
        Self {
            value: value.unwrap_or_default(),
            config,
            indent_level,
        }
    }

    pub fn append(mut self, input: &str) -> Self {
        self.value.push_str(input);
        self
    }

    pub fn prepend(mut self, input: &str) -> Self {
        self.value.insert_str(0, input);
        self
    }

    pub fn quote(mut self) -> Self {
        let quote = match self.config.quote_style {
            Some(QuoteStyle::Single) => '\'',
            Some(QuoteStyle::Double) => '"',
            None => '\"',
        };

        self.value.insert(0, quote);
        self.value.push(quote);
        self
    }

    pub fn semi(mut self) -> Self {
        if self.config.semicolons.unwrap_or(false) {
            self.value.push(';');
        }
        self
    }

    pub fn indent(mut self) -> Self {
        let indent = match self.config.indent_style {
            Some(IndentStyle::Space) => " ".repeat(self.config.indent_width.unwrap_or(2)),
            Some(IndentStyle::Tab) => "\t".to_string(),
            None => " ".repeat(self.config.indent_width.unwrap_or(2)),
        };

        let indentation = indent.repeat(self.indent_level as usize);
        self.value = format!("{}{}", indentation, self.value);
        self
    }

    pub fn append_if(mut self, condition: bool, input: &str) -> Self {
        if condition {
            self.value.push_str(input);
        }
        self
    }

    pub fn writeln<T: io::Write>(&self, writer: &mut T) -> io::Result<()> {
        writeln!(writer, "{}", self.value)
    }

    pub fn write<T: io::Write>(&self, writer: &mut T) -> io::Result<()> {
        write!(writer, "{}", self.value)
    }
}

impl Display for Formatted<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}
