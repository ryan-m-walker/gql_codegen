use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
pub enum IndentStyle {
    #[default]
    #[serde(rename = "space")]
    Space,

    #[serde(rename = "tab")]
    Tab,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
pub enum QuoteStyle {
    #[default]
    #[serde(rename = "double")]
    Single,

    #[serde(rename = "single")]
    Double,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
pub struct FormatterConfig {
    pub indent_style: Option<IndentStyle>,
    pub indent_width: Option<usize>,
    pub quote_style: Option<QuoteStyle>,
    pub semicolons: Option<bool>,
}

#[derive(Debug, Default)]
pub struct Formatter {
    config: FormatterConfig,
    indent_level: u8,
}

// TODO: escape quotes
impl Formatter {
    pub fn with_config(config: FormatterConfig) -> Self {
        Self {
            config,
            indent_level: 0,
        }
    }

    pub fn with_indent_style(mut self, indent_style: IndentStyle) -> Self {
        self.config.indent_style = Some(indent_style);
        self
    }

    pub fn with_indent_width(mut self, indent_width: usize) -> Self {
        self.config.indent_width = Some(indent_width);
        self
    }

    pub fn with_quote_style(mut self, quote_style: QuoteStyle) -> Self {
        self.config.quote_style = Some(quote_style);
        self
    }

    pub fn with_semicolons(mut self, semicolons: bool) -> Self {
        self.config.semicolons = Some(semicolons);
        self
    }

    pub fn indent_style(&self) -> IndentStyle {
        self.config.indent_style.unwrap_or_default()
    }

    pub fn indent_width(&self) -> usize {
        self.config.indent_width.unwrap_or(2)
    }

    pub fn indent(&self, input: &str) -> String {
        let mut indent = String::new();

        match self.indent_style() {
            IndentStyle::Space => {
                indent.push_str(&" ".repeat(self.indent_width()));
            }
            IndentStyle::Tab => {
                indent.push('\t');
            }
        }

        let indentation = indent.repeat(self.indent_level as usize);
        format!("{indentation}{input}")
    }

    pub fn increment_indent(&mut self) {
        self.indent_level += 1;
    }

    pub fn decrement_indent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    pub fn semicolon(&self) -> String {
        if self.config.semicolons.unwrap_or(true) {
            ";".to_string()
        } else {
            "".to_string()
        }
    }

    pub fn indent_with_semicolon(&self, input: &str) -> String {
        format!("{}{}", self.indent(input), self.semicolon())
    }
}
